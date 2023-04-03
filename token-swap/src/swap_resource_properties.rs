#![allow(dead_code, unused)]
use prusti_contracts::*;
use crate::transfer_money;
use crate::implies;
use crate::types::*;
use crate::swap_resource::*;

/*
 * This method performs a transfer chain A --> B
 * The specification ensures that the total amount of tokens does not change
 */

#[requires(bank1.id() !== bank2.id())]
#[requires(transfer_money!(bank1.id(), sender, coin))]
#[requires(coin.denom.trace_path.starts_with(source_port, source_channel) ==>
    transfer_money!(bank2.id(), ctx2.escrow_address(dest_channel), coin.drop_prefix(source_port, source_channel)))]

// Neither sender or receiver is escrow
#[requires(!is_escrow_account(receiver))]
#[requires(!is_escrow_account(sender))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
#[ensures(!coin.denom.trace_path.starts_with(source_port, source_channel) ==>
    transfer_money!(bank1.id(), ctx1.escrow_address(source_channel), coin))]
#[ensures(
    transfer_money!(
        bank2.id(), 
        receiver, 
        if coin.denom.trace_path.starts_with(source_port, source_channel) {
            coin.drop_prefix(source_port, source_channel) 
        } else {
            coin.prepend_prefix(dest_port, dest_channel) 
        }
    )
)]
#[ensures(
    forall(|c: BaseDenom|
        (Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64) ==
        old(Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64)))
    )
)]
fn send_preserves(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut BankKeeper,
    bank2: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology
) {

    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        coin,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );
    let ack = on_recv_packet(ctx2, bank2, &packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, &packet);
}

/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both banks are the
 * same as they were initially.
 */

#[requires(!(bank1.id() === bank2.id()))]

#[requires(transfer_money!(bank1.id(), sender, coin))]

// Assume that the sender is the source chain
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]

#[ensures(transfer_money!(bank1.id(), sender, coin))]

// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank2.balance_of(acct_id2, denom) ==
           old(bank2).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|c: BaseDenom|
        (Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64) ==
        old(Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64)))
    )
)]
fn round_trip(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut BankKeeper,
    bank2: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology
) {
    // Send tokens A --> B

    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        coin,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    let ack = on_recv_packet(ctx2, bank2, &packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, &packet);

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        &coin.prepend_prefix(dest_port, dest_channel),
        receiver,
        sender,
        dest_port,
        dest_channel,
        topology
    );

    let ack = on_recv_packet(ctx1, bank1, &packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx2, bank2, ack, &packet);
}

#[requires(transfer_money!(bank1.id(), sender, coin))]
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]
// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
#[ensures(transfer_money!(bank1.id(), sender, coin))]
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
fn timeout(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology
) {
    // Send tokens A --> B
    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        coin,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    on_timeout_packet(ctx1, bank1, &packet);
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(transfer_money!(bank1.id(), sender, coin))]
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
#[ensures(transfer_money!(bank1.id(), sender, coin))]
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
fn ack_fail(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology
) {
    // Send tokens A --> B
    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        coin,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    on_acknowledge_packet(
        ctx1,
        bank1,
        FungibleTokenPacketAcknowledgement { success: false },
        &packet
    );
}


/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both banks are the
 * same as they were initially.
 */

#[requires(!(bank1.id() === bank2.id()))]
// Assume the sender has sufficient funds to send to receiver
#[requires(transfer_money!(bank1.id(), sender, coin))]

// Assume that the sender is the sink chain
#[requires(coin.denom.trace_path.starts_with(source_port, source_channel))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]

// Assume the escrow has the corresponding locked tokens
#[requires(transfer_money!(
    bank2.id(),
    ctx2.escrow_address(dest_channel),
    coin.drop_prefix(source_port, source_channel)
    )
)]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]

// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

#[ensures(transfer_money!(
    bank2.id(),
    ctx2.escrow_address(dest_channel),
    coin.drop_prefix(source_port, source_channel)
    ))]
#[ensures(transfer_money!(bank1.id(), sender, coin))]
// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank2.balance_of(acct_id2, denom) ==
           old(bank2).balance_of(acct_id2, denom)))]
fn round_trip_sink(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut BankKeeper,
    bank2: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    topology: &Topology
) {
    // Send tokens A --> B

    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        coin,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    let ack = on_recv_packet(ctx2, bank2, &packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, &packet);

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        &coin.drop_prefix(packet.source_port, packet.source_channel),
        receiver,
        sender,
        dest_port,
        dest_channel,
        topology
    );

    let ack = on_recv_packet(ctx1, bank1, &packet, topology);
    on_acknowledge_packet(ctx2, bank2, ack, &packet);
}