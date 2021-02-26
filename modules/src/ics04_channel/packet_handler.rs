use send_packet::SendPacketResult;
use anomaly::Error;

use super::{context::ChannelReader, msgs::PacketMsg, packet::Sequence};
use crate::{handler::HandlerOutput, ics02_client::height::Height, ics24_host::identifier::{ChannelId, PortId}};

pub mod recv_packet;
pub mod send_packet;
pub mod verify;

#[derive(Clone, Debug, PartialEq)]
pub enum PacketType {
    Send,
    Recv,
    Ack,
    To,
    ToClose,
}

#[derive(Clone, Debug)]
pub enum PacketResult {
    Send(SendPacketResult),
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