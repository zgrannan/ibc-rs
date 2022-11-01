#![allow(dead_code, unused)]
use std::convert::TryInto;
use prusti_contracts::*;

#[derive(Copy, Clone, Eq, PartialEq)]
struct AccountID(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
struct Coin(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
struct ChannelEnd(u32);
#[derive(Copy, Clone, Eq, PartialEq)]
struct Port(u32);

struct Ctx(u32);

impl Ctx {
    #[pure]
    #[trusted]
    fn escrow_address(&self, channel_end: ChannelEnd) -> AccountID {
        unimplemented!()
    }
}

#[extern_spec]
impl<T> std::option::Option<T> {
    #[pure]
    #[ensures(matches!(*self, Some(_)) == result)]
    pub fn is_some(&self) -> bool;

    #[pure]
    #[requires(self.is_some())]
    #[ensures(self === Some(result))]
    pub fn unwrap(self) -> T;
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct FungibleTokenPacketData {
    path: Path,
    coin: Coin,
    sender: AccountID,
    receiver: AccountID,
    amount: u32
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Packet {
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    data: FungibleTokenPacketData
}

#[pure]
#[trusted]
#[ensures(result.source_port == source_port)]
#[ensures(result.source_channel == source_channel)]
#[ensures(result.data == data)]
fn mk_packet(
    source_port: Port,
    source_channel: ChannelEnd,
    data: FungibleTokenPacketData
) -> Packet {
    unimplemented!()
}

#[derive(Copy, Clone, Eq, Hash)]
struct Path(u32);

impl Path {
    #[pure]
    #[trusted]
    #[ensures(result.starts_with(port, channel))]
    #[ensures(result.tail() === self)]
    #[ensures(result.drop_prefix(port, channel) === self)]
    fn prepend_prefix(self, port: Port, channel: ChannelEnd) -> Path {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn starts_with(&self, port: Port, channel: ChannelEnd) -> bool {
        unimplemented!()
    }

    #[pure]
    #[requires(self.starts_with(port, channel))]
    #[ensures(result === self.tail())]
    #[trusted]
    fn drop_prefix(&self, port: Port, channel: ChannelEnd) -> Path {
        unimplemented!()
    }

    #[pure]
    #[trusted]
    fn tail(&self) -> Path {
       unimplemented!()
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



trait Bank {

    #[pure]
    fn balance_of(&self, acct_id: AccountID, path: Path, coin: Coin) -> u32;

    predicate! {
        fn transfer_tokens_post(
            &self,
            old_bank: &Self,
            from: AccountID,
            to: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
            forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
            self.balance_of(acct_id2, path2, coin2) ==
                if(acct_id2 == from && coin == coin2 && path === path2) {
                    old_bank.balance_of(from, path, coin) - amt
                } else if (acct_id2 == to && coin == coin2 && path === path2){
                    old_bank.balance_of(to, path, coin) + amt
                } else {
                    old_bank.balance_of(acct_id2, path2, coin2)
                }
            )
        }
    }

    #[pure]
    fn transfer_tokens_pre(
        &self,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) -> bool {
        from != to && self.balance_of(from, path, coin) >= amt
    }

    #[ensures(result == old(self.transfer_tokens_pre(from, to, path, coin, amt)))]
    #[ensures(result ==> self.transfer_tokens_post(
        old(self),
        from,
        to,
        path,
        coin,
        amt
    ))]
    fn transfer_tokens(
        &mut self,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) -> bool {
        if(self.transfer_tokens_pre(from, to, path, coin, amt)) {
            self.burn_tokens(from, path, coin, amt);
            self.mint_tokens(to, path, coin, amt);
            true
        } else {
            false
        }
    }

    predicate! {
        fn burn_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
            forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
            self.balance_of(acct_id2, path2, coin2) ==
                if(acct_id == acct_id2 && coin == coin2 && path === path2) {
                    old_bank.balance_of(acct_id, path, coin) - amt
                } else {
                    old_bank.balance_of(acct_id2, path2, coin2)
                }
            )
        }
    }

    #[ensures(result == (old(self.balance_of(to, path, coin)) >= amt))]
    #[ensures(result ==> self.burn_tokens_post(old(self), to, path, coin, amt))]
    fn burn_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool;

    predicate! {
        fn mint_tokens_post(
            &self,
            old_bank: &Self,
            acct_id: AccountID,
            path: Path,
            coin: Coin,
            amt: u32
        ) -> bool {
            forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
                self.balance_of(acct_id2, path2, coin2) ==
                    if(acct_id == acct_id2 && coin == coin2 && path === path2) {
                        old_bank.balance_of(acct_id, path, coin) + amt
                    } else {
                        old_bank.balance_of(acct_id2, path2, coin2)
                    }
            )
        }
    }

    #[ensures(result)]
    #[ensures(self.mint_tokens_post(old(self), to, path, coin, amt))]
    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool;
}

