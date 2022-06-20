use core::time::Duration;

use ibc_proto::google::protobuf::Any;
use serde::Serialize;
use tracing::{debug, error, info, warn};

use ibc::core::ics02_client::height::Height;
use ibc::core::ics03_connection::connection::{
    ConnectionEnd, Counterparty, IdentifiedConnectionEnd, State,
};
use ibc::core::ics03_connection::msgs::conn_open_ack::MsgConnectionOpenAck;
use ibc::core::ics03_connection::msgs::conn_open_confirm::MsgConnectionOpenConfirm;
use ibc::core::ics03_connection::msgs::conn_open_init::MsgConnectionOpenInit;
use ibc::core::ics03_connection::msgs::conn_open_try::MsgConnectionOpenTry;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::events::IbcEvent;
use ibc::timestamp::ZERO_DURATION;
use ibc::tx_msg::Msg;

use crate::chain::handle::ChainHandle;
use crate::chain::requests::{
    IncludeProof, PageRequest, QueryConnectionRequest, QueryConnectionsRequest,
};
use crate::chain::tracking::TrackedMsgs;
use crate::object::Connection as WorkerConnectionObject;
use crate::util::retry::{retry_count, retry_with_index, RetryResult};
use crate::util::task::Next;

mod error;
pub use error::ConnectionError;

/// Maximum value allowed for packet delay on any new connection that the relayer establishes.
pub const MAX_PACKET_DELAY: Duration = Duration::from_secs(120);

mod handshake_retry {
    //! Provides utility methods and constants to configure the retry behavior
    //! for the channel handshake algorithm.

    use crate::connection::ConnectionError;
    use crate::util::retry::{clamp_total, ConstantGrowth};
    use core::time::Duration;

    /// Approximate number of retries per block.
    const PER_BLOCK_RETRIES: u32 = 10;

    /// Defines the increment in delay between subsequent retries.
    /// A value of `0` will make the retry delay constant.
    const DELAY_INCREMENT: u64 = 0;

    /// Maximum retry delay expressed in number of blocks
    const BLOCK_NUMBER_DELAY: u32 = 10;

    /// The default retry strategy.
    /// We retry with a constant backoff strategy. The strategy is parametrized by the
    /// maximum block time expressed as a `Duration`.
    pub fn default_strategy(max_block_times: Duration) -> impl Iterator<Item = Duration> {
        let retry_delay = max_block_times / PER_BLOCK_RETRIES;

        clamp_total(
            ConstantGrowth::new(retry_delay, Duration::from_secs(DELAY_INCREMENT)),
            retry_delay,
            max_block_times * BLOCK_NUMBER_DELAY,
        )
    }

    /// Translates from an error type that the `retry` mechanism threw into
    /// a crate specific error of [`ConnectionError`] type.
    pub fn from_retry_error(
        e: retry::Error<ConnectionError>,
        description: String,
    ) -> ConnectionError {
        match e {
            retry::Error::Operation {
                error: _,
                total_delay,
                tries,
            } => ConnectionError::max_retry(description, tries, total_delay),
            retry::Error::Internal(reason) => ConnectionError::retry_internal(reason),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionSide<Chain: ChainHandle> {
    pub(crate) chain: Chain,
    client_id: ClientId,
    connection_id: Option<ConnectionId>,
}

impl<Chain: ChainHandle> ConnectionSide<Chain> {
    pub fn new(chain: Chain, client_id: ClientId, connection_id: Option<ConnectionId>) -> Self {
        Self {
            chain,
            client_id,
            connection_id,
        }
    }

    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.connection_id.as_ref()
    }

    pub fn map_chain<ChainB: ChainHandle>(
        self,
        mapper: impl FnOnce(Chain) -> ChainB,
    ) -> ConnectionSide<ChainB> {
        ConnectionSide {
            chain: mapper(self.chain),
            client_id: self.client_id,
            connection_id: self.connection_id,
        }
    }
}

impl<Chain: ChainHandle> Serialize for ConnectionSide<Chain> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Debug, Serialize)]
        struct ConnectionSide<'a> {
            client_id: &'a ClientId,
            connection_id: &'a Option<ConnectionId>,
        }

        let value = ConnectionSide {
            client_id: &self.client_id,
            connection_id: &self.connection_id,
        };

        value.serialize(serializer)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Connection<ChainA: ChainHandle, ChainB: ChainHandle> {
    pub delay_period: Duration,
    pub a_side: ConnectionSide<ChainA>,
    pub b_side: ConnectionSide<ChainB>,
}

