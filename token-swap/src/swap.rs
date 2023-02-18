#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;

struct Bank(u32);

impl Bank {

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

    predicate! {
        fn transfer_tokens_post(
            &self,
            old_bank: &Self,
            from: AccountID,
            to: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
        ((is_escrow_account(to) && !is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin) - amt) &&
        ((!is_escrow_account(to) && is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin) + amt) &&
        ((is_escrow_account(to) == is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin)) &&
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
            self.balance_of(acct_id2, path2, coin2) ==
                if(acct_id2 == from && coin == coin2 && path === path2) {
                    old_bank.balance_of(from, path, coin) - amt
                } else if (acct_id2 == to && coin == coin2 && path === path2){
                    old_bank.balance_of(to, path, coin) + amt
                } else {
                    old_bank.balance_of(acct_id2, path2, coin2)
                }
        ) && forall(|c: Coin| c != coin ==> 
            self.unescrowed_coin_balance(c) == old_bank.unescrowed_coin_balance(c)
        )
        }
    }

    #[pure]
    fn transfer_tokens_pre(
        &self,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) -> bool {
        from != to && self.balance_of(from, path, coin) >= amt
    }

    #[requires(self.transfer_tokens_pre(from, to, path, coin, amt))]
    #[ensures(self.transfer_tokens_post(
        old(self),
        from,
        to,
        path,
        coin,
        amt
    ))]
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

    predicate! {
        fn burn_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin)
            } else {
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin) - amt
            } && forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
            self.balance_of(acct_id2, path2, coin2) ==
                if(acct_id == acct_id2 && coin == coin2 && path === path2) {
                    old_bank.balance_of(acct_id, path, coin) - amt
                } else {
                    old_bank.balance_of(acct_id2, path2, coin2)
                }
            ) && forall(|c: Coin| c != coin ==> 
                self.unescrowed_coin_balance(c) == old_bank.unescrowed_coin_balance(c)
            )
        }
    }

    #[requires(self.balance_of(to, path, coin) >= amt)]
    #[ensures(self.burn_tokens_post(old(self), to, path, coin, amt))]
    #[trusted]
    fn burn_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) {
        unimplemented!()
    }

    predicate! {
        fn mint_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin)
            } else {
              self.unescrowed_coin_balance(coin) ==
                old_bank.unescrowed_coin_balance(coin) + amt
            } && forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
                self.balance_of(acct_id2, path2, coin2) ==
                    if(acct_id == acct_id2 && coin == coin2 && path === path2) {
                        old_bank.balance_of(acct_id, path, coin) + amt
                    } else {
                        old_bank.balance_of(acct_id2, path2, coin2)
                    }
            ) && forall(|c: Coin| c != coin ==> 
                self.unescrowed_coin_balance(c) == old_bank.unescrowed_coin_balance(c)
            )

        }
    }

    #[ensures(result)]
    #[ensures(self.mint_tokens_post(old(self), to, path, coin, amt))]
    #[trusted]
    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool {
        unimplemented!()
    }
}

#[pure]
fn send_will_burn(
    bank: &Bank,
    path: Path,
    source_port: Port,
    source_channel: ChannelEnd,
    sender: AccountID,
    coin: Coin,
    amount: u32
) -> bool {
    path.starts_with(source_port, source_channel) &&
    bank.balance_of(sender, path, coin) >= amount
}

#[pure]
fn send_will_transfer(
    bank: &Bank,
    path: Path,
    source_port: Port,
    source_channel: ChannelEnd,
    sender: AccountID,
    escrow_address: AccountID,
    coin: Coin,
    amount: u32
) -> bool {
    !path.starts_with(source_port, source_channel) &&
    bank.transfer_tokens_pre(sender, escrow_address, path, coin, amount)
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(path, ctx, topology))]
#[requires(
       send_will_burn(bank, path, source_port, source_channel, sender, coin, amount)
    || send_will_transfer(
            bank,
            path,
            source_port,
            source_channel,
            sender,
            ctx.escrow_address(source_channel),
            coin,
    amount)
)]
#[ensures(
    old(send_will_burn(bank, path, source_port, source_channel, sender, coin, amount)) ==>
        (bank.burn_tokens_post(old(bank), sender, path, coin, amount)
))]
#[ensures(
    old(
        send_will_transfer(
            bank,
            path,
            source_port,
            source_channel,
            sender,
            ctx.escrow_address(source_channel),
            coin,
    amount)) ==>
        bank.transfer_tokens_post(
            old(bank),
            sender,
            old(ctx.escrow_address(source_channel)),
            path,
            coin,
            amount
    )
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

#[requires(
    !packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
      bank.transfer_tokens_pre(
        ctx.escrow_address(packet.source_channel),
        packet.data.sender,
        packet.data.path,
        packet.data.coin,
        packet.data.amount
    )
)]
#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
fn on_timeout_packet(ctx: &Ctx, bank: &mut Bank, packet: Packet) {
    refund_tokens(ctx, bank, packet);
}

