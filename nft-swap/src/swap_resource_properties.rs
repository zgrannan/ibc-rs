#![allow(dead_code, unused)]
use prusti_contracts::*;
use crate::types::*;
use crate::swap_resource::*;
use crate::transfers_token;

 /*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both keepers are the
 * same as they were initially.
 */

 #[requires(!(keeper1.id() === keeper2.id()))]

 #[requires(transfers_token!(keeper1, sender, class_id, token_id))]
 #[requires(sender == keeper1.get_owner(class_id, token_id))]
 
 // Assume that the sender is the source chain
 #[requires(!class_id.path.starts_with(source_port, source_channel))]
 
 // Sanity check: The sender cannot be an escrow account
 #[requires(!is_escrow_account(sender))]
 // Sanity check: Because this is a round-trip, the receiver cannot be an escrow
 // account
 #[requires(!is_escrow_account(receiver))]
 
 // Assume the path is well-formed.
 // See the definition of `is_well_formed` for details
 #[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
 #[requires(is_well_formed(class_id.path, ctx1, topology))]
 
 #[ensures(transfers_token!(keeper1, sender, class_id, token_id))]
 
 // Ensure that the resulting balance of both keeper accounts are unchanged after the round-trip
 #[ensures(
     forall(|class_id: PrefixedClassId, token_id: TokenId|
         keeper1.get_owner(class_id, token_id) ==
            old(keeper1).get_owner(class_id, token_id)))]
 #[ensures(
     forall(|class_id: PrefixedClassId, token_id: TokenId|
         keeper2.get_owner(class_id, token_id) ==
            old(keeper2).get_owner(class_id, token_id)))]
//  #[ensures(
//      forall(|acct_id2: AccountId, denom: PrefixedDenom|
//          keeper2.balance_of(acct_id2, denom) ==
//             old(keeper2).balance_of(acct_id2, denom)))]
//  #[ensures(
//      forall(|c: BaseDenom|
//          (Int::new(keeper1.unescrowed_coin_balance(c) as i64) + Int::new(keeper2.unescrowed_coin_balance(c) as i64) ==
//          old(Int::new(keeper1.unescrowed_coin_balance(c) as i64) + Int::new(keeper2.unescrowed_coin_balance(c) as i64)))
//      )
//  )]
 fn round_trip(
     ctx1: &Ctx,
     ctx2: &Ctx,
     keeper1: &mut NFTKeeper,
     keeper2: &mut NFTKeeper,
     class_id: PrefixedClassId,
     token_id: TokenId,
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
         token_id,
         sender,
         receiver,
         source_port,
         source_channel,
         topology
     );
    prusti_assert!( keeper1.is_owner(
        ctx1.escrow_address(source_channel), 
        class_id,
        token_id)
    );

     prusti_assert!(!packet.is_source());
 
     let ack = on_recv_packet(ctx2, keeper2, &packet, topology);
     on_acknowledge_packet(ctx1, keeper1, ack, &packet);

    prusti_assert!(keeper1.is_owner(
        ctx1.escrow_address(source_channel), 
        class_id,
        token_id)
    );
    prusti_assert!(keeper2.is_owner(
        receiver,
        packet.get_recv_class_id(),
        token_id)
    );
 
     // Send tokens B --> A
 
     let packet = send_nft(
         ctx2,
         keeper2,
         class_id.prepend_prefix(dest_port, dest_channel),
         token_id,
         receiver,
         sender,
         dest_port,
         dest_channel,
         topology
     );
 
    prusti_assert!(keeper1.is_owner(
        ctx1.escrow_address(source_channel), 
        class_id,
        token_id)
    );
     let ack = on_recv_packet(ctx1, keeper1, &packet, topology);
     on_acknowledge_packet(ctx2, keeper2, ack, &packet);

    prusti_assert!(keeper1.is_owner(
        sender,
        class_id,
        token_id)
    );

}
 