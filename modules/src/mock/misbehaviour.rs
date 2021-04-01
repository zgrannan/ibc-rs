//use std::convert::{TryFrom, TryInto};

//use crate::ics02_client::error::{Error, Kind};
use chrono::{DateTime, Utc};
//use tendermint_proto::Protobuf;

//use ibc_proto::ibc::lightclients::tendermint::v1::Misbehaviour as RawMisbehaviour;

//import "ibc/core/client/v1/client.proto";

use crate::ics02_client::client_misbehaviour::AnyMisbehaviour;
// use crate::ics07_tendermint::error::{Error, Kind};
// use crate::ics07_tendermint::header::Header;
use crate::ics24_host::identifier::ClientId;
use crate::Height;

use super::header::MockHeader;

#[derive(Clone, Debug, PartialEq)]
pub struct Misbehaviour {
    pub client_id: ClientId,
    pub header1: MockHeader,
    pub header2: MockHeader,
}

impl Misbehaviour {
    pub fn default() -> Misbehaviour{
        todo!();
    }
}
impl crate::ics02_client::client_misbehaviour::Misbehaviour for Misbehaviour {
    fn client_id(&self) -> &ClientId {
        &self.client_id
    }

    fn height(&self) -> Height {
        self.header1.height()
    }

    fn time(&self) -> DateTime<Utc> {
       unimplemented!("no date for mock "); // self.header1.timestamp.into()
    }

    fn wrap_any(self) -> AnyMisbehaviour {
        AnyMisbehaviour::Mock(self)
    }
}

// impl Protobuf<RawMisbehaviour> for Misbehaviour {}

// impl TryFrom<RawMisbehaviour> for Misbehaviour {
//     type Error = Error;

//     fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
//         Ok(Self {
//             client_id: Default::default(),
//             header1: raw
//                 .header_1
//                 .ok_or_else(|| Kind::InvalidRawMisbehaviour.context("missing header1"))?
//                 .try_into()?,
//             header2: raw
//                 .header_2
//                 .ok_or_else(|| Kind::InvalidRawMisbehaviour.context("missing header2"))?
//                 .try_into()?,
//         })
//     }
// }

// impl From<Misbehaviour> for RawMisbehaviour {
//     fn from(value: Misbehaviour) -> Self {
//         RawMisbehaviour {
//             client_id: value.client_id.to_string(),
//             header_1: Some(value.header1.into()),
//             header_2: Some(value.header2.into()),
//         }
//     }
// }
