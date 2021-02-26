use anomaly::Error;

use super::{context::ChannelReader, msgs::PacketMsg, packet::Sequence};
//use crate::ics04_channel::msgs::PacketMsg;
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
pub struct PacketResult {
    pub port_id: PortId,
    pub channel_id: ChannelId,
    pub seq: Sequence,
    pub action: PacketType,
    pub receipt: Option<String>,
    pub seq_number: Sequence,
    pub timeout_height: Height,
    pub timeout_timestamp: u64,
    pub data: Vec<u8>,
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