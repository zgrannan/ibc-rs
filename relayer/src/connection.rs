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