#[pure]
fn send_will_burn<B: Bank>(
    bank: &B,
    path: Path,
    source_port: Port,
    source_channel: ChannelEnd,
    sender: AccountID,
    coin: Coin,
    amount: u32
) -> bool {
    path.starts_with(source_port, source_channel) &&
    bank.balance_of(sender, path, coin) >= amount
}

#[pure]
fn send_will_transfer<B: Bank>(
    bank: &B,
    path: Path,
    source_port: Port,
    source_channel: ChannelEnd,
    sender: AccountID,
    escrow_address: AccountID,
    coin: Coin,
    amount: u32
) -> bool {
    !path.starts_with(source_port, source_channel) &&
    bank.transfer_tokens_pre(sender, escrow_address, path, coin, amount)
}

#[ensures(
    old(send_will_burn(bank, path, source_port, source_channel, sender, coin, amount)) ==>
        (result.is_some() && bank.burn_tokens_post(old(bank), sender, path, coin, amount)
))]
#[ensures(
    old(
        send_will_transfer(
            bank,
            path,
            source_port,
            source_channel,
            sender,
            ctx.escrow_address(source_channel),
            coin,
    amount)) ==> result.is_some() &&
        bank.transfer_tokens_post(
            old(bank),
            sender,
            old(ctx.escrow_address(source_channel)),
            path,
            coin,
            amount
    )
)]
#[ensures(
    result.is_some() ==>
    result == Some(mk_packet(
        source_port,
        source_channel,
        FungibleTokenPacketData {path, coin, sender, receiver, amount}
    )
))]
fn send_fungible_tokens<B: Bank>(
    ctx: &Ctx,
    bank: &mut B,
    path: Path,
    coin: Coin,
    amount: u32,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd
) -> Option<Packet> {
    let success = if(!path.starts_with(source_port, source_channel)) {
        bank.transfer_tokens(
            sender,
            ctx.escrow_address(source_channel),
            path,
            coin,
            amount
        )
    } else {
        bank.burn_tokens(
            sender,
            path,
            coin,
            amount
        )
    };

    if success {
        let data = FungibleTokenPacketData {
            path,
            coin,
            sender,
            receiver,
            amount
        };
        Some(mk_packet(source_port, source_channel, data))
    } else {
        None
    }
}

#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
fn on_timeout_packet<B: Bank>(ctx: &Ctx, bank: &mut B, packet: Packet) {
    refund_tokens(ctx, bank, packet);
}

predicate! {
    fn refund_tokens_post<B: Bank>(ctx: &Ctx, bank: &B, old_bank: &B, packet: Packet) -> bool {
        ((!packet.data.path.starts_with(packet.source_port, packet.source_channel) &&
            old_bank.transfer_tokens_pre(
                ctx.escrow_address(packet.source_channel),
                packet.data.sender,
                packet.data.path,
                packet.data.coin,
                packet.data.amount
            )) ==> bank.transfer_tokens_post(
        old_bank,
        ctx.escrow_address(packet.source_channel),
        packet.data.sender,
        packet.data.path,
        packet.data.coin,
        packet.data.amount)) &&
        (packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
            bank.mint_tokens_post(
                old_bank,
                packet.data.sender,
                packet.data.path,
                packet.data.coin,
                packet.data.amount
            ))
    }
}

#[ensures(refund_tokens_post(ctx, bank, old(bank), packet))]
fn refund_tokens<B: Bank>(ctx: &Ctx, bank: &mut B, packet: Packet) {
    let FungibleTokenPacketData{ path, coin, sender, amount, ..} = packet.data;
    if !path.starts_with(packet.source_port, packet.source_channel) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.source_channel),
            sender,
            path,
            coin,
            amount
        );
    } else {
        bank.mint_tokens(
            sender,
            path,
            coin,
            amount
        );
    }
}

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

#[pure]
fn packet_is_source(packet: Packet) -> bool {
    packet.data.path.starts_with(packet.source_port, packet.source_channel)
}

