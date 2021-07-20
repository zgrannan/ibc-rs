use prusti_contracts::trusted;

use crate::ics02_client::client_state::ClientState;
use crate::ics02_client::{client_def::AnyClient, client_def::ClientDef};
use crate::ics03_connection::connection::ConnectionEnd;
use crate::ics04_channel::channel::ChannelEnd;
use crate::ics04_channel::context::ChannelReader;
use crate::ics04_channel::error::{Error, Kind};
use crate::ics04_channel::packet::{Packet, Sequence};
use crate::ics24_host::identifier::ClientId;
use crate::proofs::Proofs;

/// Entry point for verifying all proofs bundled in any ICS4 message for channel protocols.
#[trusted]
pub fn verify_channel_proofs(
   _ctx: &dyn ChannelReader,
   _channel_end: &ChannelEnd,
   _connection_end: &ConnectionEnd,
   _expected_chan: &ChannelEnd,
   _proofs: &Proofs,
) -> Result<(), Error> {
    panic!("no")
}

/// Entry point for verifying all proofs bundled in a ICS4 packet recv. message.
#[trusted]
pub fn verify_packet_recv_proofs(
    ctx: &dyn ChannelReader,
    packet: &Packet,
    client_id: ClientId,
    proofs: &Proofs,
) -> Result<(), Error> {
panic!("No") //     let client_state = ctx
//         .client_state(&client_id)
//         .ok_or_else(|| Kind::MissingClientState(client_id.clone()))?;
// 
//     // The client must not be frozen.
//     if client_state.is_frozen() {
//         return Err(Kind::FrozenClient(client_id).into());
//     }
// 
//     if ctx
//         .client_consensus_state(&client_id, proofs.height())
//         .is_none()
//     {
//         return Err(Kind::MissingClientConsensusState(client_id, proofs.height()).into());
//     }
// 
//     let client_def = AnyClient::from_client_type(client_state.client_type());
// 
//     let input = format!(
//         "{:?},{:?},{:?}",
//         packet.timeout_timestamp, packet.timeout_height, packet.data
//     );
//     let commitment = ctx.hash(input);
// 
//     // Verify the proof for the packet against the chain store.
//     Ok(client_def
//         .verify_packet_data(
//             &client_state,
//             proofs.height(),
//             proofs.object_proof(),
//             &packet.source_port,
//             &packet.source_channel,
//             &packet.sequence,
//             commitment,
//         )
//         .map_err(|_| Kind::PacketVerificationFailed(packet.sequence))?)
}

/// Entry point for verifying all proofs bundled in an ICS4 packet ack message.
#[trusted]
pub fn verify_packet_acknowledgement_proofs(
   _ctx: &dyn ChannelReader,
   _packet: &Packet,
   _acknowledgement: Vec<u8>,
   _client_id: ClientId,
   _proofs: &Proofs,
) -> Result<(), Error> {
    panic!("no")
    // let client_state = ctx
    //     .client_state(&client_id)
    //     .ok_or_else(|| Kind::MissingClientState(client_id.clone()))?;

    // // The client must not be frozen.
    // if client_state.is_frozen() {
    //     return Err(Kind::FrozenClient(client_id).into());
    // }

    // let client_def = AnyClient::from_client_type(client_state.client_type());

    // // Verify the proof for the packet against the chain store.
    // Ok(client_def
    //     .verify_packet_acknowledgement(
    //         &client_state,
    //         proofs.height(),
    //         proofs.object_proof(),
    //         &packet.source_port,
    //         &packet.source_channel,
    //         &packet.sequence,
    //         acknowledgement,
    //     )
    //     .map_err(|_| Kind::PacketVerificationFailed(packet.sequence))?)
}

/// Entry point for verifying all timeout proofs.
#[trusted]
pub fn verify_next_sequence_recv(
   _ctx: &dyn ChannelReader,
   _client_id: ClientId,
   _packet: Packet,
   _seq: Sequence,
   _proofs: &Proofs,
) -> Result<(), Error> {
    panic!("no")
    // let client_state = ctx
    //     .client_state(&client_id)
    //     .ok_or_else(|| Kind::MissingClientState(client_id.clone()))?;

    // // The client must not be frozen.
    // if client_state.is_frozen() {
    //     return Err(Kind::FrozenClient(client_id).into());
    // }

    // let client_def = AnyClient::from_client_type(client_state.client_type());

    // // Verify the proof for the packet against the chain store.
    // Ok(client_def
    //     .verify_next_sequence_recv(
    //         &client_state,
    //         proofs.height(),
    //         proofs.object_proof(),
    //         &packet.destination_port,
    //         &packet.destination_channel,
    //         &seq,
    //     )
    //     .map_err(|_| Kind::PacketVerificationFailed(seq))?)
}

#[trusted]
pub fn verify_packet_receipt_absence(
   _ctx: &dyn ChannelReader,
   _client_id: ClientId,
   _packet: Packet,
   _proofs: &Proofs,
) -> Result<(), Error> {
    panic!("no")
    // let client_state = ctx
    //     .client_state(&client_id)
    //     .ok_or_else(|| Kind::MissingClientState(client_id.clone()))?;

    // // The client must not be frozen.
    // if client_state.is_frozen() {
    //     return Err(Kind::FrozenClient(client_id).into());
    // }

    // let client_def = AnyClient::from_client_type(client_state.client_type());

    // // Verify the proof for the packet against the chain store.
    // Ok(client_def
    //     .verify_packet_receipt_absence(
    //         &client_state,
    //         proofs.height(),
    //         proofs.object_proof(),
    //         &packet.destination_port,
    //         &packet.destination_channel,
    //         &packet.sequence,
    //     )
    //     .map_err(|_| Kind::PacketVerificationFailed(packet.sequence))?)
}
