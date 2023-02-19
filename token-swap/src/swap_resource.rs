#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;


#[invariant_twostate(self.id() === old(self.id()))]
#[invariant_twostate(
    forall(|acct_id: AccountID, denom: PrefixedDenom|
        holds(Money(self.id(), acct_id, denom)) - 
        old(holds(Money(self.id(), acct_id, denom))) ==
        PermAmount::from(self.balance_of(acct_id, denom)) - 
            PermAmount::from(old(self.balance_of(acct_id, denom)))
    ))
]
#[invariant_twostate(
    forall(|coin: BaseDenom|
        holds(UnescrowedBalance(self.id(), coin)) - 
        old(holds(UnescrowedBalance(self.id(), coin))) ==
        PermAmount::from(self.unescrowed_coin_balance(coin)) - 
            PermAmount::from(old(self.unescrowed_coin_balance(coin)))
    ))
]
#[invariant_twostate(
    forall(|acct_id: AccountID, denom: PrefixedDenom|
        PermAmount::from(self.balance_of(acct_id, denom)) >=
        holds(Money(self.id(), acct_id, denom)) 
    ))]
struct Bank(u32);

#[derive(Copy, Clone)]
struct BankID(u32);

#[resource]
struct Money(BankID, AccountID, PrefixedDenom);

#[resource]
struct UnescrowedBalance(BankID, BaseDenom);

macro_rules! implies {
     ($lhs:expr => $rhs:expr) => {
        if $lhs { $rhs } else { true }
    }
}

macro_rules! transfer_money {
    ($bank_id:expr, $to:expr, $coin:expr) => {
    transfers(Money($bank_id, $to, $coin.denom), $coin.amount) && implies!( 
        !is_escrow_account($to) => 
            transfers(UnescrowedBalance($bank_id, $coin.denom.base_denom), $coin.amount)
    )}
}


impl Bank {

    #[pure]
    #[trusted]
    fn id(&self) -> BankID {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn unescrowed_coin_balance(&self, coin: BaseDenom) -> Amount {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn balance_of(&self, acct_id: AccountID, denom: PrefixedDenom) -> Amount {
        unimplemented!()
    }


    #[requires(from != to)]
    #[requires(transfer_money!(self.id(), from, coin))]
    #[ensures(transfer_money!(self.id(), to, coin))]
    fn transfer_tokens(
        &mut self,
        from: AccountID,
        to: AccountID,
        coin: &PrefixedCoin
    ) {
        self.burn_tokens(from, coin);
        self.mint_tokens(to, coin);
    }


    #[trusted]
    #[requires(transfer_money!(self.id(), from, coin))]
    #[requires(self.balance_of(from, coin.denom) >= coin.amount)]
    fn burn_tokens(&mut self, from: AccountID, coin: &PrefixedCoin) {
        unimplemented!()
    }

    #[trusted]
    #[ensures(transfer_money!(self.id(), to, coin))]
    fn mint_tokens(&mut self, to: AccountID, coin: &PrefixedCoin) {
        unimplemented!()
    }
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx, topology))]
#[requires(transfer_money!(bank.id(), sender, coin))]
#[ensures(
    !coin.denom.trace_path.starts_with(source_port, source_channel)
    ==> transfer_money!(
         bank.id(), 
         ctx.escrow_address(source_channel),
         coin
))]
#[ensures(
    result == mk_packet(
        ctx,
        source_port,
        source_channel,
        FungibleTokenPacketData {denom: coin.denom, sender, receiver, amount: coin.amount}
    )
)]
fn send_fungible_tokens(
    ctx: &Ctx,
    bank: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology
) -> Packet {
    if !coin.denom.trace_path.starts_with(source_port, source_channel) {
        bank.transfer_tokens(
            sender,
            ctx.escrow_address(source_channel),
            coin
        );
    } else {
        bank.burn_tokens(sender, coin);
    };

    let data = FungibleTokenPacketData {
        denom: coin.denom,
        sender,
        receiver,
        amount: coin.amount
    };
    mk_packet(ctx, source_port, source_channel, data)
}


macro_rules! refund_tokens_pre {
    ($ctx:expr, $bank:expr, $packet:expr) => { implies!(
        !$packet.data.denom.trace_path.starts_with($packet.source_port, $packet.source_channel) => 
            $ctx.escrow_address($packet.source_channel) != $packet.data.sender &&
            transfer_money!(
                $bank.id(),
                $ctx.escrow_address($packet.source_channel),
                PrefixedCoin {
                    denom: $packet.data.denom,
                    amount: $packet.data.amount
                }
        )
    )
}}

macro_rules! refund_tokens_post {
    ($bank:expr, $packet:expr) => {
        transfer_money!(
            $bank.id(), 
            $packet.data.sender, 
            PrefixedCoin {
                denom: $packet.data.denom,
                amount: $packet.data.amount
            }
        )
    }
}

