#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;


#[resource_kind]
pub struct Token(pub KeeperId, pub PrefixedClassId, pub TokenId);

#[derive(Copy, Clone)]
pub struct KeeperId(u32);

#[macro_export]
macro_rules! transfers_token {
    ($nft:expr, $class_id:expr, $token_id:expr) => {
        resource(Token($nft.id(), $class_id, $token_id), 1)
    }
}

macro_rules! implies {
    ($lhs:expr, $rhs:expr) => {
        if $lhs { $rhs } else { true }
    };
}

#[invariant_twostate(
    forall( |class_id: PrefixedClassId, token_id: TokenId|
    ( old(holds(Token(self.id(), class_id, token_id))) == PermAmount::from(0) 
    &&    holds(Token(self.id(), class_id, token_id)) == PermAmount::from(0)) ==>
        self.get_owner(class_id, token_id) == old(self.get_owner(class_id, token_id))
        // triggers = [(self.get_owner(class_id, token_id))]
))]
#[invariant_twostate(self.id() === old(self.id()))]
pub struct NFTKeeper(u32);

impl NFTKeeper {

    #[pure]
    #[trusted]
    pub fn id(&self) -> KeeperId {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn get_owner(&self, class_id: PrefixedClassId, token_id: TokenId) -> Option<AccountId> {
        unimplemented!()
    }

    #[trusted]
    pub fn create_or_update_class(
        &mut self,
        class_id: PrefixedClassId,
        class_uri: ClassUri,
        class_data: ClassData) {
        unimplemented!()
    }

    #[trusted]
    #[requires(self.get_owner(class_id, token_id) == None)]
    #[ensures(transfers_token!(self, class_id, token_id))]
    #[ensures(self.get_owner(class_id, token_id) == Some(receiver))]
    pub fn mint(&mut self,
                class_id: PrefixedClassId,
                token_id: TokenId,
                token_uri: TokenUri,
                token_data: TokenData,
                receiver: AccountId) {
        unimplemented!()
    }

    #[trusted]
    #[requires(transfers_token!(self, class_id, token_id))]
    #[ensures(transfers_token!(self, class_id, token_id))]
    #[ensures(self.get_owner(class_id, token_id) == Some(receiver))]
    pub fn transfer(&mut self,
                    class_id: PrefixedClassId,
                    token_id: TokenId,
                    receiver: AccountId,
                    token_data: Option<TokenData>) {
        unimplemented!()
    }

    #[requires(transfers_token!(self, class_id, token_id))]
    #[ensures(self.get_owner(class_id, token_id) == None)]
    #[trusted]
    pub fn burn(&mut self, class_id: PrefixedClassId, token_id: TokenId) {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn get_nft(&self, class_id: PrefixedClassId, token_id: TokenId) -> NFT {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    pub fn get_class(&self, class_id: PrefixedClassId) -> Class {
        unimplemented!()
    }
}


#[pure]
fn make_packet_data(nft: &NFTKeeper, class_id: PrefixedClassId, token_ids: TokenIdVec, sender: AccountId, receiver: AccountId) -> NFTPacketData {
    NFTPacketData {
        class_id,
        class_data: nft.get_class(class_id).data,
        class_uri: nft.get_class(class_id).uri,
        token_ids,
        token_data: TokenDataVec::new(),
        token_uris: TokenUriVec::new(),
        sender,
        receiver
    }
}

#[requires(
    forall(|i : usize| i < token_ids.len() ==>
        nft.get_owner(class_id, token_ids.get(i)) == Some(sender)
))]
#[requires(
    forall(|i : usize, j: usize| i < token_ids.len() && j < i ==>
        token_ids.get(i) != token_ids.get(j)
))]
#[requires(
    forall(|i : usize| i < token_ids.len() ==>
        transfers_token!(nft, class_id, token_ids.get(i)))
)]
// It's a transfer to escrow locally, retain permission
#[ensures(old(!class_id.path.starts_with(source_port, source_channel)) ==>
    forall(|i : usize| i < token_ids.len() ==>
        transfers_token!(nft, class_id, token_ids.get(i)))
)]
#[ensures(
    forall(|i : usize| i < token_ids.len() ==>
        if old(class_id.path.starts_with(source_port, source_channel)) {
            nft.get_owner(class_id, token_ids.get(i)) == None
        } else {
            nft.get_owner(class_id, token_ids.get(i)) == Some(ctx.escrow_address(source_channel))
        }
    )
)]
#[ensures(
    result == mk_packet(
        ctx,
        source_port,
        source_channel,
        make_packet_data(nft, class_id, token_ids, sender, receiver)
    )
)]
pub fn send_nft(
    ctx: &Ctx,
    nft: &mut NFTKeeper,
    class_id: PrefixedClassId,
    token_ids: TokenIdVec,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology
) -> Packet {
    let mut i = 0;
    while i < token_ids.len() {
        body_invariant!(forall( |class_id: PrefixedClassId, token_id: TokenId|
        ( old(holds(Token(nft.id(), class_id, token_id))) == PermAmount::from(0) 
        &&    holds(Token(nft.id(), class_id, token_id)) == PermAmount::from(0)) ==>
            nft.get_owner(class_id, token_id) == old(nft.get_owner(class_id, token_id))));
        body_invariant!(nft.id() === old(nft.id()));
        body_invariant!(i < token_ids.len());
        body_invariant!(
            if !class_id.path.starts_with(source_port, source_channel) {
                forall(|j: usize| j < token_ids.len() ==>
                    transfers_token!(nft, class_id, token_ids.get(j)))
            } else {
                forall(|j: usize| j >= i && j < token_ids.len() ==>
                    transfers_token!(nft, class_id, token_ids.get(j)))
            }
        );
        body_invariant!(forall(|j : usize| j < i ==>
            if old(class_id.path.starts_with(source_port, source_channel)) {
                nft.get_owner(class_id, token_ids.get(j)) == None
            } else {
                nft.get_owner(class_id, token_ids.get(j)) == Some(ctx.escrow_address(source_channel))
            }
        ));
        let token_id = token_ids.get(i);
        if !class_id.path.starts_with(source_port, source_channel) {
            nft.transfer(
                class_id,
                token_id,
                ctx.escrow_address(source_channel),
                None
            );
        } else {
            nft.burn(class_id, token_id);
        };
        i = i + 1;
    }

    let data = make_packet_data(nft, class_id, token_ids, sender, receiver);
    mk_packet(ctx, source_port, source_channel, data)
}

