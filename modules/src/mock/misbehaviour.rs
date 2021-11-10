#[cfg(feature="prusti")]
use prusti_contracts::*;
use std::convert::{TryFrom, TryInto};

use tendermint_proto::Protobuf;

use ibc_proto::ibc::mock::Misbehaviour as RawMisbehaviour;

use crate::ics02_client::error::Error;
use crate::ics02_client::misbehaviour::AnyMisbehaviour;
use crate::ics24_host::identifier::ClientId;
use crate::mock::header::MockHeader;
use crate::Height;

#[derive(Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub struct Misbehaviour {
    pub client_id: ClientId,
    pub header1: MockHeader,
    pub header2: MockHeader,
}

impl crate::ics02_client::misbehaviour::Misbehaviour for Misbehaviour {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn client_id(&self) -> &ClientId {
        &self.client_id
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn height(&self) -> Height {
        self.header1.height()
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn wrap_any(self) -> AnyMisbehaviour {
        AnyMisbehaviour::Mock(self)
    }
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
    type Error = Error;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn try_from(raw: RawMisbehaviour) -> Result<Self, Self::Error> {
        Ok(Self {
            client_id: Default::default(),
            header1: raw
                .header1
                .ok_or_else(Error::missing_raw_misbehaviour)?
                .try_into()?,
            header2: raw
                .header2
                .ok_or_else(Error::missing_raw_misbehaviour)?
                .try_into()?,
        })
    }
}

impl From<Misbehaviour> for RawMisbehaviour {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(value: Misbehaviour) -> Self {
        RawMisbehaviour {
            client_id: value.client_id.to_string(),
            header1: Some(value.header1.into()),
            header2: Some(value.header2.into()),
        }
    }
}