impl<ChainA: ChainHandle, ChainB: ChainHandle> Connection<ChainA, ChainB> {

    pub fn restore_from_event(
        chain: ChainA,
        counterparty_chain: ChainB,
        connection_open_event: IbcEvent,
    ) -> Result<Connection<ChainA, ChainB>, ConnectionError> {
        let connection_event_attributes = connection_open_event
            .connection_attributes()
            .ok_or_else(|| ConnectionError::invalid_event(connection_open_event.clone()))?;

        let connection_id = connection_event_attributes.connection_id.clone();

        let counterparty_connection_id = connection_event_attributes
            .counterparty_connection_id
            .clone();

        let client_id = connection_event_attributes.client_id.clone();
        let counterparty_client_id = connection_event_attributes.counterparty_client_id.clone();

        Ok(Connection {
            // The event does not include the connection delay.
            delay_period: Default::default(),
            a_side: ConnectionSide::new(chain, client_id, connection_id),
            b_side: ConnectionSide::new(
                counterparty_chain,
                counterparty_client_id,
                counterparty_connection_id,
            ),
        })
    }

    /// Recreates a 'Connection' object from the worker's object built from chain state scanning.
    /// The connection must exist on chain.
    pub fn restore_from_state(
        chain: ChainA,
        counterparty_chain: ChainB,
        connection: WorkerConnectionObject,
        height: Height,
    ) -> Result<(Connection<ChainA, ChainB>, State), ConnectionError> {
        let (a_connection, _) = chain
            .query_connection(
                QueryConnectionRequest {
                    connection_id: connection.src_connection_id.clone(),
                    height,
                },
                IncludeProof::No,
            )
            .map_err(ConnectionError::relayer)?;

        let client_id = a_connection.client_id();
        let delay_period = a_connection.delay_period();

        let counterparty_connection_id = a_connection.counterparty().connection_id.clone();

        let counterparty_client_id = a_connection.counterparty().client_id();

        let mut handshake_connection = Connection {
            delay_period,
            a_side: ConnectionSide::new(
                chain,
                client_id.clone(),
                Some(connection.src_connection_id.clone()),
            ),
            b_side: ConnectionSide::new(
                counterparty_chain.clone(),
                counterparty_client_id.clone(),
                counterparty_connection_id.clone(),
            ),
        };

        if a_connection.state_matches(&State::Init) && counterparty_connection_id.is_none() {
            let connections: Vec<IdentifiedConnectionEnd> = counterparty_chain
                .query_connections(QueryConnectionsRequest {
                    pagination: Some(PageRequest::all()),
                })
                .map_err(ConnectionError::relayer)?;

            for conn in connections {
                if !conn
                    .connection_end
                    .client_id_matches(a_connection.counterparty().client_id())
                {
                    continue;
                }
                if let Some(remote_connection_id) =
                    conn.connection_end.counterparty().connection_id()
                {
                    if remote_connection_id == &connection.src_connection_id {
                        handshake_connection.b_side.connection_id = Some(conn.connection_id);
                        break;
                    }
                }
            }
        }

        Ok((handshake_connection, *a_connection.state()))
    }


    pub fn src_chain(&self) -> ChainA {
        self.a_side.chain.clone()
    }

    pub fn dst_chain(&self) -> ChainB {
        self.b_side.chain.clone()
    }

    pub fn a_chain(&self) -> ChainA {
        self.a_side.chain.clone()
    }

    pub fn b_chain(&self) -> ChainB {
        self.b_side.chain.clone()
    }

    pub fn src_client_id(&self) -> &ClientId {
        &self.a_side.client_id
    }

    pub fn dst_client_id(&self) -> &ClientId {
        &self.b_side.client_id
    }

    pub fn src_connection_id(&self) -> Option<&ConnectionId> {
        self.a_side.connection_id()
    }

    pub fn dst_connection_id(&self) -> Option<&ConnectionId> {
        self.b_side.connection_id()
    }

    pub fn a_connection_id(&self) -> Option<&ConnectionId> {
        self.a_side.connection_id()
    }
    pub fn b_connection_id(&self) -> Option<&ConnectionId> {
        self.b_side.connection_id()
    }

