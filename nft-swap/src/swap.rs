#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;


#[resource_kind]
pub struct Token(pub KeeperId, pub PrefixedClassId, pub TokenId);

#[derive(Copy, Clone)]
pub struct KeeperId(u32);

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

    // CREATE_OR_UPDATE_CLASS_SPEC_ANNOTATIONS_START
    #[ensures(
        forall(|class_id: PrefixedClassId, token_id: TokenId|
            (old(self.get_owner(class_id, token_id)) == self.get_owner(class_id, token_id))
    ))]
    // CREATE_OR_UPDATE_CLASS_SPEC_ANNOTATIONS_END
    #[trusted]
    pub fn create_or_update_class(
        &mut self,
        class_id: PrefixedClassId,
        class_uri: ClassUri,
        class_data: ClassData) {
        unimplemented!()
    }

    #[trusted]
    // MINT_SPEC_ANNOTATIONS_START
    #[requires(self.get_owner(class_id, token_id) == None)]
    #[ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == Some(receiver)
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    ))]
    // MINT_SPEC_ANNOTATIONS_END
    pub fn mint(&mut self,
                class_id: PrefixedClassId,
                token_id: TokenId,
                token_uri: TokenUri,
                token_data: TokenData,
                receiver: AccountId) {
        unimplemented!()
    }

    #[trusted]
    // SEND_SPEC_ANNOTATIONS_START
    #[ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == Some(receiver)
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    ))]
    // SEND_SPEC_ANNOTATIONS_END
    pub fn transfer(&mut self,
                    class_id: PrefixedClassId,
                    token_id: TokenId,
                    receiver: AccountId,
                    token_data: Option<TokenData>) {
        unimplemented!()
    }

    // BURN_SPEC_ANNOTATIONS_START
    #[ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == None
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    ))]
    // BURN_SPEC_ANNOTATIONS_END
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

// SEND_NFT_SPEC_ANNOTATIONS_START
#[requires(nft.get_owner(class_id, token_id) == Some(sender))]
#[ensures(forall(|class_id2: PrefixedClassId, token_id2: TokenId|
    if (class_id2 == class_id && token_id2 == token_id) {
        if old(class_id.path.starts_with(source_port, source_channel)) {
            nft.get_owner(class_id, token_id) == None
        } else {
            nft.get_owner(class_id, token_id) == Some(ctx.escrow_address(source_channel))
        }
    } else {
        nft.get_owner(class_id2, token_id2) == old(nft.get_owner(class_id2, token_id2))
    }
))]
// SEND_NFT_SPEC_ANNOTATIONS_END
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

fn refund_token(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    // let NFTPacketData { class_id, token_id, token_uri, token_data, sender, ..} = packet.data;
    // if !class_id.path.starts_with(packet.source_port, packet.source_channel) {
    //     nft.transfer(
    //         class_id,
    //         token_id,
    //         sender,
    //         None
    //     );
    // } else {
    //     nft.mint(
    //         class_id,
    //         token_id,
    //         token_uri,
    //         token_data,
    //         sender,
    //     );
    // }
}

pub fn on_timeout_packet(ctx: &Ctx, nft: &mut NFTKeeper, packet: &Packet) {
    refund_token(ctx, nft, packet);
}

// ON_RECV_PACKET_SPEC_ANNOTATIONS_START
#[requires(
    if packet.is_source() {
        nft.get_owner(packet.get_recv_class_id(), packet.data.token_id) == Some(
            ctx.escrow_address(packet.dest_channel)
        ) 
    } else {
        nft.get_owner(packet.get_recv_class_id(), packet.data.token_id) == None
    }
)]
#[ensures(
    forall(|class_id: PrefixedClassId, token_id: TokenId|
        if (class_id == packet.get_recv_class_id() && token_id == old(packet.data.token_id)) {
            nft.get_owner(class_id, token_id) == Some(old(packet.data.receiver))
        } else {
            nft.get_owner(class_id, token_id) == old(nft.get_owner(class_id, token_id))
        }
))]
// ON_RECV_PACKET_SPEC_ANNOTATIONS_END
#[ensures(result.success)]
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

// TODO
#[ensures(
    forall(|class_id: PrefixedClassId, token_id: TokenId|
        (old(nft.get_owner(class_id, token_id)) == nft.get_owner(class_id, token_id))
))]
pub fn on_acknowledge_packet(
    ctx: &Ctx,
    nft: &mut NFTKeeper,
    ack: NFTPacketAcknowledgement,
    packet: &Packet) {
    // if(!ack.success) {
    //     refund_token(ctx, nft, packet);
    // }
}