macro_rules! implies {
    ($lhs:expr, $rhs:expr) => {
       if $lhs { $rhs } else { true }
   }
}

#[requires(
    forall(|i : usize, j: usize| i < packet.data.token_ids.len() && j < i ==>
        packet.data.token_ids.get(i) != packet.data.token_ids.get(j)
))]
#[requires(packet.data.token_ids.len() == packet.data.token_uris.len())]
#[requires(packet.data.token_ids.len() == packet.data.token_data.len())]
#[requires(
    !packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) ==>
    forall(|i : usize| i < packet.data.token_ids.len() ==>
        transfers_token!(nft, packet.data.class_id, packet.data.token_ids.get(i)))
)]
#[requires(
    if !packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) {
        forall(|i : usize| i < packet.data.token_ids.len() ==>
            nft.get_owner(packet.data.class_id, packet.data.token_ids.get(i)) == Some(ctx.escrow_address(packet.source_channel)))
    } else {
        forall(|i : usize| i < packet.data.token_ids.len() ==>
            nft.get_owner(packet.data.class_id, packet.data.token_ids.get(i)) == None)
    }
)]
#[ensures(
    forall(|i : usize| i < packet.data.token_ids.len() ==>
        nft.get_owner(packet.data.class_id, packet.data.token_ids.get(i)) == Some(packet.data.sender))
)]
#[ensures(
    forall(|i : usize| i < packet.data.token_ids.len() ==>
        transfers_token!(nft, packet.data.class_id, packet.data.token_ids.get(i)))
)]
fn refund_token(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    let NFTPacketData { class_id, token_ids, token_uris, token_data, sender, ..} = packet.data;
    let mut i = 0;
    while i < token_ids.len() {
        body_invariant!(i <= token_ids.len());
        body_invariant!(nft.id() === old(nft.id()));
        body_invariant!(forall( |class_id: PrefixedClassId, token_id: TokenId|
        ( old(holds(Token(nft.id(), class_id, token_id))) == PermAmount::from(0) 
        &&    holds(Token(nft.id(), class_id, token_id)) == PermAmount::from(0)) ==>
            nft.get_owner(class_id, token_id) == old(nft.get_owner(class_id, token_id))));
        body_invariant!(
            forall(|j: usize| j < i ==>
                nft.get_owner(class_id, token_ids.get(j)) == Some(packet.data.sender))
        );
        body_invariant!(
            if !class_id.path.starts_with(packet.source_port, packet.source_channel) {
                forall(|j: usize| j < token_ids.len() ==>
                    transfers_token!(nft, class_id, token_ids.get(j)))
            } else {
                forall(|j: usize| j < i ==>
                    transfers_token!(nft, class_id, token_ids.get(j)))
            }
        );
        body_invariant!(
            if !class_id.path.starts_with(packet.source_port, packet.source_channel) {
                forall(|j : usize| j >= i && j < token_ids.len() ==>
                    nft.get_owner(class_id, token_ids.get(j)) == Some(ctx.escrow_address(packet.source_channel)))
            } else {
                forall(|j : usize| j >= i && j < token_ids.len() ==>
                    nft.get_owner(class_id, token_ids.get(j)) == None)
            }
        );
        let token_id = token_ids.get(i);
        if !class_id.path.starts_with(packet.source_port, packet.source_channel) {
            nft.transfer(
                class_id,
                token_id,
                sender,
                None
            );
        } else {
            nft.mint(
                class_id,
                token_id,
                token_uris.get(i),
                token_data.get(i),
                sender,
            );
        }
        i += 1;
    }
}

