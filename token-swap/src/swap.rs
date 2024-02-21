#![allow(dead_code, unused)]
use std::path::Prefix;

use prusti_contracts::*;

use crate::implies;
use crate::types::*;
pub struct BankKeeper(u32);

impl BankKeeper {
    #[pure]
    #[trusted]
    pub fn unescrowed_coin_balance(&self, coin: BaseDenom) -> Amount {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn balance_of(&self, acct_id: AccountId, denom: PrefixedDenom) -> Amount {
        unimplemented!()
    }

    // SEND_SPEC_START
    predicate! {
        pub fn transfer_tokens_post(
            &self,
            old_bank: &Self,
            from: AccountId,
            to: AccountId,
            coin: &PrefixedCoin
        ) -> bool {
            self.unescrowed_coin_balance(coin.denom.base_denom) ==
                if (is_escrow_account(to) && !is_escrow_account(from)) {
                    old_bank.unescrowed_coin_balance(coin.denom.base_denom) - coin.amount
                } else if (!is_escrow_account(to) && is_escrow_account(from)) {
                    old_bank.unescrowed_coin_balance(coin.denom.base_denom) + coin.amount
                } else {
                    old_bank.unescrowed_coin_balance(coin.denom.base_denom)
                } &&
            forall(|acct_id2: AccountId, denom2: PrefixedDenom|
                self.balance_of(acct_id2, denom2) ==
                    if(acct_id2 == from && coin.denom == denom2) {
                        old_bank.balance_of(from, coin.denom) - coin.amount
                    } else if (acct_id2 == to && coin.denom == denom2){
                        old_bank.balance_of(to, coin.denom) + coin.amount
                    } else {
                        old_bank.balance_of(acct_id2, denom2)
                    }
            ) && forall(|c: BaseDenom| implies!(c != coin.denom.base_denom,
                self.unescrowed_coin_balance(c) == old_bank.unescrowed_coin_balance(c)
                )
            )
        }
    }
    // SEND_SPEC_END

    #[pure]
    // SEND_SPEC_START
    pub fn transfer_tokens_pre(&self, from: AccountId, to: AccountId, coin: &PrefixedCoin) -> bool {
        from != to && self.balance_of(from, coin.denom) >= coin.amount
    }
    // SEND_SPEC_END

    // SEND_SPEC_START
    #[requires(self.transfer_tokens_pre(from, to, coin))]
    #[ensures(self.transfer_tokens_post(old(self), from, to, coin))]
    // SEND_SPEC_END
    fn transfer_tokens(&mut self, from: AccountId, to: AccountId, coin: &PrefixedCoin) {
        self.burn_tokens(from, coin);
        self.mint_tokens(to, coin);
    }

    // BURN_SPEC_START
    predicate! {
        fn burn_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountId,
            coin: &PrefixedCoin
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom)
            } else {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) - coin.amount
            } && forall(|acct_id2: AccountId, denom: PrefixedDenom|
            self.balance_of(acct_id2, denom) ==
                if(acct_id == acct_id2 && coin.denom == denom) {
                    old_bank.balance_of(acct_id, denom) - coin.amount
                } else {
                    old_bank.balance_of(acct_id2, denom)
                }
            ) && forall(|d: BaseDenom|
                implies!(d != coin.denom.base_denom, self.unescrowed_coin_balance(d) == old_bank.unescrowed_coin_balance(d))
            )
        }
    }
    // BURN_SPEC_END

    // BURN_SPEC_START
    #[requires(self.balance_of(to, coin.denom) >= coin.amount)]
    #[ensures(self.burn_tokens_post(old(self), to, coin))]
    // BURN_SPEC_END
    #[trusted]
    fn burn_tokens(&mut self, to: AccountId, coin: &PrefixedCoin) {
        unimplemented!()
    }

    // MINT_SPEC_START
    predicate! {
        fn mint_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountId,
            coin: &PrefixedCoin
        ) -> bool {
            if is_escrow_account(acct_id) {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom)
            } else {
              self.unescrowed_coin_balance(coin.denom.base_denom) ==
                old_bank.unescrowed_coin_balance(coin.denom.base_denom) + coin.amount
            } && forall(|acct_id2: AccountId, denom: PrefixedDenom|
            self.balance_of(acct_id2, denom) ==
                if(acct_id == acct_id2 && coin.denom == denom) {
                    old_bank.balance_of(acct_id, denom) + coin.amount
                } else {
                    old_bank.balance_of(acct_id2, denom)
                }
            ) && forall(|d: BaseDenom| implies!(d != coin.denom.base_denom,
                self.unescrowed_coin_balance(d) == old_bank.unescrowed_coin_balance(d))
            )
        }
    }
    // MINT_SPEC_END

    // MINT_SPEC_START
    #[ensures(self.mint_tokens_post(old(self), to, coin))]
    // MINT_SPEC_END
    #[ensures(result)]
    #[trusted]
    fn mint_tokens(&mut self, to: AccountId, coin: &PrefixedCoin) -> bool {
        unimplemented!()
    }
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx, topology))]
// PROPSPEC_START
// SEND_FUNGIBLE_TOKENS_SPEC_START
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
// SEND_FUNGIBLE_TOKENS_SPEC_END
// PROPSPEC_STOP
#[ensures(
    result == mk_packet(
        ctx,
        source_port,
        source_channel,
        FungibleTokenPacketData {denom: coin.denom, sender, receiver, amount: coin.amount}
    )
)]
pub fn send_fungible_tokens(
    ctx: &Ctx,
    bank: &mut BankKeeper,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology,
) -> Packet {
    if !coin
        .denom
        .trace_path
        .starts_with(source_port, source_channel)
    {
        bank.transfer_tokens(sender, ctx.escrow_address(source_channel), coin);
    } else {
        bank.burn_tokens(sender, coin);
    };

    let data = FungibleTokenPacketData {
        denom: coin.denom,
        sender,
        receiver,
        amount: coin.amount,
    };
    mk_packet(ctx, source_port, source_channel, data)
}

