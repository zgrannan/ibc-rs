#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;


#[resource]
pub struct HoldsToken(pub KeeperId, pub AccountId, pub PrefixedClassId, pub TokenId);

#[derive(Copy, Clone)]
pub struct KeeperId(u32);

// #[invariant_twostate(
//     forall(
//         |acct_id: AccountId, class_id: PrefixedClassId, token_id: TokenId|
//             holds(HoldsToken(self.id(), acct_id, class_id, token_id)) == PermAmount::from(1) ==>
//                self.get_owner(class_id, token_id) == acct_id
// ))]
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
    pub fn get_owner(&self, class_id: PrefixedClassId, token_id: TokenId) -> AccountId {
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
    #[ensures(transfers(HoldsToken(self.id(), receiver, class_id, token_id), 1))]
    #[ensures(self.get_owner(class_id, token_id) == receiver)]
    pub fn mint(&mut self,
                class_id: PrefixedClassId,
                token_id: TokenId,
                token_uri: TokenUri,
                token_data: TokenData,
                receiver: AccountId) {
        unimplemented!()
    }

    #[requires(transfers(HoldsToken(self.id(), self.get_owner(class_id, token_id), class_id, token_id), 1))]
    #[ensures(transfers(HoldsToken(self.id(), receiver, class_id, token_id), 1))]
    #[ensures(self.get_owner(class_id, token_id) == receiver)]
    #[trusted]
    pub fn transfer(&mut self,
                    class_id: PrefixedClassId,
                    token_id: TokenId,
                    receiver: AccountId,
                    token_data: Option<TokenData>) {
        unimplemented!()
    }

    #[requires(transfers(HoldsToken(self.id(), self.get_owner(class_id, token_id), class_id, token_id), 1))]
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
fn make_packet_data(nft: &NFTKeeper, class_id: PrefixedClassId, token_id: TokenId, sender: AccountId, receiver: AccountId) -> NFTPacketData {
    NFTPacketData {
        class_id,
        class_data: nft.get_class(class_id).data,
        class_uri: nft.get_class(class_id).uri,
        token_id,
        token_data: nft.get_nft(class_id, token_id).data,
        token_uri: nft.get_nft(class_id, token_id).uri,
        sender,
        receiver
    }
}

#[requires(transfers(HoldsToken(nft.id(), sender, class_id, token_id), 1))]
#[requires(sender == nft.get_owner(class_id, token_id))]
#[ensures(!class_id.path.starts_with(source_port, source_channel) ==>
    transfers(HoldsToken(nft.id(), ctx.escrow_address(source_channel), class_id, token_id), 1)
)]
#[ensures(
    result == mk_packet(
        ctx,
        source_port,
        source_channel,
        make_packet_data(nft, class_id, token_id, sender, receiver)
    )
)]
pub fn send_nft(
    ctx: &Ctx,
    nft: &mut NFTKeeper,
    class_id: PrefixedClassId,
    token_id: TokenId,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology
) -> Packet {
    assert!(sender == nft.get_owner(class_id, token_id));
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

    let data = make_packet_data(nft, class_id, token_id, sender, receiver);
    mk_packet(ctx, source_port, source_channel, data)
}

macro_rules! implies {
    ($lhs:expr, $rhs:expr) => {
       if $lhs { $rhs } else { true }
   }
}

macro_rules! refund_token_pre {
    ($ctx:expr, $nft:expr, $packet:expr) => {
        implies!(
            !$packet.data.class_id.path.starts_with($packet.source_port, $packet.source_channel),
            ($nft.get_owner($packet.data.class_id, $packet.data.token_id) == 
                $ctx.escrow_address($packet.source_channel))
            && 
            transfers(
                HoldsToken($nft.id(),
                    $ctx.escrow_address($packet.source_channel),
                    $packet.data.class_id,
                    $packet.data.token_id
                ), 1)
        )
    }
}

macro_rules! refund_tokens_post {
    ($nft:expr, $packet:expr) => {
        transfers(HoldsToken($nft.id(),$packet.data.sender, $packet.data.class_id, $packet.data.token_id), 1) &&
        $nft.get_owner($packet.data.class_id, $packet.data.token_id) == $packet.data.sender
    }
}


#[requires(refund_token_pre!(ctx, nft, packet))]
#[ensures(refund_tokens_post!(nft, packet))]
fn refund_token(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    let NFTPacketData { class_id, token_id, token_uri, token_data, sender, ..} = packet.data;
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
            token_uri,
            token_data,
            sender,
        );
    }
}

#[requires(refund_token_pre!(ctx, nft, packet))]
#[ensures(refund_tokens_post!(nft, packet))]
pub fn on_timeout_packet(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    refund_token(ctx, nft, packet);
}

#[requires(packet.is_source() ==> transfers(
    HoldsToken(nft.id(),
        nft.get_owner(packet.get_recv_class_id(), packet.data.token_id),
        packet.get_recv_class_id(),
        packet.data.token_id
    ), 1)
)]
#[ensures(
    transfers(
        HoldsToken(nft.id(),
            packet.data.receiver,
            packet.get_recv_class_id(),
            packet.data.token_id
        ), 1
    )
)]
#[ensures(nft.get_owner(packet.get_recv_class_id(), packet.data.token_id) == packet.data.receiver)]
pub fn on_recv_packet(
    ctx: &Ctx, 
    nft: &mut NFTKeeper,
    packet: &Packet,
    topology: &Topology
) -> NFTPacketAcknowledgement {
    let class_id = packet.get_recv_class_id();
    let NFTPacketData { token_id, receiver, token_data, token_uri, .. } = packet.data;
    if packet.is_source() {
        nft.transfer(class_id, token_id, receiver, Some(token_data));
    } else {
        nft.create_or_update_class(class_id, packet.data.class_uri, packet.data.class_data);
        nft.mint(class_id, token_id, token_uri, token_data, receiver);
    };
    NFTPacketAcknowledgement { success: true }
}

#[requires(!ack.success ==> refund_token_pre!(ctx, nft, packet))]
#[ensures(!ack.success ==> refund_tokens_post!(nft, packet))]
pub fn on_acknowledge_packet(
    ctx: &Ctx,
    nft: &mut NFTKeeper,
    ack: NFTPacketAcknowledgement,
    packet: &Packet) {
    if(!ack.success) {
        refund_token(ctx, nft, packet);
    }
}
