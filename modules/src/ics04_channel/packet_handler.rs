use send_packet::SendPacketResult;

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