// PROPSPEC_START
#[requires(refund_tokens_pre(ctx, bank, packet))]
#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
// PROPSPEC_STOP
pub fn on_timeout_packet(ctx: &Ctx, bank: &mut BankKeeper, packet: &Packet) {
    refund_tokens(ctx, bank, packet);
}

// PROPSPEC_START
predicate! {
    fn refund_tokens_post(ctx: &Ctx, bank: &BankKeeper, old_bank: &BankKeeper, packet: &Packet) -> bool {
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
    fn refund_tokens_pre(ctx: &Ctx, bank: &BankKeeper, packet: &Packet) -> bool {
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
// PROPSPEC_STOP
fn refund_tokens(ctx: &Ctx, bank: &mut BankKeeper, packet: &Packet) {
    let FungibleTokenPacketData {
        denom,
        sender,
        amount,
        ..
    } = packet.data;
    if !denom
        .trace_path
        .starts_with(packet.source_port, packet.source_channel)
    {
        bank.transfer_tokens(
            ctx.escrow_address(packet.source_channel),
            sender,
            &PrefixedCoin { denom, amount },
        );
    } else {
        bank.mint_tokens(sender, &PrefixedCoin { denom, amount });
    }
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
#[requires(!packet.is_source() && !packet.data.denom.trace_path.is_empty() ==>
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.denom.trace_path.head_port(),
      packet.data.denom.trace_path.head_channel(),
))]
// PROPSPEC_START
// ON_RECV_PACKET_SPEC_START
#[requires(implies!(packet.is_source(),
    bank.transfer_tokens_pre(
        ctx.escrow_address(packet.dest_channel),
        packet.data.receiver,
        &packet.get_recv_coin()
    )
))]
#[ensures(
    if packet.is_source() {
        bank.transfer_tokens_post(
            old(bank),
            ctx.escrow_address(packet.dest_channel),
            packet.data.receiver,
            &packet.get_recv_coin()
        )
    } else {
        bank.mint_tokens_post(old(bank), packet.data.receiver, &packet.get_recv_coin())
    }
)]
// ON_RECV_PACKET_SPEC_END
// PROPSPEC_STOP
#[ensures(result.success)]
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
pub fn on_recv_packet(
    ctx: &Ctx,
    bank: &mut BankKeeper,
    packet: &Packet,
    topology: &Topology,
) -> FungibleTokenPacketAcknowledgement {
    let coin = packet.get_recv_coin();
    if packet.is_source() {
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            packet.data.receiver,
            &coin,
        );
    } else {
        bank.mint_tokens(packet.data.receiver, &coin);
    };
    FungibleTokenPacketAcknowledgement { success: true }
}

// PROPSPEC_START
#[requires(!ack.success ==> refund_tokens_pre(ctx, bank, packet))]
#[ensures(
    if !ack.success {
        refund_tokens_post(ctx, bank, old(bank), packet)
    } else {
        snap(bank) === old(snap(bank))
    }
)]
// PROPSPEC_STOP
pub fn on_acknowledge_packet(
    ctx: &Ctx,
    bank: &mut BankKeeper,
    ack: FungibleTokenPacketAcknowledgement,
    packet: &Packet,
) {
    if (!ack.success) {
        refund_tokens(ctx, bank, packet);
    }
}
