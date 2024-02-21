#![allow(dead_code, unused)]
use prusti_contracts::*;

use crate::types::*;

/*
 * This macro is used in specifications instead of Prusti's ==> syntax, because
 * The program used to calculate syntactic complexity only supports Rust syntax for AST
 */
#[macro_export]
macro_rules! implies {
    ($lhs:expr, $rhs:expr) => {
        if $lhs {
            $rhs
        } else {
            true
        }
    };
}

#[cfg(feature = "resource")]
#[resource_kind]
pub struct Token(pub KeeperId, pub PrefixedClassId, pub TokenId);

#[cfg(feature = "resource")]
#[macro_export]
// MACRO_RESOURCE_SPEC_START
macro_rules! token_permission {
    ($nft:expr, $class_id:expr, $token_id:expr) => {
        resource(Token($nft.id(), $class_id, $token_id), 1)
    };
}
// MACRO_RESOURCE_SPEC_END

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct KeeperId(u32);

// INVARIANTS_RESOURCE_SPEC_START
#[cfg_attr(
    feature="resource",
    invariant_twostate(
        (forall( |class_id: PrefixedClassId, token_id: TokenId|
            implies!( old(holds(Token(self.id(), class_id, token_id))) == PermAmount::from(0)
                &&    holds(Token(self.id(), class_id, token_id)) == PermAmount::from(0),
        self.get_owner(class_id, token_id) == old(self.get_owner(class_id, token_id))
        // triggers = [(self.get_owner(class_id, token_id))]
)))))]
#[cfg_attr(feature="resource", invariant_twostate(self.id() == old(self.id())))]
// INVARIANTS_RESOURCE_SPEC_END
pub trait NFTKeeper {
    #[pure]
    #[trusted]
    fn id(&self) -> KeeperId {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn get_owner(&self, class_id: PrefixedClassId, token_id: TokenId) -> Option<AccountId> {
        unimplemented!()
    }

    // CREATE_OR_UPDATE_CLASS_SPEC_START
    #[cfg_attr(not(feature="resource"), ensures(
        forall(|class_id: PrefixedClassId, token_id: TokenId|
            (old(self.get_owner(class_id, token_id)) == self.get_owner(class_id, token_id))
    )))]
    // CREATE_OR_UPDATE_CLASS_SPEC_END
    #[trusted]
    fn create_or_update_class(
        &mut self,
        class_id: PrefixedClassId,
        class_uri: ClassUri,
        class_data: ClassData,
    ) {
        unimplemented!()
    }

    #[trusted]
    // MINT_COMMON_SPEC_START
    #[requires(self.get_owner(class_id, token_id) == None)]
    // MINT_COMMON_SPEC_END
    // MINT_RESOURCE_SPEC_START
    #[cfg_attr(feature="resource", ensures(token_permission!(self, class_id, token_id)))]
    #[cfg_attr(feature="resource", ensures(self.get_owner(class_id, token_id) == Some(receiver)))]
    // MINT_RESOURCE_SPEC_END
    // MINT_SPEC_START
    #[cfg_attr(not(feature="resource"), ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == Some(receiver)
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    )))]
    // MINT_SPEC_END
    fn mint(
        &mut self,
        class_id: PrefixedClassId,
        token_id: TokenId,
        token_uri: TokenUri,
        token_data: TokenData,
        receiver: AccountId,
    );

    #[trusted]
    // TRANSFER_RESOURCE_SPEC_START
    #[cfg_attr(feature="resource", requires(token_permission!(self, class_id, token_id)))]
    #[cfg_attr(feature="resource", ensures(token_permission!(self, class_id, token_id)))]
    #[cfg_attr(feature="resource", ensures(self.get_owner(class_id, token_id) == Some(receiver)))]
    // TRANSFER_RESOURCE_SPEC_END
    // TRANSFER_SPEC_START
    #[cfg_attr(not(feature="resource"), ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == Some(receiver)
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    )))]
    // TRANSFER_SPEC_END
    fn transfer(
        &mut self,
        class_id: PrefixedClassId,
        token_id: TokenId,
        receiver: AccountId,
        token_data: Option<TokenData>,
    );

    // BURN_RESOURCE_SPEC_START
    #[cfg_attr(feature="resource", requires(token_permission!(self, class_id, token_id)))]
    #[cfg_attr(feature="resource", ensures(self.get_owner(class_id, token_id) == None))]
    // BURN_RESOURCE_SPEC_END
    // BURN_SPEC_START
    #[cfg_attr(not(feature="resource"), ensures(
        forall(|class_id2: PrefixedClassId, token_id2: TokenId|
            if (class_id2 == class_id && token_id2 == token_id) {
                self.get_owner(class_id, token_id) == None
            } else {
                self.get_owner(class_id2, token_id2) == old(self.get_owner(class_id2, token_id2))
            }
    )))]
    // BURN_SPEC_END
    #[trusted]
    fn burn(&mut self, class_id: PrefixedClassId, token_id: TokenId) {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn get_nft(&self, class_id: PrefixedClassId, token_id: TokenId) -> NFT {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn get_class(&self, class_id: PrefixedClassId) -> Class {
        unimplemented!()
    }
}

#[pure]
fn make_packet_data<T: NFTKeeper>(
    nft: &T,
    class_id: PrefixedClassId,
    token_ids: TokenIdVec,
    token_uris: TokenUriVec,
    token_data: TokenDataVec,
    sender: AccountId,
    receiver: AccountId,
) -> NFTPacketData {
    NFTPacketData {
        class_id,
        class_data: nft.get_class(class_id).data,
        class_uri: nft.get_class(class_id).uri,
        token_ids,
        token_data,
        token_uris,
        sender,
        receiver,
    }
}

// SEND_NFT_COMMON_SPEC_START
#[requires(
    forall(|i : usize| implies!(i < token_ids.len(),
        nft.get_owner(class_id, token_ids.get(i)) == Some(sender))
))]
#[requires(
    forall(|i : usize, j: usize| implies!(i < token_ids.len() && j < i,
        token_ids.get(i) != token_ids.get(j))
))]
// SEND_NFT_COMMON_SPEC_END
#[cfg_attr(feature="resource", requires(
    forall(|i : usize| implies!(i < token_ids.len(),
        token_permission!(nft, class_id, token_ids.get(i))))
))]
// It's a transfer to escrow locally, retain permission
// SEND_NFT_RESOURCE_SPEC_START
#[cfg_attr(feature="resource", ensures(
    implies!(old(!class_id.path.starts_with(source_port, source_channel)),
    forall(|i : usize| implies!(i < token_ids.len(),
        token_permission!(nft, class_id, token_ids.get(i)))))
))]
// SEND_NFT_RESOURCE_SPEC_END
// SEND_NFT_COMMON_SPEC_START
#[ensures(
    forall(|i : usize| implies!(i < token_ids.len(),
        if old(class_id.path.starts_with(source_port, source_channel)) {
            nft.get_owner(class_id, token_ids.get(i)) == None
        } else {
            nft.get_owner(class_id, token_ids.get(i)) == Some(ctx.escrow_address(source_channel))
        }
    )
))]
// SEND_NFT_COMMON_SPEC_END
// SEND_NFT_SPEC_START
#[cfg_attr(not(feature="resource"), ensures(
    forall(|class_id2: PrefixedClassId, token_id2: TokenId|
        implies!((class_id2 != class_id || forall(
            |i: usize| implies!(i < token_ids.len(), token_ids.get(i) != token_id2)
        )),
            nft.get_owner(class_id2, token_id2) == old(nft.get_owner(class_id2, token_id2)))
)))]
// SEND_NFT_SPEC_END
#[ensures(result.data.token_ids.len() == result.data.token_uris.len())]
#[ensures(result.data.token_ids.len() == result.data.token_data.len())]
#[ensures(result.data.token_ids === token_ids)]
#[ensures(result.data.class_id === class_id)]
#[ensures(result.data.sender === sender)]
#[ensures(result.data.receiver === receiver)]
#[ensures(result.source_port === source_port)]
#[ensures(result.source_channel === source_channel)]
#[ensures(result.dest_port === ctx.counterparty_port(source_port, source_channel))]
#[ensures(result.dest_channel === ctx.counterparty_channel(source_port, source_channel))]
pub fn send_nft<T: NFTKeeper>(
    ctx: &Ctx,
    nft: &mut T,
    class_id: PrefixedClassId,
    token_ids: TokenIdVec,
    sender: AccountId,
    receiver: AccountId,
    source_port: Port,
    source_channel: ChannelEnd,
    topology: &Topology,
) -> Packet {
    let mut i = 0;
    let mut token_uris = TokenUriVec::new();
    let mut token_data = TokenDataVec::new();
    while i < token_ids.len() {
        body_invariant!(i < token_ids.len());
        #[cfg(feature = "resource")]
        // SEND_NFT_RESOURCE_SPEC_START
        body_invariant!(if !class_id.path.starts_with(source_port, source_channel) {
            forall(|j: usize| {
                implies!(
                    j < token_ids.len(),
                    token_permission!(nft, class_id, token_ids.get(j))
                )
            })
        } else {
            forall(|j: usize| {
                implies!(
                    j >= i && j < token_ids.len(),
                    token_permission!(nft, class_id, token_ids.get(j))
                )
            })
        });
        // SEND_NFT_RESOURCE_SPEC_END
        // SEND_NFT_COMMON_SPEC_START
        body_invariant!(forall(|j: usize| implies!(
            j < i,
            if old(class_id.path.starts_with(source_port, source_channel)) {
                nft.get_owner(class_id, token_ids.get(j)) == None
            } else {
                nft.get_owner(class_id, token_ids.get(j))
                    == Some(ctx.escrow_address(source_channel))
            }
        )));
        // SEND_NFT_COMMON_SPEC_END
        let token_id = token_ids.get(i);
        if !class_id.path.starts_with(source_port, source_channel) {
            nft.transfer(class_id, token_id, ctx.escrow_address(source_channel), None);
        } else {
            nft.burn(class_id, token_id);
        };
        let token = nft.get_nft(class_id, token_id);
        token_uris.push(token.uri);
        token_data.push(token.data);
        i += 1;
    }

    let data = make_packet_data(
        nft, class_id, token_ids, token_uris, token_data, sender, receiver,
    );
    mk_packet(ctx, source_port, source_channel, data)
}

