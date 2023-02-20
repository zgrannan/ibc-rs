#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;

// PROPSPEC_START
#[invariant_twostate(self.id() === old(self.id()))]
#[invariant_twostate(
    forall(|acct_id: AccountId, denom: PrefixedDenom|
        holds(Money(self.id(), acct_id, denom)) - 
        old(holds(Money(self.id(), acct_id, denom))) ==
        PermAmount::from(self.balance_of(acct_id, denom)) - 
            PermAmount::from(old(self.balance_of(acct_id, denom)))
    ))
]
#[invariant_twostate(
    forall(|coin: BaseDenom|
        holds(UnescrowedCoins(self.id(), coin)) - 
        old(holds(UnescrowedCoins(self.id(), coin))) ==
        PermAmount::from(self.unescrowed_coin_balance(coin)) - 
            PermAmount::from(old(self.unescrowed_coin_balance(coin)))
    ))
]
#[invariant_twostate(
    forall(|acct_id: AccountId, denom: PrefixedDenom|
        PermAmount::from(self.balance_of(acct_id, denom)) >=
        holds(Money(self.id(), acct_id, denom)) 
    ))]
// PROPSPEC_STOP
pub struct Bank(u32);

// PROPSPEC_START
#[derive(Copy, Clone)]
pub struct BankID(u32);

#[resource]
pub struct Money(pub BankID, pub AccountId, pub PrefixedDenom);

#[resource]
pub struct UnescrowedCoins(pub BankID, pub BaseDenom);

#[macro_export]
macro_rules! implies {
     ($lhs:expr, $rhs:expr) => {
        if $lhs { $rhs } else { true }
    }
}

#[macro_export]
macro_rules! transfer_money {
    ($bank_id:expr, $to:expr, $coin:expr) => {
    transfers(Money($bank_id, $to, $coin.denom), $coin.amount) && 
        implies!( 
            !is_escrow_account($to),
            transfers(UnescrowedCoins($bank_id, $coin.denom.base_denom), $coin.amount)
        )
    }
}
//PROPSPEC_STOP


impl Bank {

    #[pure]
    #[trusted]
    pub fn id(&self) -> BankID {
        unimplemented!()
    }

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


    //PROPSPEC_START
    #[requires(from != to)]
    #[requires(transfer_money!(self.id(), from, coin))]
    #[ensures(transfer_money!(self.id(), to, coin))]
    //PROPSPEC_STOP
    pub fn transfer_tokens(
        &mut self,
        from: AccountId,
        to: AccountId,
        coin: &PrefixedCoin
    ) {
        self.burn_tokens(from, coin);
        self.mint_tokens(to, coin);
    }


    #[trusted]
    //PROPSPEC_START
    #[requires(transfer_money!(self.id(), from, coin))]
    #[requires(self.balance_of(from, coin.denom) >= coin.amount)]
    //PROPSPEC_STOP
    pub fn burn_tokens(&mut self, from: AccountId, coin: &PrefixedCoin) {
        unimplemented!()
    }

    #[trusted]
    //PROPSPEC_START
    #[ensures(transfer_money!(self.id(), to, coin))]
    //PROPSPEC_STOP
    pub fn mint_tokens(&mut self, to: AccountId, coin: &PrefixedCoin) {
        unimplemented!()
    }
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx, topology))]
//PROPSPEC_START
#[requires(transfer_money!(bank.id(), sender, coin))]
#[ensures(!coin.denom.trace_path.starts_with(source_port, source_channel)
    ==> transfer_money!(bank.id(), ctx.escrow_address(source_channel), coin))]
//PROPSPEC_STOP
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
    bank: &mut Bank,
    coin: &PrefixedCoin,
    sender: AccountId,
    receiver: AccountId,
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


//PROPSPEC_START
macro_rules! refund_tokens_pre {
    ($ctx:expr, $bank:expr, $packet:expr) => { implies!(
        !$packet.data.denom.trace_path.starts_with($packet.source_port, $packet.source_channel),
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
//PROPSPEC_STOP
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

//PROPSPEC_START
#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
//PROPSPEC_STOP
pub fn on_timeout_packet(ctx: &Ctx, bank: &mut Bank, packet: &Packet) {
    refund_tokens(ctx, bank, packet);
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
#[requires(!packet.is_source() && !packet.data.denom.trace_path.is_empty() ==> 
    !ctx.has_channel(
      packet.dest_port,
      packet.dest_channel,
      packet.data.denom.trace_path.head_port(),
      packet.data.denom.trace_path.head_channel(),
))]
//PROPSPEC_START
#[requires(packet.is_source() ==> transfer_money!(
    bank.id(),
    ctx.escrow_address(packet.dest_channel), 
    packet.get_recv_coin()
))]
#[ensures(transfer_money!(bank.id(), packet.data.receiver, packet.get_recv_coin()))]
//PROPSPEC_STOP
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
#[ensures(result.success)]
pub fn on_recv_packet(
    ctx: &Ctx, 
    bank: &mut Bank, 
    packet: &Packet,
    topology: &Topology
) -> FungibleTokenPacketAcknowledgement {
    let coin = packet.get_recv_coin();
    if packet.is_source() {
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            packet.data.receiver,
            &coin
        );
    } else {
        bank.mint_tokens(packet.data.receiver, &coin);
    };
    FungibleTokenPacketAcknowledgement { success: true }
}

// PROPSPEC_START
#[requires(!ack.success ==> refund_tokens_pre!(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post!(bank, packet))]
// PROPSPEC_STOP
pub fn on_acknowledge_packet(
    ctx: &Ctx,
    bank: &mut Bank,
    ack: FungibleTokenPacketAcknowledgement,
    packet: &Packet) {
    if(!ack.success) {
        refund_tokens(ctx, bank, packet);
    }
}
