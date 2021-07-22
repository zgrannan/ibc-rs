use anomaly::{BoxError, Context};
use thiserror::Error;
use prusti_contracts::*;

use crate::ics24_host::identifier::{ChannelId, PortId};

pub type Error = anomaly::Error<Kind>;

impl std::fmt::Display for Kind {
    #[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::fmt::Debug for Kind {
    #[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}


#[derive(Clone, Error, PartialEq, Eq)]
pub enum Kind {
//     #[error("unrecognized ICS-20 transfer message type URL {0}")]
    UnknownMessageTypeUrl(String),

//     #[error("error raised by message handler")]
    HandlerRaisedError,

//     #[error("sending sequence number not found for port {0} and channel {1}")]
    SequenceSendNotFound(PortId, ChannelId),

//     #[error("missing channel for port_id {0} and channel_id {1} ")]
    ChannelNotFound(PortId, ChannelId),

//     #[error(
        // "destination channel not found in the counterparty of port_id {0} and channel_id {1} "
    // )]
    DestinationChannelNotFound(PortId, ChannelId),

//     #[error("invalid port identifier")]
    InvalidPortId(String),

//     #[error("invalid channel identifier")]
    InvalidChannelId(String),

//     #[error("invalid packet timeout height value")]
    InvalidPacketTimeoutHeight(String),

//     #[error("invalid packet timeout timestamp value")]
    InvalidPacketTimeoutTimestamp(u64),
}

impl Kind {
#[trusted]
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
unreachable!() //         Context::new(self, Some(source.into()))
    }
}
