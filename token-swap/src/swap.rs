#![allow(dead_code, unused)]
use std::path::Prefix;

use prusti_contracts::*;

use crate::types::*;
struct Bank(u32);

impl Bank {

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

    predicate! {
        fn transfer_tokens_post(
            &self,
            old_bank: &Self,
            from: AccountID,
            to: AccountID,
            coin: &PrefixedCoin,
        ) -> bool {
        ((is_escrow_account(to) && !is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) - coin.amount) &&
        ((!is_escrow_account(to) && is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) + coin.amount) &&
        ((is_escrow_account(to) == is_escrow_account(from)) ==>
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom)) &&
        forall(|acct_id2: AccountID, denom2: PrefixedDenom|
            self.balance_of(acct_id2, denom2) ==
                if(acct_id2 == from && coin.denom == denom2) {
                    old_bank.balance_of(from, coin.denom) - coin.amount
                } else if (acct_id2 == to && coin.denom == denom2){
                    old_bank.balance_of(to, coin.denom) + coin.amount
                } else {
                    old_bank.balance_of(acct_id2, denom2)
                }
        ) && forall(|c: BaseDenom| c != coin.denom.base_denom ==> 
            self.unescrowed_coin_balance(c) == old_bank.unescrowed_coin_balance(c)
        )
        }
    }

    #[pure]
    fn transfer_tokens_pre(
        &self,
        from: AccountID,
        to: AccountID,
        coin: &PrefixedCoin,
    ) -> bool {
        from != to && self.balance_of(from, coin.denom) >= coin.amount
    }

    #[requires(self.transfer_tokens_pre(from, to, coin))]
    #[ensures(self.transfer_tokens_post(old(self), from, to, coin))]
    fn transfer_tokens(
        &mut self,
        from: AccountID,
        to: AccountID,
        coin: &PrefixedCoin
    ) {
        self.burn_tokens(from, coin);
        self.mint_tokens(to, coin);
    }

    predicate! {
        fn burn_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            coin: &PrefixedCoin
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom)
            } else {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) - coin.amount
            } && forall(|acct_id2: AccountID, denom: PrefixedDenom|
            self.balance_of(acct_id2, denom) ==
                if(acct_id == acct_id2 && coin.denom == denom) {
                    old_bank.balance_of(acct_id, denom) - coin.amount
                } else {
                    old_bank.balance_of(acct_id2, denom)
                }
            ) && forall(|d: BaseDenom| d != coin.denom.base_denom ==> 
                self.unescrowed_coin_balance(d) == old_bank.unescrowed_coin_balance(d)
            )
        }
    }

    #[requires(self.balance_of(to, coin.denom) >= coin.amount)]
    #[ensures(self.burn_tokens_post(old(self), to, coin))]
    #[trusted]
    fn burn_tokens(&mut self, to: AccountID, coin: &PrefixedCoin) {
        unimplemented!()
    }

    predicate! {
        fn mint_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            coin: &PrefixedCoin
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom)
            } else {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) + coin.amount
            } && forall(|acct_id2: AccountID, denom: PrefixedDenom|
            self.balance_of(acct_id2, denom) ==
                if(acct_id == acct_id2 && coin.denom == denom) {
                    old_bank.balance_of(acct_id, denom) + coin.amount
                } else {
                    old_bank.balance_of(acct_id2, denom)
                }
            ) && forall(|d: BaseDenom| d != coin.denom.base_denom ==> 
                self.unescrowed_coin_balance(d) == old_bank.unescrowed_coin_balance(d)
            )
        }
    }

    #[ensures(result)]
    #[ensures(self.mint_tokens_post(old(self), to, coin))]
    #[trusted]
    fn mint_tokens(&mut self, to: AccountID, coin: &PrefixedCoin) -> bool {
        unimplemented!()
    }
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx, topology))]
#[requires(bank.balance_of(sender, coin.denom) >= coin.amount)]
#[ensures(
   if !coin.denom.trace_path.starts_with(source_port, source_channel) {
        bank.transfer_tokens_post(
            old(bank),
            sender,
            ctx.escrow_address(source_channel),
            coin
        )
   } else {
        bank.burn_tokens_post(old(bank), sender, coin)
   }
)]
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

#[requires(refund_tokens_pre(ctx, bank, packet))]
#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
fn on_timeout_packet(ctx: &Ctx, bank: &mut Bank, packet: &Packet) {
    refund_tokens(ctx, bank, packet);
}

predicate! {
    fn refund_tokens_post(ctx: &Ctx, bank: &Bank, old_bank: &Bank, packet: &Packet) -> bool {
        let coin = &PrefixedCoin { denom: packet.data.denom, amount: packet.data.amount };
        if !packet.data.denom.trace_path.starts_with(packet.source_port, packet.source_channel) {  
            bank.transfer_tokens_post(
                old_bank,
                ctx.escrow_address(packet.source_channel),
                packet.data.sender,
                coin
            ) 
        } else {
            bank.mint_tokens_post(
                old_bank,
                packet.data.sender,
                coin
            )
        }
    }
}

