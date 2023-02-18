#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;


#[invariant_twostate(self.id() === old(self.id()))]
#[invariant_twostate(
    forall(|acct_id: AccountID, path: Path, coin: Coin|
        holds(Money(self.id(), acct_id, path, coin)) - 
        old(holds(Money(self.id(), acct_id, path, coin))) ==
        PermAmount::from(self.balance_of(acct_id, path, coin)) - 
            PermAmount::from(old(self.balance_of(acct_id, path, coin)))
    ))
]
#[invariant_twostate(
    forall(|coin: Coin|
        holds(UnescrowedBalance(self.id(), coin)) - 
        old(holds(UnescrowedBalance(self.id(), coin))) ==
        PermAmount::from(self.unescrowed_coin_balance(coin)) - 
            PermAmount::from(old(self.unescrowed_coin_balance(coin)))
    ))
]
#[invariant_twostate(
    forall(|acct_id: AccountID, path: Path, coin: Coin|
        PermAmount::from(self.balance_of(acct_id, path, coin)) >=
        holds(Money(self.id(), acct_id, path, coin)) 
    ))]
struct Bank(u32);

#[derive(Copy, Clone)]
struct BankID(u32);

#[resource]
struct Money(BankID, AccountID, Path, Coin);


#[resource]
struct UnescrowedBalance(BankID, Coin);

macro_rules! implies {
     ($lhs:expr => $rhs:expr) => {
        if $lhs { $rhs } else { true }
    }
}

macro_rules! transfer_money {
    ($bank_id:expr, $to:expr, $path:expr, $coin:expr, $amt:expr) => {
    transfers(Money($bank_id, $to, $path, $coin), $amt) && implies!( 
        !is_escrow_account($to) => transfers(UnescrowedBalance($bank_id, $coin), $amt)
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
    fn unescrowed_coin_balance(&self, coin: Coin) -> u32 {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn balance_of(&self, acct_id: AccountID, path: Path, coin: Coin) -> u32 {
        unimplemented!()
    }


    #[requires(from != to)]
    #[requires(transfer_money!(self.id(), from, path, coin, amt))]
    #[ensures(transfer_money!(self.id(), to, path, coin, amt))]
    fn transfer_tokens(
        &mut self,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) {
        self.burn_tokens(from, path, coin, amt);
        self.mint_tokens(to, path, coin, amt);
    }


    #[trusted]
    #[requires(transfer_money!(self.id(), from, path, coin, amt))]
    #[requires(self.balance_of(from, path, coin) >= amt)]
    fn burn_tokens(&mut self, from: AccountID, path: Path, coin: Coin, amt: u32) {
        unimplemented!()
    }

    #[trusted]
    #[ensures(transfer_money!(self.id(), to, path, coin, amt))]
    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) {
        unimplemented!()
    }
}

#[pure]
fn send_will_transfer(
    path: Path,
    source_port: Port,
    source_channel: ChannelEnd,
) -> bool {
    !path.starts_with(source_port, source_channel)
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(path, ctx, topology))]
#[requires(transfer_money!(bank.id(), sender, path, coin, amount))]
#[ensures(
    old(
        send_will_transfer(
            path,
            source_port,
            source_channel,
        )
    ) ==> transfer_money!(
            old(bank.id()), 
            old(ctx.escrow_address(source_channel)),
            path,
            coin,
            amount)
)]
#[ensures(
    result == mk_packet(
        ctx,
        source_port,
        source_channel,
        FungibleTokenPacketData {path, coin, sender, receiver, amount}
    )
)]
fn send_fungible_tokens(
    ctx: &Ctx,
    bank: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology
) -> Packet {
    if !path.starts_with(source_port, source_channel) {
        bank.transfer_tokens(
            sender,
            ctx.escrow_address(source_channel),
            path,
            coin,
            amount
        );
    } else {
        bank.burn_tokens(
            sender,
            path,
            coin,
            amount
        );
    };

    let data = FungibleTokenPacketData {
        path,
        coin,
        sender,
        receiver,
        amount
    };
    mk_packet(ctx, source_port, source_channel, data)
}


