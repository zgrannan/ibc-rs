//! Protocol logic specific to ICS4 messages of type `MsgChannelCloseInit`.
use crate::events::IbcEvent;
use prusti_contracts::*;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::ics03_connection::connection::State as ConnectionState;
use crate::ics04_channel::channel::State;
use crate::ics04_channel::context::ChannelReader;
use crate::ics04_channel::error::{Error, Kind};
use crate::ics04_channel::events::Attributes;
use crate::ics04_channel::handler::{ChannelIdState, ChannelResult};
use crate::ics04_channel::msgs::chan_close_init::MsgChannelCloseInit;

#[trusted]
pub(crate) fn process(
    ctx: &dyn ChannelReader,
    msg: MsgChannelCloseInit,
) -> HandlerResult<ChannelResult, Error> {
unreachable!() //     let mut output = HandlerOutput::builder();
// 
//     // Unwrap the old channel end and validate it against the message.
//     let mut channel_end = ctx
//         .channel_end(&(msg.port_id().clone(), msg.channel_id().clone()))
//         .ok_or_else(|| Kind::ChannelNotFound(msg.port_id.clone(), msg.channel_id().clone()))?;
// 
//     // Validate that the channel end is in a state where it can be closed.
//     if channel_end.state_matches(&State::Closed) {
//         return Err(Into::<Error>::into(Kind::InvalidChannelState(
//             msg.channel_id().clone(),
//             channel_end.state,
//         )));
//     }
// 
//     // Channel capabilities
//     let channel_cap = ctx.authenticated_capability(&msg.port_id().clone())?;
//     // An OPEN IBC connection running on the local (host) chain should exist.
// 
//     if channel_end.connection_hops().len() != 1 {
//         return Err(
//             Kind::InvalidConnectionHopsLength(1, channel_end.connection_hops().len()).into(),
//         );
//     }
// 
//     let conn = ctx
//         .connection_end(&channel_end.connection_hops()[0])
//         .ok_or_else(|| Kind::MissingConnection(channel_end.connection_hops()[0].clone()))?;
// 
//     if !conn.state_matches(&ConnectionState::Open) {
//         return Err(Kind::ConnectionNotOpen(channel_end.connection_hops()[0].clone()).into());
//     }
// 
//     output.log("success: channel close init ");
// 
//     // Transition the channel end to the new state & pick a version.
//     channel_end.set_state(State::Closed);
// 
//     let result = ChannelResult {
//         port_id: msg.port_id().clone(),
//         channel_id: msg.channel_id().clone(),
//         channel_id_state: ChannelIdState::Reused,
//         channel_cap,
//         channel_end,
//     };
// 
//     let event_attributes = Attributes {
//         channel_id: Some(msg.channel_id().clone()),
//         ..Default::default()
//     };
//     output.emit(IbcEvent::CloseInitChannel(event_attributes.into()));
// 
//     Ok(output.with_result(result))
}
