use std::convert::TryFrom;

use prusti_contracts::*;
use prost_types::Any;
use tendermint_proto::Protobuf;

use crate::ics02_client::error::{Error, Kind};
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
pub trait Misbehaviour: Clone + std::fmt::Debug + Send + Sync {
    /// The type of client (eg. Tendermint)
    fn client_id(&self) -> &ClientId;

    /// The height of the consensus state
    fn height(&self) -> Height;

    fn wrap_any(self) -> AnyMisbehaviour;
}

#[derive(Clone, PartialEq)] // TODO: Add Eq bound once possible
#[allow(clippy::large_enum_variant)]
pub enum AnyMisbehaviour {
    Tendermint(TmMisbehaviour),

    #[cfg(any(test, feature = "mocks"))]
    Mock(MockMisbehaviour),
}

impl std::fmt::Debug for AnyMisbehaviour {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}

impl Misbehaviour for AnyMisbehaviour {
#[trusted]
    fn client_id(&self) -> &ClientId {
panic!("No") //         match self {
//             Self::Tendermint(misbehaviour) => misbehaviour.client_id(),
// 
//             #[cfg(any(test, feature = "mocks"))]
//             Self::Mock(misbehaviour) => misbehaviour.client_id(),
//         }
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

#[trusted]
    fn try_from(raw: Any) -> Result<Self, Self::Error> {
panic!("No") //         match raw.type_url.as_str() {
//             TENDERMINT_MISBEHAVIOR_TYPE_URL => Ok(AnyMisbehaviour::Tendermint(
//                 TmMisbehaviour::decode_vec(&raw.value)
//                     .map_err(|e| Kind::InvalidRawMisbehaviour.context(e))?,
//             )),
// 
//             #[cfg(any(test, feature = "mocks"))]
//             MOCK_MISBEHAVIOUR_TYPE_URL => Ok(AnyMisbehaviour::Mock(
//                 MockMisbehaviour::decode_vec(&raw.value)
//                     .map_err(|e| Kind::InvalidRawMisbehaviour.context(e))?,
//             )),
//             _ => Err(Kind::UnknownMisbehaviourType(raw.type_url).into()),
//         }
    }
}

impl From<AnyMisbehaviour> for Any {
#[trusted]
    fn from(value: AnyMisbehaviour) -> Self {
panic!("No") //         match value {
//             AnyMisbehaviour::Tendermint(misbehaviour) => Any {
//                 type_url: TENDERMINT_MISBEHAVIOR_TYPE_URL.to_string(),
//                 value: misbehaviour
//                     .encode_vec()
//                     .expect("encoding to `Any` from `AnyMisbehavior::Tendermint`"),
//             },
// 
//             #[cfg(any(test, feature = "mocks"))]
//             AnyMisbehaviour::Mock(misbehaviour) => Any {
//                 type_url: MOCK_MISBEHAVIOUR_TYPE_URL.to_string(),
//                 value: misbehaviour
//                     .encode_vec()
//                     .expect("encoding to `Any` from `AnyMisbehavior::Mock`"),
//             },
//         }
    }
}

impl std::fmt::Display for AnyMisbehaviour {
#[trusted]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
panic!("No") //         match self {
//             AnyMisbehaviour::Tendermint(tm) => write!(f, "{}", tm),
// 
//             #[cfg(any(test, feature = "mocks"))]
//             AnyMisbehaviour::Mock(mock) => write!(f, "{:?}", mock),
//         }
    }
}

#[derive(Clone, PartialEq)]
pub struct MisbehaviourEvidence {
    pub misbehaviour: AnyMisbehaviour,
    pub supporting_headers: Vec<AnyHeader>,
}