macro_rules! refund_tokens_pre {
    ($ctx:expr, $bank:expr, $packet:expr) => { implies!(
        !$packet.data.path.starts_with($packet.source_port, $packet.source_channel) => 
            $ctx.escrow_address($packet.source_channel) != $packet.data.sender &&
            transfer_money!(
                $bank.id(),
                $ctx.escrow_address($packet.source_channel),
                $packet.data.path,
                $packet.data.coin,
                $packet.data.amount
        )
    )
}}

macro_rules! refund_tokens_post {
    ($bank:expr, $packet:expr) => {
        transfer_money!(
            old($bank.id()), 
            old($packet.data.sender), 
            old($packet.data.path), 
            old($packet.data.coin),
            old($packet.data.amount)
        )
    }
}

#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
fn refund_tokens(ctx: &Ctx, bank: &mut Bank, packet: Packet) {
    let FungibleTokenPacketData{ path, coin, sender, amount, ..} = packet.data;
    if !path.starts_with(packet.source_port, packet.source_channel) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.source_channel),
            sender,
            path,
            coin,
            amount
        );
    } else {
        bank.mint_tokens(
            sender,
            path,
            coin,
            amount
        );
    }
}

#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
fn on_timeout_packet(ctx: &Ctx, bank: &mut Bank, packet: Packet) {
    refund_tokens(ctx, bank, packet);
}

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

#[pure]
fn packet_is_source(packet: Packet) -> bool {
    packet.data.path.starts_with(packet.source_port, packet.source_channel)
}

#[requires(packet.data.receiver != ctx.escrow_address(packet.dest_channel))]
#[requires(
    is_well_formed(
        packet.data.path, 
        topology.ctx_at(
            ctx, 
            packet.dest_port,
            packet.dest_channel
        ),
        topology
    )
)]
#[requires(!packet_is_source(packet) && !packet.data.path.is_empty() ==> 
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.path.head_port(),
      packet.data.path.head_channel(),
))]
#[requires(packet_is_source(packet) ==> 
    transfer_money!(
            bank.id(),
            ctx.escrow_address(packet.dest_channel), 
            packet.data.path.drop_prefix(packet.source_port, packet.source_channel),
            packet.data.coin,
            packet.data.amount))]
#[ensures(
    !packet_is_source(packet) ==>
        is_well_formed(
            old(
                packet.data.path.prepend_prefix(
                    packet.dest_port, 
                    packet.dest_channel)
            ),
            ctx,
            topology)
)]
#[ensures(result.success)]
#[ensures(
    transfer_money!(
            old(bank.id()),
            old(packet.data.receiver),
            if packet_is_source(packet) {
                old(packet.data.path.tail())
            } else {
                old(
                    packet.data.path.prepend_prefix(
                        packet.dest_port, packet.dest_channel
                    )
                )
            },
            old(packet.data.coin), 
            old(packet.data.amount))
    )
]
fn on_recv_packet(
    ctx: &Ctx, 
    bank: &mut Bank, 
    packet: Packet,
    topology: &Topology
) -> FungibleTokenPacketAcknowledgement {
    let FungibleTokenPacketData{ path, coin, receiver, amount, ..} = packet.data;
    if packet_is_source(packet) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            receiver,
            path.drop_prefix(packet.source_port, packet.source_channel),
            coin,
            amount
        );
    } else {
        bank.mint_tokens(
            receiver,
            path.prepend_prefix(packet.dest_port, packet.dest_channel),
            coin,
            amount
        );
    };

    FungibleTokenPacketAcknowledgement { success: true }
}

#[requires(!ack.success ==> refund_tokens_pre!(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post!(bank, packet))]
fn on_acknowledge_packet(
    ctx: &Ctx,
    bank: &mut Bank,
    ack: FungibleTokenPacketAcknowledgement,
    packet: Packet) {
    if(!ack.success) {
        refund_tokens(ctx, bank, packet);
    }
}

/*
 * This method performs a transfer chain A --> B
 * The specification ensures that the total amount of tokens does not change
 */

#[requires(!(bank1.id() === bank2.id()))]
#[requires(transfer_money!(bank1.id(), sender, path, coin, amount))]

// Assume that the sender is the source chain
#[requires(!path.starts_with(source_port, source_channel))]