// I suppose we don't care
fn refund_token<T: NFTKeeper>(ctx: &Ctx, nft: &mut T, packet: &Packet) {}

// I supose we don't care
fn on_timeout_packet<T: NFTKeeper>(ctx: &Ctx, nft: &mut T, packet: &Packet) {
    refund_token(ctx, nft, packet);
}

// ON_RECV_PACKET_COMMON_SPEC_START
#[requires(
    forall(|i : usize, j: usize| implies!(i < packet.data.token_ids.len() && j < i,
        packet.data.token_ids.get(i) != packet.data.token_ids.get(j))
))]
#[requires(packet.data.token_ids.len() == packet.data.token_uris.len())]
#[requires(packet.data.token_ids.len() == packet.data.token_data.len())]
#[requires(
    if packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel) {
        forall(|i : usize| implies!(i < packet.data.token_ids.len(),
            nft.get_owner(
                packet.data.class_id.drop_prefix(
                    packet.source_port,
                    packet.source_channel
                ),
                packet.data.token_ids.get(i)
            ) == Some(ctx.escrow_address(packet.dest_channel))))
    } else {
        forall(|i : usize| implies!(i < packet.data.token_ids.len(),
            nft.get_owner(
                packet.data.class_id.prepend_prefix(
                    packet.dest_port,
                    packet.dest_channel
                ),
                packet.data.token_ids.get(i)
            ) == None))
    }
)]
// ON_RECV_PACKET_COMMON_SPEC_END
// ON_RECV_PACKET_RESOURCE_SPEC_START
#[cfg_attr(feature="resource", requires(
    implies!(packet.data.class_id.path.starts_with(packet.source_port, packet.source_channel)
    , forall(|i : usize| implies!(i < packet.data.token_ids.len(),
        token_permission!(
            nft,
            packet.data.class_id.drop_prefix(packet.source_port, packet.source_channel),
            packet.data.token_ids.get(i)
        ))
    )
)))]
#[cfg_attr(feature="resource", ensures(
    forall(|i : usize| implies!(i < old(packet.data).token_ids.len(),
        token_permission!(
            nft,
            old(packet.get_recv_class_id()),
            packet.data.token_ids.get(i)
        )
    )
)))]
// ON_RECV_PACKET_RESOURCE_SPEC_END
// ON_RECV_PACKET_COMMON_SPEC_START
#[ensures(
    forall(|i : usize| implies!(i < old(packet.data.token_ids.len()),
        nft.get_owner(
            old(packet.get_recv_class_id()),
            packet.data.token_ids.get(i)
        ) == Some(packet.data.receiver))
))]
#[ensures(result.success)]
// ON_RECV_PACKET_COMMON_SPEC_END
// ON_RECV_PACKET_SPEC_START
#[cfg_attr(not(feature="resource"), ensures(
    forall(|class_id2: PrefixedClassId, token_id2: TokenId|
        implies!(class_id2 != old(packet.get_recv_class_id()) || forall(
            |i: usize| implies!(i < packet.data.token_ids.len(), packet.data.token_ids.get(i) != token_id2)
        ), nft.get_owner(class_id2, token_id2) == old(nft.get_owner(class_id2, token_id2)))
)))]
// ON_RECV_PACKET_SPEC_END
pub fn on_recv_packet<T: NFTKeeper>(
    ctx: &Ctx,
    nft: &mut T,
    packet: &Packet,
    topology: &Topology,
) -> NFTPacketAcknowledgement {
    // let class_id = packet.get_recv_class_id();
    let NFTPacketData {
        class_uri,
        class_data,
        token_ids,
        receiver,
        token_data,
        token_uris,
        ..
    } = packet.data;
    let mut i = 0;
    while i < token_ids.len() {
        // ON_RECV_PACKET_COMMON_SPEC_START
        body_invariant!(i <= token_ids.len());
        body_invariant!(forall(|j: usize| implies!(
            j < i,
            nft.get_owner(packet.get_recv_class_id(), token_ids.get(j)) == Some(receiver)
        )));
        body_invariant!(forall(|j: usize| implies!(
            j >= i && j < token_ids.len() && !packet.is_source(),
            nft.get_owner(packet.get_recv_class_id(), token_ids.get(j)) == None
        )));
        // ON_RECV_PACKET_COMMON_SPEC_END
        // ON_RECV_PACKET_RESOURCE_SPEC_START
        #[cfg(feature = "resource")]
        body_invariant!(if packet.is_source() {
            forall(|j: usize| {
                implies!(
                    j < token_ids.len(),
                    token_permission!(nft, packet.get_recv_class_id(), token_ids.get(j))
                )
            })
        } else {
            forall(|j: usize| {
                implies!(
                    j < i,
                    token_permission!(nft, packet.get_recv_class_id(), token_ids.get(j))
                )
            })
        });
        // ON_RECV_PACKET_RESOURCE_SPEC_END
        // ON_RECV_PACKET_SPEC_START
        #[cfg(not(feature = "resource"))]
        body_invariant!(forall(
            |class_id2: PrefixedClassId, token_id2: TokenId| implies!(
                class_id2 != old(packet.get_recv_class_id())
                    || forall(|i: usize| implies!(
                        i < token_ids.len(),
                        token_ids.get(i) != token_id2
                    )),
                nft.get_owner(class_id2, token_id2) == old(nft.get_owner(class_id2, token_id2))
            )
        ));
        // ON_RECV_PACKET_SPEC_END

        if packet.is_source() {
            nft.transfer(
                packet.get_recv_class_id(),
                token_ids.get(i),
                receiver,
                Some(token_data.get(i)),
            );
        } else {
            nft.create_or_update_class(packet.get_recv_class_id(), class_uri, class_data);
            nft.mint(
                packet.get_recv_class_id(),
                token_ids.get(i),
                token_uris.get(i),
                token_data.get(i),
                receiver,
            );
        };
        i += 1;
    }
    NFTPacketAcknowledgement { success: true }
}

#[ensures(
    forall(|class_id: PrefixedClassId, token_id: TokenId|
        (old(nft.get_owner(class_id, token_id)) == nft.get_owner(class_id, token_id))
))]
pub fn on_acknowledge_packet<T: NFTKeeper>(
    ctx: &Ctx,
    nft: &mut T,
    ack: NFTPacketAcknowledgement,
    packet: &Packet,
) {
    // if(!ack.success) {
    //     refund_token(ctx, nft, packet);
    // }
}