    fn a_connection(
        &self,
        connection_id: Option<&ConnectionId>,
    ) -> Result<ConnectionEnd, ConnectionError> {
        if let Some(id) = connection_id {
            self.a_chain()
                .query_connection(
                    QueryConnectionRequest {
                        connection_id: id.clone(),
                        height: Height::zero(),
                    },
                    IncludeProof::No,
                )
                .map(|(connection_end, _)| connection_end)
                .map_err(|e| ConnectionError::chain_query(self.a_chain().id(), e))
        } else {
            Ok(ConnectionEnd::default())
        }
    }

    fn b_connection(
        &self,
        connection_id: Option<&ConnectionId>,
    ) -> Result<ConnectionEnd, ConnectionError> {
        if let Some(id) = connection_id {
            self.b_chain()
                .query_connection(
                    QueryConnectionRequest {
                        connection_id: id.clone(),
                        height: Height::zero(),
                    },
                    IncludeProof::No,
                )
                .map(|(connection_end, _)| connection_end)
                .map_err(|e| ConnectionError::chain_query(self.b_chain().id(), e))
        } else {
            Ok(ConnectionEnd::default())
        }
    }

    /// Returns a `Duration` representing the maximum value among the
    /// [`ChainConfig.max_block_time`] for the two networks that
    /// this connection belongs to.
    fn max_block_times(&self) -> Result<Duration, ConnectionError> {
        let a_block_time = self
            .a_chain()
            .config()
            .map_err(ConnectionError::relayer)?
            .max_block_time;
        let b_block_time = self
            .b_chain()
            .config()
            .map_err(ConnectionError::relayer)?
            .max_block_time;
        Ok(a_block_time.max(b_block_time))
    }

    pub fn flipped(&self) -> Connection<ChainB, ChainA> {
        Connection {
            a_side: self.b_side.clone(),
            b_side: self.a_side.clone(),
            delay_period: self.delay_period,
        }
    }

    /// Queries the chains for latest connection end information. It verifies the relayer connection
    /// IDs and updates them if needed.
    /// Returns the states of the two connection ends.
    ///
    /// The relayer connection stores the connection identifiers on the two chains a and b.
    /// These identifiers need to be cross validated with the corresponding on-chain ones at some
    /// handshake steps.
    /// This is required because of crossing handshake messages in the presence of multiple relayers.
    ///
    /// Chain a is queried with the relayer's `a_side.connection_id` (`relayer_a_id`) with result
    /// `a_connection`. If the counterparty id of this connection, `a_counterparty_id`,
    /// is some id then it must match the relayer's `b_side.connection_id` (`relayer_b_id`).
    /// A similar check is done for the `b_side` of the connection.
    ///
    ///  a                                 relayer                                    b
    ///  |                     a_side -- connection -- b_side                         |
    ///  a_id _____________> relayer_a_id             relayer_b_id <______________> b_id
    ///  |                      \                                /                    |
    /// a_counterparty_id <_____________________________________/                     |
    ///                           \____________________________________>   b_counterparty_id
    ///
    /// Case 1 (fix connection ID):
    ///  a                                                      b
    ///  | <-- Init (r1)                                        |
    ///  | a_id = 1, a_counterparty_id = None                   |
    ///  |                                         Try (r2) --> |
    ///  |                    b_id = 100, b_counterparty_id = 1 |
    ///  |                                         Try (r1) --> |
    ///  |                    b_id = 101, b_counterparty_id = 1 |
    ///  | <-- Ack (r2)
    ///  | a_id = 1, a_counterparty_id = 100
    ///
    /// Here relayer r1 has a_side connection 1 and b_side connection 101
    /// while on chain a the counterparty of connection 1 is 100. r1 needs to update
    /// its b_side to 100
    ///
    /// Case 2 (update from None to some connection ID):
    ///  a                                                      b
    ///  | <-- Init (r1)                                        |
    ///  | a_id = 1, a_counterparty_id = None                   |
    ///  |                                         Try (r2) --> |
    ///  |                    b_id = 100, b_counterparty_id = 1 |
    ///  | <-- Ack (r2)
    ///  | a_id = 1, a_counterparty_id = 100
    ///
    /// Here relayer r1 has a_side connection 1 and b_side is unknown
    /// while on chain a the counterparty of connection 1 is 100. r1 needs to update
    /// its b_side to 100
    fn update_connection_and_query_states(&mut self) -> Result<(State, State), ConnectionError> {
        let relayer_a_id = self.a_side.connection_id();
        let relayer_b_id = self.b_side.connection_id().cloned();

        let a_connection = self.a_connection(relayer_a_id)?;
        let a_counterparty_id = a_connection.counterparty().connection_id();

        if a_counterparty_id.is_some() && a_counterparty_id != relayer_b_id.as_ref() {
            warn!(
                "updating the expected {:?} of side_b({}) since it is different than the \
                counterparty of {:?}: {:?}, on {}. This is typically caused by crossing handshake \
                messages in the presence of multiple relayers.",
                relayer_b_id,
                self.b_chain().id(),
                relayer_a_id,
                a_counterparty_id,
                self.a_chain().id(),
            );
            self.b_side.connection_id = a_counterparty_id.cloned();
        }

        let updated_relayer_b_id = self.b_side.connection_id();
        let b_connection = self.b_connection(updated_relayer_b_id)?;
        let b_counterparty_id = b_connection.counterparty().connection_id();

        if b_counterparty_id.is_some() && b_counterparty_id != relayer_a_id {
            if updated_relayer_b_id == relayer_b_id.as_ref() {
                warn!(
                    "updating the expected {:?} of side_b({}) since it is different than the \
                counterparty of {:?}: {:?}, on {}. This is typically caused by crossing handshake \
                messages in the presence of multiple relayers.",
                    relayer_a_id,
                    self.a_chain().id(),
                    updated_relayer_b_id,
                    b_counterparty_id,
                    self.b_chain().id(),
                );
                self.a_side.connection_id = b_counterparty_id.cloned();
            } else {
                panic!(
                    "mismatched connection ids in connection ends: {} - {:?} and {} - {:?}",
                    self.a_chain().id(),
                    a_connection,
                    self.b_chain().id(),
                    b_connection,
                );
            }
        }
        Ok((*a_connection.state(), *b_connection.state()))
    }