#[ensures(
    !packet_is_source(packet) ==>
        result.success && bank.mint_tokens_post(
            old(bank),
            old(packet.data.receiver),
            old(packet.data.path.prepend_prefix(packet.dest_port, packet.dest_channel)),
            old(packet.data.coin),
            old(packet.data.amount)
        )
)]
#[ensures(
    (packet_is_source(packet) &&
     old(
         bank.transfer_tokens_pre(
            ctx.escrow_address(packet.dest_channel),
            packet.data.receiver,
            packet.data.path.drop_prefix(packet.source_port, packet.source_channel),
            packet.data.coin,
            packet.data.amount
        ))) ==> (result.success &&
          bank.transfer_tokens_post(
              old(bank),
              old(ctx.escrow_address(packet.dest_channel)),
              old(packet.data.receiver),
              old(packet.data.path.tail()),
              old(packet.data.coin),
              old(packet.data.amount)
          )
        )
)]
fn on_recv_packet<B: Bank>(ctx: &Ctx, bank: &mut B, packet: Packet) -> FungibleTokenPacketAcknowledgement {
    let FungibleTokenPacketData{ path, coin, receiver, amount, ..} = packet.data;
    let success = if packet_is_source(packet) {
        bank.transfer_tokens(
            ctx.escrow_address(packet.dest_channel),
            receiver,
            path.drop_prefix(packet.source_port, packet.source_channel),
            coin,
            amount
        )
    } else {
        bank.mint_tokens(
            receiver,
            path.prepend_prefix(packet.dest_port, packet.dest_channel),
            coin,
            amount
        )
    };

    FungibleTokenPacketAcknowledgement { success }
}

#[ensures(ack.success ==> snap(bank) === old(snap(bank)))]
fn on_acknowledge_packet<B: Bank>(
    ctx: &Ctx,
    bank: &mut B,
    ack: FungibleTokenPacketAcknowledgement,
    packet: Packet) {
    if(!ack.success) {
        refund_tokens(ctx, bank, packet);
    }
}

/*
 * This method performs a round trip of a token from chain A --> B --> A,
 * The specification ensures that the resulting balances on both banks are the
 * same as they were initially.
 */

// Assume the sender's address is distinct from the escrow address for the source channel,
// and that they have sufficient funds to send to `receiver`
#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]

// Assume that the sender is the source chain
#[requires(!path.starts_with(source_port, source_channel))]

// Ensure that the resulting balance of both bank accounts are unchanged after the round-trip
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank2.balance_of(acct_id2, path2, coin2) ==
           old(bank2).balance_of(acct_id2, path2, coin2)))]
fn round_trip<B: Bank>(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut B,
    bank2: &mut B,
    path: Path,
    coin: Coin,
    amount: u32,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd
) {
    // Send tokens A --> B

    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel
    );
    // packet.is_some() means that this call did not fail
    prusti_assert!(packet.is_some());
    let packet = packet.unwrap();

    // Assume that the destination port and channel have been set correctly
    // (I think this would be done by some routing module?)
    prusti_assume!(packet.dest_port == dest_port);
    prusti_assume!(packet.dest_channel == dest_channel);

    let ack = on_recv_packet(ctx2, bank2, packet);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, packet);

    // Send tokens B --> A

    let packet = send_fungible_tokens(
        ctx2,
        bank2,
        path.prepend_prefix(dest_port, dest_channel),
        coin,
        amount,
        receiver,
        sender,
        dest_port,
        dest_channel
    );
    prusti_assert!(packet.is_some());
    let packet = packet.unwrap();
    prusti_assume!(packet.dest_port == source_port);
    prusti_assume!(packet.dest_channel == source_channel);

    let ack = on_recv_packet(ctx1, bank1, packet);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx2, bank2, ack, packet);
}

#[requires(
    bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))
]
#[requires(!path.starts_with(source_port, source_channel))]
#[ensures(
    forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
        bank1.balance_of(acct_id2, path2, coin2) ==
           old(bank1).balance_of(acct_id2, path2, coin2)))]
fn timeout<B: Bank>(
    ctx1: &Ctx,
    ctx2: &Ctx,
    bank1: &mut B,
    path: Path,
    coin: Coin,
    amount: u32,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd
) {
    // Send tokens A --> B
    let packet = send_fungible_tokens(
        ctx1,
        bank1,
        path,
        coin,
        amount,
        sender,
        receiver,
        source_port,
        source_channel
    );
    // packet.is_some() means that this call did not fail
    prusti_assert!(packet.is_some());
    let packet = packet.unwrap();

    // Assume that the destination port and channel have been set correctly
    // (I think this would be done by some routing module?)
    prusti_assume!(packet.dest_port == dest_port);
    prusti_assume!(packet.dest_channel == dest_channel);

    on_timeout_packet(ctx1, bank1, packet);
}


pub fn main(){}
