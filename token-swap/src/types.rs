#![allow(dead_code, unused)]
use prusti_contracts::*;

pub type Amount = u32;


#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AccountID(u32);

#[pure]
#[trusted]
pub fn is_escrow_account(_: AccountID) -> bool {
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
    pub fn escrow_address(&self, channel_end: ChannelEnd) -> AccountID {
        unimplemented!()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct FungibleTokenPacketData {
    pub denom: PrefixedDenom,
    pub sender: AccountID,
    pub receiver: AccountID,
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

#[pure]
pub fn packet_is_source(packet: &Packet) -> bool {
    packet.data.denom.trace_path.starts_with(packet.source_port, packet.source_channel)
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