#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
fn refund_tokens(ctx: &Ctx, bank: &mut Bank, packet: &Packet) {
    let FungibleTokenPacketData{ denom, sender, amount, ..} = packet.data;
    if !denom.trace_path.starts_with(packet.source_port, packet.source_channel) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.source_channel),
            sender,
            &PrefixedCoin { denom, amount }
        );
    } else {
        bank.mint_tokens(
            sender,
            &PrefixedCoin { denom, amount }
        );
    }
}

#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
fn on_timeout_packet(ctx: &Ctx, bank: &mut Bank, packet: &Packet) {
    refund_tokens(ctx, bank, packet);
}

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

#[requires(packet.data.receiver != ctx.escrow_address(packet.dest_channel))]
#[requires(
    is_well_formed(
        packet.data.denom.trace_path, 
        topology.ctx_at(
            ctx, 
            packet.dest_port,
            packet.dest_channel
        ),
        topology
    )
)]
#[requires(!packet_is_source(packet) && !packet.data.denom.trace_path.is_empty() ==> 
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.denom.trace_path.head_port(),
      packet.data.denom.trace_path.head_channel(),
))]
#[requires(packet_is_source(packet) ==> transfer_money!(
    bank.id(),
    ctx.escrow_address(packet.dest_channel), 
    PrefixedCoin { 
        denom: packet.data.denom,
        amount: packet.data.amount
    }.drop_prefix(packet.source_port, packet.source_channel)
))]
#[ensures(
    !packet_is_source(packet) ==>
        is_well_formed(
            packet.data.denom.trace_path.prepend_prefix(
                packet.dest_port, 
                packet.dest_channel
            ),
            ctx,
            topology)
)]
#[ensures(result.success)]
#[ensures(
    transfer_money!(
        bank.id(),
        packet.data.receiver,
        if packet_is_source(packet) {
            PrefixedCoin {
                denom: packet.data.denom,
                amount: packet.data.amount
            }.drop_prefix(packet.source_port, packet.source_channel)
        } else {
            PrefixedCoin {
                denom: packet.data.denom,
                amount: packet.data.amount
            }.prepend_prefix(packet.dest_port, packet.dest_channel)
        }
    )
)]
fn on_recv_packet(
    ctx: &Ctx, 
    bank: &mut Bank, 
    packet: &Packet,
    topology: &Topology
) -> FungibleTokenPacketAcknowledgement {
    let FungibleTokenPacketData{ denom, receiver, amount, ..} = packet.data;
    if packet_is_source(packet) {
        let coin = PrefixedCoin {
            denom: PrefixedDenom {
                trace_path: denom.trace_path.drop_prefix(packet.source_port, packet.source_channel),
                base_denom: denom.base_denom
            },
            amount
        };
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            receiver,
            &coin
        );
    } else {
        let coin = PrefixedCoin {
            denom: PrefixedDenom {
                trace_path: denom.trace_path.prepend_prefix(packet.dest_port, packet.dest_channel),
                base_denom: denom.base_denom
            },
            amount
        };
        bank.mint_tokens(receiver, &coin);
    };

    FungibleTokenPacketAcknowledgement { success: true }
}

#[requires(!ack.success ==> refund_tokens_pre!(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post!(bank, packet))]
fn on_acknowledge_packet(
    ctx: &Ctx,
    bank: &mut Bank,
    ack: FungibleTokenPacketAcknowledgement,
    packet: &Packet) {
    if(!ack.success) {
        refund_tokens(ctx, bank, packet);
    }
}

/*
 * This method performs a transfer chain A --> B
 * The specification ensures that the total amount of tokens does not change
 */

#[requires(!(bank1.id() === bank2.id()))]
#[requires(transfer_money!(bank1.id(), sender, coin))]

// Assume that the sender is the source chain
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]

// Neither sender or receiver is escrow
#[requires(!is_escrow_account(receiver))]
#[requires(!is_escrow_account(sender))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
#[ensures(transfer_money!(bank1.id(), ctx1.escrow_address(source_channel), coin))]
#[ensures(
    transfer_money!(
        bank2.id(), 
        receiver, 
        coin.prepend_prefix(dest_port, dest_channel) 
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
    bank1: &mut Bank,
    bank2: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
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
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank2.balance_of(acct_id2, denom) ==
           old(bank2).balance_of(acct_id2, denom)))]
fn round_trip(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    bank2: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
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
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
fn timeout(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
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
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
fn ack_fail(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
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
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank1.balance_of(acct_id2, denom) ==
           old(bank1).balance_of(acct_id2, denom)))]
#[ensures(
    forall(|acct_id2: AccountID, denom: PrefixedDenom|
        bank2.balance_of(acct_id2, denom) ==
           old(bank2).balance_of(acct_id2, denom)))]
fn round_trip_sink(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    bank2: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountID,
    receiver: AccountID,
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