pub fn on_timeout_packet(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    // refund_token(ctx, nft, packet);
}

#[requires(
    forall(|i : usize, j: usize| i < packet.data.token_ids.len() && j < i ==>
        packet.data.token_ids.get(i) != packet.data.token_ids.get(j)
))]
#[requires(packet.data.token_ids.len() == packet.data.token_uris.len())]
#[requires(packet.data.token_ids.len() == packet.data.token_data.len())]
#[requires(
    packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) ==>
    forall(|i : usize| i < packet.data.token_ids.len() ==>
        transfers_token!(
            nft, 
            packet.data.class_id.drop_prefix(packet.source_port, packet.source_channel),
            packet.data.token_ids.get(i)
        )
    )
)]
// PROBLEM BELOW
#[requires(
    if packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) {
        forall(|i : usize| i < packet.data.token_ids.len() ==>
            nft.get_owner(
                PrefixedClassId {
                    base: packet.data.class_id.base,
                    path: packet.data.class_id.path.drop_prefix(packet.source_port, packet.source_channel)
                },
                packet.data.token_ids.get(i)
            ) != None) // TODO: Is the escrow address
    } else {
        forall(|i : usize| i < packet.data.token_ids.len() ==>
            nft.get_owner(
                packet.data.class_id.prepend_prefix(
                    packet.dest_port, 
                    packet.dest_channel
                ),
                packet.data.token_ids.get(i)
            ) == None)
    }
)]
#[ensures(
    forall(|i : usize| i < packet.data.token_ids.len() ==>
        transfers_token!(
            nft,
            packet.get_recv_class_id(),
            packet.data.token_ids.get(i)
        )
    )
)]
// #[ensures(
//     forall(|i : usize| i < packet.data.token_ids.len() ==>
//         nft.get_owner(
//             packet.get_recv_class_id(),
//             packet.data.token_ids.get(i)
//         ) == Some(packet.data.receiver))
// )]
// #[ensures(result.success)]
pub fn on_recv_packet(
    ctx: &Ctx, 
    nft: &mut NFTKeeper,
    packet: &Packet,
    topology: &Topology
) -> NFTPacketAcknowledgement {
    // let class_id = packet.get_recv_class_id();
    let NFTPacketData { class_uri, class_data, token_ids, receiver, token_data, token_uris, .. } = packet.data;
    let mut i = 0;
    while i < token_ids.len() {
        body_invariant!(i <= token_ids.len());
        body_invariant!(nft.id() === old(nft.id()));
        body_invariant!(forall( |class_id: PrefixedClassId, token_id: TokenId|
        ( old(holds(Token(nft.id(), class_id, token_id))) == PermAmount::from(0) 
        &&    holds(Token(nft.id(), class_id, token_id)) == PermAmount::from(0)) ==>
            nft.get_owner(class_id, token_id) == old(nft.get_owner(class_id, token_id))));
        body_invariant!(
            forall(|j: usize| j < i ==>
                nft.get_owner(packet.get_recv_class_id(), token_ids.get(j)) == Some(receiver))
        );
        body_invariant!(
            if packet.is_source() {
                forall(|j: usize| j < token_ids.len() ==>
                    transfers_token!(nft, packet.get_recv_class_id(), token_ids.get(j)))
            } else {
                forall(|j: usize| j < i ==>
                    transfers_token!(nft, packet.get_recv_class_id(), token_ids.get(j)))
            }
        );
        body_invariant!(
            if packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) {
                forall(|j : usize| j >= i && j < token_ids.len() ==>
                    nft.get_owner(packet.get_recv_class_id(), token_ids.get(j)) != None)
            } else {
                forall(|j : usize| j >= i && j < token_ids.len() ==>
                    nft.get_owner(packet.get_recv_class_id(), token_ids.get(j)) == None)
            }
        );
        if packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) {
            nft.transfer(packet.get_recv_class_id(), token_ids.get(i), receiver, Some(token_data.get(i)));
        } else {
            nft.create_or_update_class(packet.get_recv_class_id(), class_uri, class_data);
            nft.mint(packet.get_recv_class_id(), token_ids.get(i), token_uris.get(i), token_data.get(i), receiver);
        };
        i += 1;
    }
    NFTPacketAcknowledgement { success: true }
}

pub fn on_acknowledge_packet(
    ctx: &Ctx,
    nft: &mut NFTKeeper,
    ack: NFTPacketAcknowledgement,
    packet: &Packet) {
    if(!ack.success) {
        // refund_token(ctx, nft, packet);
    }
}