    /// Retrieves the connection from destination and compares against the expected connection
    /// built from the message type (`msg_type`) and options (`opts`).
    /// If the expected and the destination connections are compatible, it returns the expected connection.
    fn validated_expected_connection(
        &self,
        msg_type: ConnectionMsgType,
    ) -> Result<ConnectionEnd, ConnectionError> {
        let dst_connection_id = self
            .dst_connection_id()
            .ok_or_else(ConnectionError::missing_counterparty_connection_id)?;

        let prefix = self
            .src_chain()
            .query_commitment_prefix()
            .map_err(|e| ConnectionError::chain_query(self.src_chain().id(), e))?;

        // If there is a connection present on the destination chain, it should look like this:
        let counterparty = Counterparty::new(
            self.src_client_id().clone(),
            self.src_connection_id().cloned(),
            prefix,
        );

        // The highest expected state, depends on the message type:
        let highest_state = match msg_type {
            ConnectionMsgType::OpenAck => State::TryOpen,
            ConnectionMsgType::OpenConfirm => State::TryOpen,
            _ => State::Uninitialized,
        };

        let versions = self
            .src_chain()
            .query_compatible_versions()
            .map_err(|e| ConnectionError::chain_query(self.src_chain().id(), e))?;

        let dst_expected_connection = ConnectionEnd::new(
            highest_state,
            self.dst_client_id().clone(),
            counterparty,
            versions,
            ZERO_DURATION,
        );

        // Retrieve existing connection if any
        let (dst_connection, _) = self
            .dst_chain()
            .query_connection(
                QueryConnectionRequest {
                    connection_id: dst_connection_id.clone(),
                    height: Height::zero(),
                },
                IncludeProof::No,
            )
            .map_err(|e| ConnectionError::chain_query(self.dst_chain().id(), e))?;

        // Check if a connection is expected to exist on destination chain
        // A connection must exist on destination chain for Ack and Confirm Tx-es to succeed
        if dst_connection.state_matches(&State::Uninitialized) {
            return Err(ConnectionError::missing_connection_id(
                self.dst_chain().id(),
            ));
        }

        check_destination_connection_state(
            dst_connection_id.clone(),
            dst_connection,
            dst_expected_connection.clone(),
        )?;

        Ok(dst_expected_connection)
    }

