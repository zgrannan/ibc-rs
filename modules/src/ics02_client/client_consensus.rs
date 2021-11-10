#[cfg(feature="prusti")]
use prusti_contracts::*;
use core::marker::{Send, Sync};
use std::convert::TryFrom;

use chrono::{DateTime, Utc};
use prost_types::Any;
use serde::Serialize;
use std::convert::Infallible;
use tendermint_proto::Protobuf;

use ibc_proto::ibc::core::client::v1::ConsensusStateWithHeight;

use crate::events::IbcEventType;
use crate::ics02_client::client_type::ClientType;
use crate::ics02_client::error::Error;
use crate::ics02_client::height::Height;
use crate::ics07_tendermint::consensus_state;
use crate::ics23_commitment::commitment::CommitmentRoot;
use crate::ics24_host::identifier::ClientId;
use crate::timestamp::Timestamp;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::client_state::MockConsensusState;

pub const TENDERMINT_CONSENSUS_STATE_TYPE_URL: &str =
    "/ibc.lightclients.tendermint.v1.ConsensusState";

pub const MOCK_CONSENSUS_STATE_TYPE_URL: &str = "/ibc.mock.ConsensusState";

pub trait ConsensusState: Clone + Send + Sync {
    type Error;

    /// Type of client associated with this consensus state (eg. Tendermint)
    fn client_type(&self) -> ClientType;

    /// Commitment root of the consensus state, which is used for key-value pair verification.
    // fn root(&self) -> &CommitmentRoot;

    /// Performs basic validation of the consensus state
    fn validate_basic(&self) -> Result<(), Self::Error>;

    /// Wrap into an `AnyConsensusState`
    fn wrap_any(self) -> AnyConsensusState;
}

#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[derive(Clone)]
// #[serde(tag = "type")]
pub enum AnyConsensusState {
    Tendermint(consensus_state::ConsensusState),

    #[cfg(any(test, feature = "mocks"))]
    Mock(MockConsensusState),
}

impl AnyConsensusState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn timestamp(&self) -> Timestamp {
        match self {
            Self::Tendermint(cs_state) => {
                let date: DateTime<Utc> = cs_state.timestamp.into();
                Timestamp::from_datetime(date)
            }

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(mock_state) => mock_state.timestamp(),
        }
    }

    pub fn client_type(&self) -> ClientType {
        match self {
            AnyConsensusState::Tendermint(_cs) => ClientType::Tendermint,

            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(_cs) => ClientType::Mock,
        }
    }
}

#[cfg(feature="prusti")]
impl std::fmt::Debug for AnyConsensusState {
    #[cfg_attr(feature="prusti", trusted)]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}


impl Protobuf<Any> for AnyConsensusState {}

impl TryFrom<Any> for AnyConsensusState {
    type Error = Error;

    #[cfg_attr(feature="prusti", trusted)]
    fn try_from(value: Any) -> Result<Self, Self::Error> {
        match value.type_url.as_str() {
            "" => Err(Error::empty_consensus_state_response()),

            TENDERMINT_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Tendermint(
                consensus_state::ConsensusState::decode_vec(&value.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            #[cfg(any(test, feature = "mocks"))]
            MOCK_CONSENSUS_STATE_TYPE_URL => Ok(AnyConsensusState::Mock(
                MockConsensusState::decode_vec(&value.value)
                    .map_err(Error::decode_raw_client_state)?,
            )),

            _ => Err(Error::unknown_consensus_state_type(value.type_url)),
        }
    }
}

impl From<AnyConsensusState> for Any {
    fn from(value: AnyConsensusState) -> Self {
        match value {
            AnyConsensusState::Tendermint(value) => Any {
                type_url: TENDERMINT_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: value
                    .encode_vec()
                    .expect("encoding to `Any` from `AnyConsensusState::Tendermint`"),
            },
            #[cfg(any(test, feature = "mocks"))]
            AnyConsensusState::Mock(value) => Any {
                type_url: MOCK_CONSENSUS_STATE_TYPE_URL.to_string(),
                value: value
                    .encode_vec()
                    .expect("encoding to `Any` from `AnyConsensusState::Mock`"),
            },
        }
    }
}

#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub struct AnyConsensusStateWithHeight {
    pub height: Height,
    pub consensus_state: AnyConsensusState,
}

impl Protobuf<ConsensusStateWithHeight> for AnyConsensusStateWithHeight {}

impl TryFrom<ConsensusStateWithHeight> for AnyConsensusStateWithHeight {
    type Error = Error;

    fn try_from(value: ConsensusStateWithHeight) -> Result<Self, Self::Error> {
        let state = value
            .consensus_state
            .map(AnyConsensusState::try_from)
            .transpose()?
            .ok_or_else(Error::empty_consensus_state_response)?;

        Ok(AnyConsensusStateWithHeight {
            height: value.height.ok_or_else(Error::missing_height)?.into(),
            consensus_state: state,
        })
    }
}

impl From<AnyConsensusStateWithHeight> for ConsensusStateWithHeight {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(value: AnyConsensusStateWithHeight) -> Self {
        ConsensusStateWithHeight {
            height: Some(value.height.into()),
            consensus_state: Some(value.consensus_state.into()),
        }
    }
}

impl ConsensusState for AnyConsensusState {
    type Error = Infallible;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn client_type(&self) -> ClientType {
        self.client_type()
    }

    // fn root(&self) -> &CommitmentRoot {
    //     todo!()
    // }

    #[cfg_attr(feature="prusti", trusted)]
    fn validate_basic(&self) -> Result<(), Infallible> {
        todo!()
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn wrap_any(self) -> AnyConsensusState {
        self
    }
}

/// Query request for a single client event, identified by `event_id`, for `client_id`.
#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
pub struct QueryClientEventRequest {
    pub height: crate::Height,
    pub event_id: IbcEventType,
    pub client_id: ClientId,
    pub consensus_height: crate::Height,
}
