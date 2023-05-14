#![allow(dead_code, unused)]
use prusti_contracts::*;
use crate::types::*;
use crate::swap::*;

 /*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both keepers are the
 * same as they were initially.
 */

 #[requires(!(keeper1.id() === keeper2.id()))]

#[requires(
    forall(|i : usize| i < token_ids.len() ==>
        keeper1.get_owner(class_id, token_ids.get(i)) == Some(sender)
))]
#[requires(
    forall(|i : usize, j: usize| i < token_ids.len() && j < i ==>
        token_ids.get(i) != token_ids.get(j)
))]
#[requires(
    if class_id.path.starts_with(source_port, source_channel) {
        forall(|i : usize| i < token_ids.len() ==>
            keeper2.get_owner(
                class_id.drop_prefix(source_port, source_channel), 
                token_ids.get(i)
            ) == Some(ctx2.escrow_address(dest_channel)))
    } else {
        forall(|i : usize| i < token_ids.len() ==>
            keeper2.get_owner(
                class_id.prepend_prefix(
                    dest_port, 
                    dest_channel
                ),
                token_ids.get(i)
            ) == None)
    }
)]
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
 #[ensures(
     forall(|class_id: PrefixedClassId, token_ids: TokenId|
         keeper1.get_owner(class_id, token_ids) ==
            old(keeper1).get_owner(class_id, token_ids)))]
 #[ensures(
     forall(|class_id: PrefixedClassId, token_ids: TokenId|
         keeper2.get_owner(class_id, token_ids) ==
            old(keeper2).get_owner(class_id, token_ids)))]
 fn round_trip(
     ctx1: &Ctx,
     ctx2: &Ctx,
     keeper1: &mut NFTKeeper,
     keeper2: &mut NFTKeeper,
     class_id: PrefixedClassId,
     token_ids: TokenIdVec,
     sender: AccountId,
     receiver: AccountId,
     source_port: Port,
     source_channel: ChannelEnd,
     dest_port: Port,
     dest_channel: ChannelEnd,
     topology: &Topology
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
         topology
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
         topology
     );

    prusti_assert!(packet.get_recv_class_id() === class_id);
    prusti_assert!(packet.dest_channel === source_channel);
    // prusti_assert!(packet.source_port === source_port);

    if class_id.path.starts_with(source_port, source_channel) {
        prusti_assert!(
            !packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel)
        );
        prusti_assert!(
        forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == None
        ));
    } else {
        prusti_assert!(
            packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel)
        );
        prusti_assert!(
            forall(|i : usize| i < token_ids.len() ==>
                keeper1.get_owner(class_id, token_ids.get(i)) == Some(ctx1.escrow_address(source_channel))
        ));
    }


     let ack = on_recv_packet(ctx1, keeper1, &packet, topology);
     on_acknowledge_packet(ctx2, keeper2, ack, &packet);

}
 