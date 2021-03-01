use send_packet::SendPacketResult;
use recv_packet::RecvPacketResult;
use crate::ics04_channel::error::Error;

use super::{context::ChannelReader, msgs::PacketMsg};
use crate::handler::HandlerOutput;

pub mod recv_packet;
pub mod send_packet;
pub mod verify;

// #[derive(Clone, Debug, PartialEq)]
// pub enum PacketType {
//     Send,
//     Recv,
//     Ack,
//     To,
//     ToClose,
// }

#[derive(Clone, Debug)]
pub enum PacketResult {
    Send(SendPacketResult),
    Recv(RecvPacketResult)
}

/// General entry point for processing any type of message related to the ICS4 channel open
/// handshake protocol.
pub fn packet_dispatch<Ctx>(ctx: &Ctx, msg: PacketMsg) -> Result<HandlerOutput<PacketResult>, Error>
where
    Ctx: ChannelReader,
{
    match msg {
        PacketMsg::RecvPacket(msg) => recv_packet::process(ctx, msg),
        _ => todo!(),
    }
}
