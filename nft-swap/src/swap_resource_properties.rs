#![allow(dead_code, unused)]
use prusti_contracts::*;
use crate::types::*;
use crate::swap_resource::*;
use crate::transfers_token;

//  /*
//  * This method performs a round trip of a token from chain A --> B --> A,
//  * The specification ensures that the resulting balances on both keepers are the
//  * same as they were initially.
//  */

//  #[requires(!(keeper1.id() === keeper2.id()))]

//  #[requires(transfers_token!(keeper1, class_id, token_id))]
//  #[requires(keeper1.get_owner(class_id, token_id) == Some(sender))]
//  #[requires(
//     if class_id.path.starts_with(source_port, source_channel) {
//         transfers_token!(keeper2, class_id.drop_prefix(source_port, source_channel), token_id) &&
//         keeper2.get_owner(
//             class_id.drop_prefix(source_port, source_channel), 
//             token_id
//         ) == Some(ctx2.escrow_address(dest_channel))
//     } else {
//         keeper2.get_owner(
//             class_id.prepend_prefix(dest_port, dest_channel), 
//             token_id
//         ) == None
// })]
//  // Sanity check: The sender cannot be an escrow account
//  #[requires(!is_escrow_account(sender))]
//  // Sanity check: Because this is a round-trip, the receiver cannot be an escrow
//  // account
//  #[requires(!is_escrow_account(receiver))]
 
//  // Assume the path is well-formed.
//  // See the definition of `is_well_formed` for details
//  #[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
//  #[requires(is_well_formed(class_id.path, ctx1, topology))]
 
//  #[ensures(transfers_token!(keeper1, class_id, token_id))]
 
//  // Ensure that the resulting balance of both keeper accounts are unchanged after the round-trip
//  #[ensures(
//      forall(|class_id: PrefixedClassId, token_id: TokenId|
//          keeper1.get_owner(class_id, token_id) ==
//             old(keeper1).get_owner(class_id, token_id)))]
//  #[ensures(
//      forall(|class_id: PrefixedClassId, token_id: TokenId|
//          keeper2.get_owner(class_id, token_id) ==
//             old(keeper2).get_owner(class_id, token_id)))]
//  fn round_trip(
//      ctx1: &Ctx,
//      ctx2: &Ctx,
//      keeper1: &mut NFTKeeper,
//      keeper2: &mut NFTKeeper,
//      class_id: PrefixedClassId,
//      token_id: Vec<TokenId>,
//      sender: AccountId,
//      receiver: AccountId,
//      source_port: Port,
//      source_channel: ChannelEnd,
//      dest_port: Port,
//      dest_channel: ChannelEnd,
//      topology: &Topology
//  ) {
//      // Send tokens A --> B
 
//      let packet = send_nft(
//          ctx1,
//          keeper1,
//          class_id,
//          token_id,
//          sender,
//          receiver,
//          source_port,
//          source_channel,
//          topology
//      );

//      let ack = on_recv_packet(ctx2, keeper2, &packet, topology);
//      on_acknowledge_packet(ctx1, keeper1, ack, &packet);

//      // Send tokens B --> A
 
//      let packet = send_nft(
//          ctx2,
//          keeper2,
//          packet.get_recv_class_id(),
//          token_id,
//          receiver,
//          sender,
//          dest_port,
//          dest_channel,
//          topology
//      );

//      let ack = on_recv_packet(ctx1, keeper1, &packet, topology);
//      on_acknowledge_packet(ctx2, keeper2, ack, &packet);

// }
 