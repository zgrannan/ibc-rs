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
// SEND_PRESERVES_RESOURCE_SPEC_START
#[requires(transfer_money!(bank1.id(), sender, coin))]
#[requires(implies!(coin.denom.trace_path.starts_with(source_port, source_channel),
    transfer_money!(
        bank2.id(), 
        ctx2.escrow_address(dest_channel), 
        coin.drop_prefix(source_port, source_channel))
))]
// SEND_PRESERVES_RESOURCE_SPEC_END

// Neither sender or receiver is escrow
#[requires(!is_escrow_account(receiver))]
#[requires(!is_escrow_account(sender))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
// SEND_PRESERVES_RESOURCE_SPEC_START
#[ensures(implies!(!coin.denom.trace_path.starts_with(source_port, source_channel),
    transfer_money!(bank1.id(), ctx1.escrow_address(source_channel), coin)))]
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
        bank1.unescrowed_coin_balance(c) + bank2.unescrowed_coin_balance(c) ==
        old(bank1.unescrowed_coin_balance(c)) + old(bank2.unescrowed_coin_balance(c)))
    )
]
// SEND_PRESERVES_RESOURCE_SPEC_END
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

// ROUND_TRIP_RESOURCE_SPEC_START
#[requires(transfer_money!(bank1.id(), sender, coin))]

#[requires(implies!(coin.denom.trace_path.starts_with(source_port, source_channel),
    transfer_money!(
        bank2.id(),
        ctx2.escrow_address(dest_channel),
        coin.drop_prefix(source_port, source_channel)
    )
))]
// ROUND_TRIP_RESOURCE_SPEC_END

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]

// ROUND_TRIP_RESOURCE_SPEC_START
#[ensures(transfer_money!(bank1.id(), sender, coin))]
#[ensures(implies!(coin.denom.trace_path.starts_with(source_port, source_channel),
    transfer_money!(
    bank2.id(),
    ctx2.escrow_address(dest_channel),
    coin.drop_prefix(source_port, source_channel)
)))]

// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|acct_id2: AccountId, denom: PrefixedDenom|
        bank2.balance_of(acct_id2, denom) ==
           old(bank2).balance_of(acct_id2, denom)))]
// ROUND_TRIP_RESOURCE_SPEC_END
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
    on_acknowledge_packet(ctx1, bank1, ack, &packet);

    let coin = if coin.denom.trace_path.starts_with(source_port, source_channel) {
        coin.drop_prefix(source_port, source_channel)
    } else {
        coin.prepend_prefix(dest_port, dest_channel)
    };

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        &coin,
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