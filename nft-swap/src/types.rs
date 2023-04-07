#![allow(dead_code, unused)]
use prusti_contracts::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct BaseClassId(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ClassUri(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ClassData(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenId(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenUri(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenData(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct AccountId(u32);

#[pure]
#[trusted]
pub fn is_escrow_account(_: AccountId) -> bool {
    unimplemented!()
}


#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PrefixedClassId {
    pub path: Path,
    pub base: BaseClassId
}

impl PrefixedClassId {
    #[pure]
    #[requires(self.path.starts_with(port, channel_end))]
    pub fn drop_prefix(&self, port: Port, channel_end: ChannelEnd) -> PrefixedClassId {
        PrefixedClassId {
            base: self.base,
            path: self.path.drop_prefix(port, channel_end)
        }
    }
    #[pure]
    pub fn prepend_prefix(&self, port: Port, channel_end: ChannelEnd) -> PrefixedClassId {
        PrefixedClassId {
            base: self.base,
            path: self.path.prepend_prefix(port, channel_end)
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ChannelEnd(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Port(u32);

pub struct Ctx(u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenIdVec(u32);

impl TokenIdVec {

    #[pure]
    #[trusted]
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[requires(i < self.len())]
    pub fn get(&self, i: usize) -> TokenId {
        unimplemented!()
    }

}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenUriVec(u32);

impl TokenUriVec {

    #[pure]
    #[trusted]
    pub fn new() -> TokenUriVec {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[requires(i < self.len())]
    pub fn get(&self, i: usize) -> TokenUri {
        unimplemented!()
    }

}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TokenDataVec(u32);

impl TokenDataVec {

    #[pure]
    #[trusted]
    pub fn new() -> TokenDataVec {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn len(&self) -> usize {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    #[requires(i < self.len())]
    pub fn get(&self, i: usize) -> TokenData {
        unimplemented!()
    }

}


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
pub struct Class {
    pub uri: ClassUri,
    pub data: ClassData,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NFT {
    pub uri: TokenUri,
    pub data: TokenData,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct NFTPacketData {
    pub class_id: PrefixedClassId,
    pub class_uri: ClassUri,
    pub class_data: ClassData,
    pub token_ids: TokenIdVec,
    pub token_data: TokenDataVec,
    pub token_uris: TokenUriVec,
    pub sender: AccountId,
    pub receiver: AccountId,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Packet {
    pub source_port: Port,
    pub source_channel: ChannelEnd,
    pub dest_port: Port,
    pub dest_channel: ChannelEnd,
    pub data: NFTPacketData
}

impl Packet {
    #[pure]
    pub fn is_source(&self) -> bool {
        self.data.class_id.path.starts_with(self.source_port, self.source_channel)
    }

    #[pure]
    pub fn get_recv_class_id(&self) -> PrefixedClassId {
        if self.is_source() {
            self.data.class_id.drop_prefix(self.source_port, self.source_channel)
        } else {
            self.data.class_id.prepend_prefix(self.dest_port, self.dest_channel)
        }
    }

}

#[pure]
pub fn mk_packet(
    ctx: &Ctx,
    source_port: Port,
    source_channel: ChannelEnd,
    data: NFTPacketData
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

#[derive(Clone, Copy)]
pub struct NFTPacketAcknowledgement {
    pub success: bool
}
