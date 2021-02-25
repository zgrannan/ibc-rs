use std::cmp::Ordering;

use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};

use super::{PacketResult, PacketType};
use crate::ics02_client::state::ClientState;
use crate::ics03_connection::connection::State as ConnectionState;

use crate::ics04_channel::channel::Counterparty;
use crate::ics04_channel::channel::{Order, State};
use crate::ics04_channel::events::SendPacket;
use crate::ics04_channel::msgs::recv_packet::MsgRecvPacket;
use crate::ics04_channel::packet::Sequence;
use crate::ics04_channel::{context::ChannelReader, error::Error, error::Kind, packet::Packet};

pub fn recv_packet(
    ctx: &dyn ChannelReader,
    msg: MsgRecvPacket,
) -> HandlerResult<PacketResult, Error> {
    let mut output = HandlerOutput::builder();

    let packet = msg.packet;
    let channel_end = ctx
        .channel_end(&(packet.source_port.clone(), packet.source_channel.clone()))
        .ok_or_else(|| {
            Kind::ChannelNotFound(packet.source_port.clone(), packet.source_channel.clone())
                .context(packet.source_channel.clone().to_string())
        })?;

    if !channel_end.state_matches(&State::Open) {
        return Err(Kind::InvalidChannelState(packet.source_channel, channel_end.state).into());
    }
    let _channel_cap = ctx.authenticated_capability(&packet.source_port)?;

    let channel_id = match channel_end.counterparty().channel_id() {
        Some(c) => Some(c.clone()),
        None => None,
    };

    let counterparty = Counterparty::new(channel_end.counterparty().port_id().clone(), channel_id);

    if !channel_end.counterparty_matches(&counterparty) {
        return Err(Kind::InvalidPacketCounterparty(
            packet.source_port.clone(),
            packet.source_channel,
        )
        .into());
    }

    let connection_end = ctx
        .connection_end(&channel_end.connection_hops()[0])
        .ok_or_else(|| Kind::MissingConnection(channel_end.connection_hops()[0].clone()))?;

    if !connection_end.state_matches(&ConnectionState::Open) {
        return Err(Kind::ConnectionNotOpen(channel_end.connection_hops()[0].clone()).into());
    }

    let client_id = connection_end.client_id().clone();

    let client_state = ctx
        .client_state(&client_id)
        .ok_or_else(|| Kind::MissingClientState(client_id.clone()))?;
    // check if packet height timeouted on the receiving chain

    // prevent accidental sends with clients that cannot be updated
    if client_state.is_frozen() {
        return Err(Kind::FrozenClient(connection_end.client_id().clone()).into());
    }

    let latest_height = client_state.latest_height();
    let packet_height = packet.timeout_height;

    if !packet.timeout_height.is_zero() && packet_height.cmp(&latest_height).eq(&Ordering::Less) {
        return Err(Kind::LowPacketHeight(latest_height, packet.timeout_height).into());
    }

    //check if packet timestamp timeouted on the receiving chain
    let consensus_state = ctx
        .client_consensus_state(&client_id, latest_height)
        .ok_or_else(|| Kind::MissingClientConsensusState(client_id.clone(), latest_height))?;

    let latest_timestamp = consensus_state.latest_timestamp();

    let packet_timestamp = packet.timeout_timestamp;
    if !packet.timeout_timestamp == 0 && packet_timestamp.cmp(&latest_timestamp).eq(&Ordering::Less)
    {
        return Err(Kind::LowPacketTimestamp.into());
    }

    //TODO: verify packet data
    // abortTransactionUnless(connection.verifyPacketData(
    //     proofHeight,
    //     proof,
    //     packet.sourcePort,
    //     packet.sourceChannel,
    //     packet.sequence,
    //     concat(packet.data, packet.timeoutHeight, packet.timeoutTimestamp)
    //   ))

    output.log("success: packet send ");
    let result;

    if channel_end.order_matches(&Order::Ordered) {
        let next_seq_recv = ctx
            .get_next_sequence_recv(&(packet.source_port.clone(), packet.source_channel.clone()))
            .ok_or(Kind::MissingNextRecvSeq)?;

        if !packet.sequence.eq(&Sequence::from(*next_seq_recv)) {
            return Err(Kind::InvalidPacketSequence(
                packet.sequence,
                Sequence::from(*next_seq_recv),
            )
            .into());
        }
        result = PacketResult {
            port_id: packet.source_port.clone(),
            channel_id: packet.source_channel.clone(),
            seq: packet.sequence.clone(),
            seq_number: Sequence::from(*next_seq_recv + 1),
            receipt: None,
            action: PacketType::Recv,
            data: packet.clone().data,
            timeout_height: packet.timeout_height,
            timeout_timestamp: packet.timeout_timestamp,
        };
    } else {
        let packet_rec = ctx.get_packet_receipt(&(
            packet.source_port.clone(),
            packet.source_channel.clone(),
            packet.sequence.clone(),
        ));

        match packet_rec {
            Some(_r) => {
                return Err(Kind::PacketReceived(<u64 as From<Sequence>>::from(
                    packet.sequence.clone(),
                ))
                .into())
            }
            None => {
                result = PacketResult {
                    port_id: packet.source_port.clone(),
                    channel_id: packet.source_channel.clone(),
                    seq: packet.sequence.clone(),
                    seq_number: Sequence::from(1),
                    receipt: Some("1".to_string()),
                    action: PacketType::Recv,
                    data: packet.clone().data,
                    timeout_height: packet.timeout_height,
                    timeout_timestamp: packet.timeout_timestamp,
                };
            }
        }
    }

    output.emit(IbcEvent::SendPacket(SendPacket {
        height: packet_height,
        packet,
    }));

    Ok(output.with_result(result))
}
