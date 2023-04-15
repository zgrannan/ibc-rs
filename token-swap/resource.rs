#![allow(dead_code, unused)]
use prusti_contracts::*;

/* 
 * This macro is used in specifications instead of Prusti's ==> syntax, for 2 reasons
 * 1. The program used to calculate syntactic complexity only supports Rust syntax for AST
 * 2. ==> cannot be used in macros, which are used in the resource specifications
 */
#[macro_export]
macro_rules! implies {
     ($lhs:expr, $rhs:expr) => {
        if $lhs { $rhs } else { true }
    }
}

pub type Amount = u32;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AccountId(u32);

#[pure]
#[trusted]
pub fn is_escrow_account(_: AccountId) -> bool {
    unimplemented!()
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrefixedCoin {
    pub denom: PrefixedDenom,
    pub amount: Amount
}

impl PrefixedCoin {
    #[pure]
    #[requires(self.denom.trace_path.starts_with(port, channel_end))]
    pub fn drop_prefix(&self, port: Port, channel_end: ChannelEnd) -> PrefixedCoin {
        PrefixedCoin { 
            denom: PrefixedDenom { 
                trace_path: self.denom.trace_path.drop_prefix(port, channel_end),
                base_denom: self.denom.base_denom
            },
            amount: self.amount
        }
    }
    #[pure]
    pub fn prepend_prefix(&self, port: Port, channel_end: ChannelEnd) -> PrefixedCoin {
        PrefixedCoin { 
            denom: PrefixedDenom { 
                trace_path: self.denom.trace_path.prepend_prefix(port, channel_end),
                base_denom: self.denom.base_denom
            },
            amount: self.amount
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrefixedDenom {
    pub trace_path: Path,
    pub base_denom: BaseDenom
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BaseDenom(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Coin(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ChannelEnd(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Port(u32);

pub struct Ctx(u32);


impl Ctx {

    #[pure]
    #[trusted]
    fn counterparty_port(&self, source_port: Port, source_channel: ChannelEnd) -> Port {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn counterparty_channel(&self, source_port: Port, source_channel: ChannelEnd) -> ChannelEnd {
        unimplemented!()
    }

    predicate! {
        pub fn has_channel(&self,
            source_port: Port, source_channel: ChannelEnd,
            dest_port: Port, dest_channel: ChannelEnd
        ) -> bool {
            self.counterparty_port(source_port, source_channel) === dest_port &&
            self.counterparty_channel(source_port, source_channel) === dest_channel
        }
    }

    #[pure]
    #[trusted]
    #[ensures(is_escrow_account(result))]
    pub fn escrow_address(&self, channel_end: ChannelEnd) -> AccountId {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FungibleTokenPacketData {
    pub denom: PrefixedDenom,
    pub sender: AccountId,
    pub receiver: AccountId,
    pub amount: u32
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Packet {
    pub source_port: Port,
    pub source_channel: ChannelEnd,
    pub dest_port: Port,
    pub dest_channel: ChannelEnd,
    pub data: FungibleTokenPacketData
}

impl Packet {
    #[pure]
    pub fn is_source(&self) -> bool {
        self.data.denom.trace_path.starts_with(self.source_port, self.source_channel)
    }

    #[pure]
    pub fn get_recv_coin(&self) -> PrefixedCoin {
        let coin = PrefixedCoin {
            denom: self.data.denom,
            amount: self.data.amount
        };
        if self.is_source() {
            coin.drop_prefix(self.source_port, self.source_channel)
        } else {
            coin.prepend_prefix(self.dest_port, self.dest_channel)
        }
    }

}

#[pure]
pub fn mk_packet(
    ctx: &Ctx,
    source_port: Port,
    source_channel: ChannelEnd,
    data: FungibleTokenPacketData
) -> Packet {
    Packet {
        source_port,
        source_channel,
        data,
        dest_port: ctx.counterparty_port(source_port, source_channel),
        dest_channel: ctx.counterparty_channel(source_port, source_channel)
    }
}

#[derive(Copy, Clone, Eq, Hash)]
pub struct Path(u32);

impl Path {

    #[pure]
    #[trusted]
    pub fn empty() -> Path {
        unimplemented!();
    }

    #[pure]
    #[trusted]
    #[ensures(result == (self === Path::empty()))]
    pub fn is_empty(self) -> bool {
        unimplemented!();
    }

    #[pure]
    #[trusted]
    #[requires(!self.is_empty())]
    pub fn head_port(self) -> Port {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[requires(!self.is_empty())]
    pub fn head_channel(self) -> ChannelEnd {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[ensures(!(result === Path::empty()))]
    #[ensures(result.starts_with(port, channel))]
    #[ensures(result.tail() === self)]
    pub fn prepend_prefix(self, port: Port, channel: ChannelEnd) -> Path {
        unimplemented!()
    }

    #[pure]
    pub fn starts_with(self, port: Port, channel: ChannelEnd) -> bool {
        !self.is_empty() &&
        port == self.head_port() &&
        channel == self.head_channel()
    }

    #[pure]
    #[requires(self.starts_with(port, channel))]
    #[ensures(result === self.tail())]
    #[ensures(result.prepend_prefix(port, channel) === self)]
    #[trusted]
    pub fn drop_prefix(self, port: Port, channel: ChannelEnd) -> Path {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn tail(self) -> Path {
       unimplemented!()
    }
}

pub struct Topology(u32);

impl Topology {

    predicate! {
        pub fn connects(
            &self,
            ctx1: &Ctx,
            port12: Port,
            channel12: ChannelEnd,
            ctx2: &Ctx,
            port21: Port,
            channel21: ChannelEnd
        ) -> bool {
            self.ctx_at(ctx1, port12, channel12) === ctx2 &&
            self.ctx_at(ctx2, port21, channel21) === ctx1 &&
            ctx1.has_channel(port12, channel12, port21, channel21) &&
            ctx2.has_channel(port21, channel21, port12, channel12)
        }
    }

    #[pure]
    #[trusted]
    pub fn ctx_at(&self, from: &Ctx, port: Port, channel: ChannelEnd) -> &Ctx {
        unimplemented!()
    }
}

/**
 * A path `P` is well-formed with respect to a chain `C` and network topology `T`
 * iff P has less than two segments, or if P has at least two segments then:
 *
 * Let P1/H1 be the port/channel pair in first segment of the path, and
 * and P2/H2 be the second segment.
 * Let C' be the chain on the end of P1/H1.
 *
 * Then, P is well-formed with respect to chain C and topology T if:
 * 1. P1/H1 and P2/H2 do not descibre a channel between C and C', and
 * 2. The tail of P (after removing P1/H1) is well-formed with respect to
 *    chain C' and topology T
 *
 * Informally, the well-formedness requirements corresponds to the path not having
 * any cycles of length 2. It shouldn't be possible to create such a path, because
 * if a transfer C -> C' adds an additional segment to the path, the subsequent
 * transfer C' -> C should remove it. However, this well-formedness property does
 * not rule out longer cycles, i.e., C1 -> C2 -> C3 -> C1; it is possible to create paths
 * forming such cycles in the protocol.
 */
predicate! {
    pub fn is_well_formed(path: Path, ctx: &Ctx, topology: &Topology) -> bool {
        path.is_empty() || path.tail().is_empty() || {
            let path_tail = path.tail();
            let port1 = path.head_port();
            let channel1 = path.head_channel();
            let port2 = path_tail.head_port();
            let channel2 = path_tail.head_channel();
            let ctx2 = topology.ctx_at(ctx, port1, channel1);
            !ctx.has_channel(
                port1,
                channel1,
                port2,
                channel2,
            ) && is_well_formed(path_tail, ctx2, topology)
        }
    }
}


// Prusti does not like derived PartialEQ
impl PartialEq for Path {
    #[trusted]
    #[pure]
    fn eq(&self, other: &Path) -> bool {
        unimplemented!()
    }

}

pub struct FungibleTokenPacketAcknowledgement {
    pub success: bool
}

// PROPSPEC_START INVARIANT
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
#[invariant(
    forall(|acct_id: AccountId, denom: PrefixedDenom|
        PermAmount::from(self.balance_of(acct_id, denom)) >=
        holds(Money(self.id(), acct_id, denom)) 
    ))]
// PROPSPEC_STOP
pub struct BankKeeper(u32);

#[derive(Copy, Clone)]
pub struct BankID(u32);

// PROPSPEC_START TYPE
#[resource_kind]
pub struct Money(pub BankID, pub AccountId, pub PrefixedDenom);

#[resource_kind]
pub struct UnescrowedCoins(pub BankID, pub BaseDenom);
// PROPSPEC_STOP TYPE

// PROPSPEC_START RESOURCE_OP

#[macro_export]
macro_rules! transfer_money {
    ($bank_id:expr, $to:expr, $coin:expr) => {
    resource(Money($bank_id, $to, $coin.denom), $coin.amount) && 
        implies!( 
            !is_escrow_account($to),
            resource(UnescrowedCoins($bank_id, $coin.denom.base_denom), $coin.amount)
        )
    }
}
//PROPSPEC_STOP


impl BankKeeper {

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


    #[requires(from != to)]
    //PROPSPEC_START RESOURCE_OP
    //SEND_RESOURCE_SPEC_START
    #[requires(transfer_money!(self.id(), from, coin))]
    #[ensures(transfer_money!(self.id(), to, coin))]
    //SEND_RESOURCE_SPEC_END
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
    //PROPSPEC_START RESOURCE_OP
    //BURN_RESOURCE_SPEC_START
    #[requires(transfer_money!(self.id(), from, coin))]
    //PROPSPEC_STOP
    //BURN_RESOURCE_SPEC_END
    pub fn burn_tokens(&mut self, from: AccountId, coin: &PrefixedCoin) {
        unimplemented!()
    }

    #[trusted]
    //PROPSPEC_START RESOURCE_OP
    //MINT_RESOURCE_SPEC_START
    #[ensures(transfer_money!(self.id(), to, coin))]
    //MINT_RESOURCE_SPEC_END
    //PROPSPEC_STOP
    pub fn mint_tokens(&mut self, to: AccountId, coin: &PrefixedCoin) {
        unimplemented!()
    }
}

// Sanity check: The sender cannot be an escrow account
#[requires(!is_escrow_account(sender))]
#[requires(is_well_formed(coin.denom.trace_path, ctx, topology))]
//PROPSPEC_START RESOURCE_OP
// SEND_FUNGIBLE_TOKENS_RESOURCE_SPEC_START
#[requires(transfer_money!(bank.id(), sender, coin))]
#[ensures(implies!(!coin.denom.trace_path.starts_with(source_port, source_channel), 
    transfer_money!(bank.id(), ctx.escrow_address(source_channel), coin)))]
// SEND_FUNGIBLE_TOKENS_RESOURCE_SPEC_END
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
    bank: &mut BankKeeper,
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


//PROPSPEC_START RESOURCE_OP
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
fn refund_tokens(ctx: &Ctx, bank: &mut BankKeeper, packet: &Packet) {
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

//PROPSPEC_START RESOURCE_OP
#[requires(refund_tokens_pre!(ctx, bank, packet))]
#[ensures(refund_tokens_post!(bank, packet))]
//PROPSPEC_STOP
pub fn on_timeout_packet(ctx: &Ctx, bank: &mut BankKeeper, packet: &Packet) {
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
//PROPSPEC_START RESOURCE_OP
// ON_RECV_PACKET_RESOURCE_SPEC_START
#[requires(implies!(packet.is_source(), transfer_money!(
    bank.id(),
    ctx.escrow_address(packet.dest_channel), 
    packet.get_recv_coin()
)))]
#[ensures(transfer_money!(bank.id(), packet.data.receiver, packet.get_recv_coin()))]
// ON_RECV_PACKET_RESOURCE_SPEC_END
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
    bank: &mut BankKeeper, 
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

// PROPSPEC_START RESOURCE_OP
#[requires(!ack.success ==> refund_tokens_pre!(ctx, bank, packet))]
#[ensures(!ack.success ==> refund_tokens_post!(bank, packet))]
// PROPSPEC_STOP
pub fn on_acknowledge_packet(
    ctx: &Ctx,
    bank: &mut BankKeeper,
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
}fn main() {
}
