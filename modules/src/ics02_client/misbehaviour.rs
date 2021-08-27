use std::convert::TryFrom;

#[cfg(feature="prusti")]
use prusti_contracts::*;
use prost_types::Any;
use tendermint_proto::Protobuf;

use crate::ics02_client::error::Error;
use crate::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;

#[cfg(any(test, feature = "mocks"))]
use crate::mock::misbehaviour::Misbehaviour as MockMisbehaviour;

use crate::ics24_host::identifier::ClientId;
use crate::Height;

use super::header::AnyHeader;

pub const TENDERMINT_MISBEHAVIOR_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.Misbehaviour";

#[cfg(any(test, feature = "mocks"))]
pub const MOCK_MISBEHAVIOUR_TYPE_URL: &str = "/ibc.mock.Misbehavior";

// #[dyn_clonable::clonable]
pub trait Misbehaviour: Clone + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_id(&self) -> &ClientId;

    /// The height of the consensus state
    fn height(&self) -> Height;

    fn wrap_any(self) -> AnyMisbehaviour;
}

#[derive(Clone)] // TODO: Add Eq bound once possible
#[allow(clippy::large_enum_variant)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
pub enum AnyMisbehaviour {
    Tendermint(TmMisbehaviour),

    #[cfg(any(test, feature = "mocks"))]
    Mock(MockMisbehaviour),
}

impl Misbehaviour for AnyMisbehaviour {
#[cfg_attr(feature="prusti", trusted)]
    fn client_id(&self) -> &ClientId {
        match self {
            Self::Tendermint(misbehaviour) => misbehaviour.client_id(),

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(misbehaviour) => misbehaviour.client_id(),
        }
    }

    fn height(&self) -> Height {
        match self {
            Self::Tendermint(misbehaviour) => misbehaviour.height(),

            #[cfg(any(test, feature = "mocks"))]
            Self::Mock(misbehaviour) => misbehaviour.height(),
        }
    }

    fn wrap_any(self) -> AnyMisbehaviour {
        self
    }
}

impl Protobuf<Any> for AnyMisbehaviour {}

impl TryFrom<Any> for AnyMisbehaviour {
    type Error = Error;

    #[cfg_attr(feature="prusti", trusted)]
    fn try_from(raw: Any) -> Result<Self, Error> {
        match raw.type_url.as_str() {
            TENDERMINT_MISBEHAVIOR_TYPE_URL => Ok(AnyMisbehaviour::Tendermint(
                TmMisbehaviour::decode_vec(&raw.value).map_err(Error::decode_raw_misbehaviour)?,
            )),

            #[cfg(any(test, feature = "mocks"))]
            MOCK_MISBEHAVIOUR_TYPE_URL => Ok(AnyMisbehaviour::Mock(
                MockMisbehaviour::decode_vec(&raw.value).map_err(Error::decode_raw_misbehaviour)?,
            )),
            _ => Err(Error::unknown_misbehaviour_type(raw.type_url)),
        }
    }
}

impl From<AnyMisbehaviour> for Any {
    fn from(value: AnyMisbehaviour) -> Self {
        match value {
            AnyMisbehaviour::Tendermint(misbehaviour) => Any {
                type_url: TENDERMINT_MISBEHAVIOR_TYPE_URL.to_string(),
                value: misbehaviour
                    .encode_vec()
                    .expect("encoding to `Any` from `AnyMisbehavior::Tendermint`"),
            },

            #[cfg(any(test, feature = "mocks"))]
            AnyMisbehaviour::Mock(misbehaviour) => Any {
                type_url: MOCK_MISBEHAVIOUR_TYPE_URL.to_string(),
                value: misbehaviour
                    .encode_vec()
                    .expect("encoding to `Any` from `AnyMisbehavior::Mock`"),
            },
        }
    }
}

impl std::fmt::Display for AnyMisbehaviour {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            AnyMisbehaviour::Tendermint(tm) => write!(f, "{}", tm),

            #[cfg(any(test, feature = "mocks"))]
            AnyMisbehaviour::Mock(mock) => write!(f, "{:?}", mock),
        }
    }
}

#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub struct MisbehaviourEvidence {
    pub misbehaviour: AnyMisbehaviour,
    pub supporting_headers: Vec<AnyHeader>,
}