// Neither sender or receiver is escrow
#[requires(!is_escrow_account(receiver))]
#[requires(!is_escrow_account(sender))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(path, ctx1, topology))]
#[ensures(transfer_money!(
        bank1.id(), 
        ctx1.escrow_address(source_channel), 
        path, 
        coin, 
        amount)
)]
#[ensures(
    transfer_money!(
        bank2.id(), 
        receiver, 
        path.prepend_prefix(dest_port, dest_channel), 
        coin,
        amount
    )
)]
#[ensures(
    forall(|c: Coin|
        (Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64) ==
        old(Int::new(bank1.unescrowed_coin_balance(c) as i64) + Int::new(bank2.unescrowed_coin_balance(c) as i64)))
    )
)]
fn send_preserves(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    bank2: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
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
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );
    let ack = on_recv_packet(ctx2, bank2, packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, packet);
}

/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both banks are the
 * same as they were initially.
 */

#[requires(!(bank1.id() === bank2.id()))]

#[requires(transfer_money!(bank1.id(), sender, path, coin, amount))]

// Assume that the sender is the source chain
#[requires(!path.starts_with(source_port, source_channel))]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(path, ctx1, topology))]

#[ensures(transfer_money!(bank1.id(), sender, path, coin, amount))]

// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank2.balance_of(acct_id2, path2, coin2) ==
           old(bank2).balance_of(acct_id2, path2, coin2)))]
fn round_trip(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    bank2: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
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
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    let ack = on_recv_packet(ctx2, bank2, packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, packet);

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        path.prepend_prefix(dest_port, dest_channel),
        coin,
        amount,
        receiver,
        sender,
        dest_port,
        dest_channel,
        topology
    );

    let ack = on_recv_packet(ctx1, bank1, packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx2, bank2, ack, packet);
}

#[requires(transfer_money!(bank1.id(), sender, path, coin, amount))]
#[requires(!path.starts_with(source_port, source_channel))]
// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(path, ctx1, topology))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
#[ensures(transfer_money!(bank1.id(), sender, path, coin, amount))]
fn timeout(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
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
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    on_timeout_packet(ctx1, bank1, packet);
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(transfer_money!(bank1.id(), sender, path, coin, amount))]
#[requires(!path.starts_with(source_port, source_channel))]
#[requires(is_well_formed(path, ctx1, topology))]
#[ensures(transfer_money!(bank1.id(), sender, path, coin, amount))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
fn ack_fail(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
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
        path,
        coin,
        amount,
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
        packet
    );
}


/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both banks are the
 * same as they were initially.
 */

#[requires(!(bank1.id() === bank2.id()))]
// Assume the sender has sufficient funds to send to receiver
#[requires(transfer_money!(bank1.id(), sender, path, coin, amount))]

// Assume the receiver is not the escrow address
#[requires(receiver != ctx2.escrow_address(dest_channel))]

// Assume that the sender is the sink chain
#[requires(path.starts_with(source_port, source_channel))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(path, ctx1, topology))]

// Assume the escrow has the corresponding locked tokens
#[requires(transfer_money!(
    bank2.id(),
    ctx2.escrow_address(dest_channel),
    path.drop_prefix(source_port, source_channel),
    coin, 
    amount)
)]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]

// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

#[ensures(transfer_money!(
    bank2.id(),
    ctx2.escrow_address(dest_channel),
    path.drop_prefix(source_port, source_channel),
    coin, 
    amount))]
#[ensures(transfer_money!(bank1.id(), sender, path, coin, amount))]
// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank2.balance_of(acct_id2, path2, coin2) ==
           old(bank2).balance_of(acct_id2, path2, coin2)))]
fn round_trip_sink(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut Bank,
    bank2: &mut Bank,
    path: Path,
    coin: Coin,
    amount: u32,
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
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel,
        topology
    );

    let ack = on_recv_packet(ctx2, bank2, packet, topology);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, packet);

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        path.drop_prefix(packet.source_port, packet.source_channel),
        coin,
        amount,
        receiver,
        sender,
        dest_port,
        dest_channel,
        topology
    );

    let ack = on_recv_packet(ctx1, bank1, packet, topology);
    on_acknowledge_packet(ctx2, bank2, ack, packet);
}
