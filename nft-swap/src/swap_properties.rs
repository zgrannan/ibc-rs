#![allow(dead_code, unused)]
use crate::implies;
use crate::swap::*;
#[cfg(feature = "resource")]
use crate::token_permission;
use crate::types::*;
use prusti_contracts::*;

/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both keepers are the
 * same as they were initially.
 */

// ROUND_TRIP_COMMON_SPEC_START
#[requires(!(keeper1.id() == keeper2.id()))]
#[requires(
    forall(|i : usize| implies!(i < token_ids.len(),
        keeper1.get_owner(class_id, token_ids.get(i)) == Some(sender))
))]
#[requires(
    forall(|i : usize, j: usize| implies!(i < token_ids.len() && j < i,
        token_ids.get(i) != token_ids.get(j))
))]
// ROUND_TRIP_COMMON_SPEC_END
// ROUND_TRIP_RESOURCE_SPEC_START
#[cfg_attr(feature="resource", requires(
    forall(|i : usize| implies!(i < token_ids.len(),
        token_permission!(keeper1, class_id, token_ids.get(i)))))
)]
#[cfg_attr(feature="resource", requires(
    if class_id.path.starts_with(source_port, source_channel) {
        forall(|i : usize| implies!(i < token_ids.len(),
            token_permission!(keeper2, class_id.drop_prefix(source_port, source_channel), token_ids.get(i)))) &&
        forall(|i : usize| implies!(i < token_ids.len(),
            keeper2.get_owner(
                class_id.drop_prefix(source_port, source_channel),
                token_ids.get(i)
            ) == Some(ctx2.escrow_address(dest_channel))))
    } else {
        forall(|i : usize| implies!(i < token_ids.len(),
            keeper2.get_owner(
                class_id.prepend_prefix(
                    dest_port,
                    dest_channel
                ),
                token_ids.get(i)
            ) == None))
    }
))]
// ROUND_TRIP_RESOURCE_SPEC_END
// ROUND_TRIP_SPEC_START
#[cfg_attr(not(feature="resource"), requires(
    if class_id.path.starts_with(source_port, source_channel) {
        forall(|i : usize| implies!(i < token_ids.len(),
            keeper2.get_owner(
                class_id.drop_prefix(source_port, source_channel),
                token_ids.get(i)
            ) == Some(ctx2.escrow_address(dest_channel))))
    } else {
        forall(|i : usize| implies!(i < token_ids.len(),
            keeper2.get_owner(
                class_id.prepend_prefix(
                    dest_port,
                    dest_channel
                ),
                token_ids.get(i)
            ) == None))
    }
))]
// ROUND_TRIP_SPEC_END
// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]
// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(class_id.path, ctx1, topology))]
// Ensure that the resulting balance of both keeper accounts are unchanged after the round-trip
// ROUND_TRIP_COMMON_SPEC_START
#[ensures(
     forall(|class_id: PrefixedClassId, token_ids: TokenId|
         keeper1.get_owner(class_id, token_ids) ==
            old(keeper1).get_owner(class_id, token_ids)))]
#[ensures(
     forall(|class_id: PrefixedClassId, token_ids: TokenId|
         keeper2.get_owner(class_id, token_ids) ==
            old(keeper2).get_owner(class_id, token_ids)))]
// ROUND_TRIP_COMMON_SPEC_END
// ROUND_TRIP_RESOURCE_SPEC_START
#[cfg_attr(feature="resource", ensures(
    forall(|i : usize| implies!(i < token_ids.len(),
        token_permission!(keeper1, class_id, token_ids.get(i))))
    ))]
// ROUND_TRIP_RESOURCE_SPEC_END
fn round_trip<T: NFTKeeper>(
    ctx1: &Ctx,
    ctx2: &Ctx,
    keeper1: &mut T,
    keeper2: &mut T,
    class_id: PrefixedClassId,
    token_ids: TokenIdVec,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology,
) {
    // Send tokens A --> B

    let packet = send_nft(
        ctx1,
        keeper1,
        class_id,
        token_ids,
        sender,
        receiver,
        source_port,
        source_channel,
        topology,
    );

    if class_id.path.starts_with(source_port, source_channel) {
        prusti_assert!(
        forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == None
        ));
    } else {
        prusti_assert!(
            forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == Some(ctx1.escrow_address(source_channel))
        ));
    }

    let ack = on_recv_packet(ctx2, keeper2, &packet, topology);
    on_acknowledge_packet(ctx1, keeper1, ack, &packet);

    prusti_assert!(packet.data.receiver == receiver);
    prusti_assert!(packet.data.token_ids === token_ids);

    prusti_assert!(forall(|i : usize| i < token_ids.len() ==>
            keeper2.get_owner(
                packet.get_recv_class_id(),
                token_ids.get(i)
        ) === Some(receiver)));

    // Send tokens B --> A

    let packet = send_nft(
        ctx2,
        keeper2,
        packet.get_recv_class_id(),
        token_ids,
        receiver,
        sender,
        dest_port,
        dest_channel,
        topology,
    );

    prusti_assert!(packet.get_recv_class_id() === class_id);
    prusti_assert!(packet.dest_channel === source_channel);
    // prusti_assert!(packet.source_port === source_port);

    if class_id.path.starts_with(source_port, source_channel) {
        prusti_assert!(!packet
            .data
            .class_id
            .path
            .starts_with(packet.source_port, packet.source_channel));
        prusti_assert!(
        forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == None
        ));
    } else {
        prusti_assert!(packet
            .data
            .class_id
            .path
            .starts_with(packet.source_port, packet.source_channel));
        prusti_assert!(
            forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == Some(ctx1.escrow_address(source_channel))
        ));
    }

    let ack = on_recv_packet(ctx1, keeper1, &packet, topology);
    on_acknowledge_packet(ctx2, keeper2, ack, &packet);
}