predicate! {
    fn refund_tokens_post(ctx: &Ctx, bank: &Bank, old_bank: &Bank, packet: Packet) -> bool {
        (!packet.data.path.starts_with(packet.source_port, packet.source_channel) ==> bank.transfer_tokens_post(
        old_bank,
        ctx.escrow_address(packet.source_channel),
        packet.data.sender,
        packet.data.path,
        packet.data.coin,
        packet.data.amount)) &&
        (packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
            bank.mint_tokens_post(
                old_bank,
                packet.data.sender,
                packet.data.path,
                packet.data.coin,
                packet.data.amount
            ))
    }
}

predicate! {
    fn refund_tokens_pre(ctx: &Ctx, bank: &Bank, packet: Packet) -> bool {
        !packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
        bank.transfer_tokens_pre(
            ctx.escrow_address(packet.source_channel),
            packet.data.sender,
            packet.data.path,
            packet.data.coin,
            packet.data.amount
        )
    }
}

#[requires(refund_tokens_pre(ctx, bank, packet))]
#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
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

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

#[pure]
fn packet_is_source(packet: Packet) -> bool {
    packet.data.path.starts_with(packet.source_port, packet.source_channel)
}

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
#[requires(packet_is_source(packet) ==>
    bank.transfer_tokens_pre(
                ctx.escrow_address(packet.dest_channel),
                packet.data.receiver,
                packet.data.path.drop_prefix(packet.source_port, packet.source_channel),
                packet.data.coin,
                packet.data.amount
            )
)]
#[requires(!packet_is_source(packet) && !packet.data.path.is_empty() ==>
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.path.head_port(),
      packet.data.path.head_channel(),
))]
#[ensures(result.success)]
#[ensures(
    !packet_is_source(packet) ==>
        bank.mint_tokens_post(
            old(bank),
            old(packet.data.receiver),
            old(packet.data.path.prepend_prefix(packet.dest_port, packet.dest_channel)),
            old(packet.data.coin),
            old(packet.data.amount)
        )
)]
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

#[ensures(
    (packet_is_source(packet)) ==> 
        bank.transfer_tokens_post(
            old(bank),
            old(ctx.escrow_address(packet.dest_channel)),
            old(packet.data.receiver),
            old(packet.data.path.tail()),
            old(packet.data.coin),
            old(packet.data.amount)
        )
)]
fn on_recv_packet(
    ctx: &Ctx,
    bank: &mut Bank,
    packet: Packet,
    topology: &Topology
) -> FungibleTokenPacketAcknowledgement {
    let FungibleTokenPacketData{ path, coin, receiver, amount, ..} = packet.data;
    let success = if packet_is_source(packet) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            receiver,
            path.drop_prefix(packet.source_port, packet.source_channel),
            coin,
            amount
        );
        true
    } else {
        bank.mint_tokens(
            receiver,
            path.prepend_prefix(packet.dest_port, packet.dest_channel),
            coin,
            amount
        )
    };

    FungibleTokenPacketAcknowledgement { success }
}

#[requires(!ack.success ==> refund_tokens_pre(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post(ctx, bank, old(bank), packet))]
#[ensures(ack.success ==> snap(bank) === old(snap(bank)))]
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

// Assume the sender's address is distinct from the escrow address for the source channel,
// and that they have sufficient funds to send to `receiver`
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]

// Assume that the sender is the source chain
#[requires(!path.starts_with(source_port, source_channel))]

// Sanity check: Neither account is escrow
#[requires(!is_escrow_account(sender))]
#[requires(!is_escrow_account(receiver))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(path, ctx1, topology))]
#[ensures(
    forall(|c: Coin|
        (PermAmount::from(bank1.unescrowed_coin_balance(c)) + PermAmount::from(bank2.unescrowed_coin_balance(c)) ==
        old(PermAmount::from(bank1.unescrowed_coin_balance(c)) + PermAmount::from(bank2.unescrowed_coin_balance(c))))
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

    prusti_assert!(
        send_will_transfer(
            bank1,
            path,
            source_port,
            source_channel,
            sender,
            ctx1.escrow_address(source_channel),
            coin,
    amount));
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

// Assume the sender's address is distinct from the escrow address for the source channel,
// and that they have sufficient funds to send to `receiver`
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]

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

#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]
#[requires(!path.starts_with(source_port, source_channel))]
// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(path, ctx1, topology))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
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
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]
#[requires(!path.starts_with(source_port, source_channel))]
#[requires(is_well_formed(path, ctx1, topology))]
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

// Assume the sender has sufficient funds to send to receiver
#[requires(bank1.balance_of(sender, path, coin) >= amount)]

// Assume the receiver is not the escrow address
#[requires(receiver != ctx2.escrow_address(dest_channel))]

// Assume that the sender is the sink chain
#[requires(path.starts_with(source_port, source_channel))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(path, ctx1, topology))]

// Assume the escrow has the corresponding locked tokens
#[requires(
    bank2.balance_of(
        ctx2.escrow_address(dest_channel),
        path.drop_prefix(source_port, source_channel),
        coin
    ) >= amount
)]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]

// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

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