predicate! {
    fn refund_tokens_pre(ctx: &Ctx, bank: &Bank, packet: &Packet) -> bool {
        !packet.data.denom.trace_path.starts_with(packet.source_port, packet.source_channel) ==>
        bank.transfer_tokens_pre(
            ctx.escrow_address(packet.source_channel),
            packet.data.sender,
            &PrefixedCoin { denom: packet.data.denom, amount: packet.data.amount }
        )
    }
}

#[requires(refund_tokens_pre(ctx, bank, packet))]
#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
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

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

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
#[requires(packet.is_source() ==>
    bank.transfer_tokens_pre(
        ctx.escrow_address(packet.dest_channel),
        packet.data.receiver,
        &PrefixedCoin {
            denom: packet.data.denom,
            amount: packet.data.amount
        }.drop_prefix(packet.source_port, packet.source_channel)
    )
)]
#[requires(!packet.is_source() && !packet.data.denom.trace_path.is_empty() ==>
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.denom.trace_path.head_port(),
      packet.data.denom.trace_path.head_channel(),
))]
#[ensures(result.success)]
#[ensures(
    if packet.is_source() {
        bank.transfer_tokens_post(
            old(bank),
            ctx.escrow_address(packet.dest_channel),
            packet.data.receiver,
            &PrefixedCoin {
                denom: packet.data.denom,
                amount: packet.data.amount
            }.drop_prefix(packet.source_port, packet.source_channel)
        )
    } else {
        bank.mint_tokens_post(
            old(bank),
            packet.data.receiver,
            &PrefixedCoin {
                denom: packet.data.denom,
                amount: packet.data.amount
            }.prepend_prefix(packet.dest_port, packet.dest_channel)
        )
    }

)]
#[ensures(
    !packet.is_source() ==>
        is_well_formed(
            packet.data.denom.trace_path.prepend_prefix(
                packet.dest_port, 
                packet.dest_channel
            ),
            ctx,
            topology)
)]
fn on_recv_packet(
    ctx: &Ctx,
    bank: &mut Bank,
    packet: &Packet,
    topology: &Topology
) -> FungibleTokenPacketAcknowledgement {
    let FungibleTokenPacketData{ denom, receiver, amount, ..} = packet.data;
    if packet.is_source() {
        let coin = PrefixedCoin { denom, amount }
            .drop_prefix(packet.source_port, packet.source_channel);
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            receiver,
            &coin
        );
    } else {
        let coin = PrefixedCoin { denom, amount }
            .prepend_prefix(packet.dest_port, packet.dest_channel);
        bank.mint_tokens(receiver, &coin);
    };
    FungibleTokenPacketAcknowledgement { success: true }
}

#[requires(!ack.success ==> refund_tokens_pre(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post(ctx, bank, old(bank), packet))]
#[ensures(ack.success ==> snap(bank) === old(snap(bank)))]
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

// Assume the sender's address is distinct from the escrow address for the source channel,
// and that they have sufficient funds to send to `receiver`
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), coin))
]

// Assume that the sender is the source chain
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]

// Sanity check: Neither account is escrow
#[requires(!is_escrow_account(sender))]
#[requires(!is_escrow_account(receiver))]

#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
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

// Assume the sender's address is distinct from the escrow address for the source channel,
// and that they have sufficient funds to send to `receiver`
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), coin))
]

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

#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), coin))
]
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]
// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
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
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), coin))
]
#[requires(!coin.denom.trace_path.starts_with(source_port, source_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]
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

// Assume the sender has sufficient funds to send to receiver
#[requires(bank1.balance_of(sender, coin.denom) >= coin.amount)]

// Assume the receiver is not the escrow address
#[requires(receiver != ctx2.escrow_address(dest_channel))]

// Assume that the sender is the sink chain
#[requires(coin.denom.trace_path.starts_with(source_port, source_channel))]

// Assume the path is well-formed.
// See the definition of `is_well_formed` for details
#[requires(topology.connects(ctx1, source_port, source_channel, ctx2, dest_port, dest_channel))]
#[requires(is_well_formed(coin.denom.trace_path, ctx1, topology))]

// Assume the escrow has the corresponding locked tokens
#[requires(
    bank2.balance_of(
        ctx2.escrow_address(dest_channel),
        coin.drop_prefix(source_port, source_channel).denom,
    ) >= coin.amount
)]

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]

// Sanity check: Because this is a round-trip, the receiver cannot be an escrow
// account
#[requires(!is_escrow_account(receiver))]

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