    pub fn build_conn_init(&self) -> Result<Vec<Any>, ConnectionError> {
        // Get signer
        let signer = self
            .dst_chain()
            .get_signer()
            .map_err(|e| ConnectionError::signer(self.dst_chain().id(), e))?;

        let prefix = self
            .src_chain()
            .query_commitment_prefix()
            .map_err(|e| ConnectionError::chain_query(self.src_chain().id(), e))?;

        let counterparty = Counterparty::new(self.src_client_id().clone(), None, prefix);

        let version = self
            .dst_chain()
            .query_compatible_versions()
            .map_err(|e| ConnectionError::chain_query(self.dst_chain().id(), e))?[0]
            .clone();

        // Build the domain type message
        let new_msg = MsgConnectionOpenInit {
            client_id: self.dst_client_id().clone(),
            counterparty,
            version: Some(version),
            delay_period: self.delay_period,
            signer,
        };

        Ok(vec![new_msg.to_any()])
    }

    pub fn build_conn_init_and_send(&self) -> Result<IbcEvent, ConnectionError> {
        let dst_msgs = self.build_conn_init()?;

        let tm = TrackedMsgs::new_static(dst_msgs, "ConnectionOpenInit");

        let events = self
            .dst_chain()
            .send_messages_and_wait_commit(tm)
            .map_err(|e| ConnectionError::submit(self.dst_chain().id(), e))?;

        // Find the relevant event for connection init
        let event = events
            .into_iter()
            .find(|event| {
                matches!(event, IbcEvent::OpenInitConnection(_))
                    || matches!(event, IbcEvent::ChainError(_))
            })
            .ok_or_else(ConnectionError::missing_connection_init_event)?;

        // TODO - make chainError an actual error
        match event {
            IbcEvent::OpenInitConnection(_) => {
                info!("🥂 {} => {:#?}\n", self.dst_chain().id(), event);
                Ok(event)
            }
            IbcEvent::ChainError(e) => Err(ConnectionError::tx_response(e)),
            _ => panic!("internal error"),
        }
    }

    pub fn map_chain<ChainC: ChainHandle, ChainD: ChainHandle>(
        self,
        mapper_a: impl Fn(ChainA) -> ChainC,
        mapper_b: impl Fn(ChainB) -> ChainD,
    ) -> Connection<ChainC, ChainD> {
        Connection {
            delay_period: self.delay_period,
            a_side: self.a_side.map_chain(mapper_a),
            b_side: self.b_side.map_chain(mapper_b),
        }
    }
}

pub fn extract_connection_id(event: &IbcEvent) -> Result<&ConnectionId, ConnectionError> {
    match event {
        IbcEvent::OpenInitConnection(ev) => ev.connection_id(),
        IbcEvent::OpenTryConnection(ev) => ev.connection_id(),
        IbcEvent::OpenAckConnection(ev) => ev.connection_id(),
        IbcEvent::OpenConfirmConnection(ev) => ev.connection_id(),
        _ => None,
    }
    .ok_or_else(ConnectionError::missing_connection_id_from_event)
}

/// Enumeration of proof carrying ICS3 message, helper for relayer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionMsgType {
    OpenTry,
    OpenAck,
    OpenConfirm,
}

/// Verify that the destination connection exhibits the expected state.
fn check_destination_connection_state(
    connection_id: ConnectionId,
    existing_connection: ConnectionEnd,
    expected_connection: ConnectionEnd,
) -> Result<(), ConnectionError> {
    let good_client_ids = existing_connection.client_id() == expected_connection.client_id()
        && existing_connection.counterparty().client_id()
            == expected_connection.counterparty().client_id();

    let good_state = *existing_connection.state() as u32 <= *expected_connection.state() as u32;

    let good_connection_ids = existing_connection.counterparty().connection_id().is_none()
        || existing_connection.counterparty().connection_id()
            == expected_connection.counterparty().connection_id();

    let good_version = existing_connection.versions() == expected_connection.versions();

    let good_counterparty_prefix =
        existing_connection.counterparty().prefix() == expected_connection.counterparty().prefix();

    if good_state
        && good_client_ids
        && good_connection_ids
        && good_version
        && good_counterparty_prefix
    {
        Ok(())
    } else {
        Err(ConnectionError::connection_already_exists(connection_id))
    }
}
