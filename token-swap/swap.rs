#![allow(dead_code, unused)]
use std::convert::TryInto;
use prusti_contracts::*;

// type AccountID = u32;

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

predicate! {
    fn transfer_post<B: Bank>(
        new_bank: &B,
        old_bank: &B,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) -> bool {
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          new_bank.balance_of(acct_id2, path2, coin2) ==
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

predicate! {
    fn adjusted<B: Bank>(
        new_bank: &B,
        old_bank: &B,
        acct_id: AccountID,
        path: Path,
        coin: Coin,
        amt: i32
    ) -> bool {
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          new_bank.balance_of(acct_id2, path2, coin2) ==
          if(acct_id == acct_id2 && coin == coin2 && path === path2) {
            if(amt >= 0) {
                old_bank.balance_of(acct_id, path, coin) + (amt as u32)
            } else {
                old_bank.balance_of(acct_id, path, coin) - ((0 - amt) as u32)
            }
          } else {
            old_bank.balance_of(acct_id2, path2, coin2)
          }
        )
    }
}

trait Bank {

    #[pure]
    fn balance_of(&self, acct_id: AccountID, path: Path, coin: Coin) -> u32;

    #[requires(amt < 0 ==> ((0 - amt) as u32) <= self.balance_of(acct_id, path, coin))]
    #[ensures(
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          self.balance_of(acct_id2, path2, coin2) ==
          if(acct_id == acct_id2 && coin == coin2 && path === path2) {
            if(amt >= 0) {
                old(self.balance_of(acct_id, path, coin)) + (amt as u32)
            } else {
                old(self.balance_of(acct_id, path, coin)) - ((0 - amt) as u32)
            }
          } else {
            old(self.balance_of(acct_id2, path2, coin2))
          }
        )
    )]
    fn adjust_amount(&mut self, acct_id: AccountID, path: Path, coin: Coin, amt: i32) -> u32;

    #[pure]
    fn transfer_tokens_pre(&self, from: AccountID, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool {
        from != to && self.balance_of(from, path, coin) >= amt
    }

    #[requires(amt <= i32::MAX as u32)]
    #[ensures(result == old(self.transfer_tokens_pre(from, to, path, coin, amt)))]
    #[ensures(result ==>
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          self.balance_of(acct_id2, path2, coin2) ==
            if(acct_id2 == from && coin == coin2 && path === path2) {
                old(self).balance_of(from, path, coin) - amt
            } else if (acct_id2 == to && coin == coin2 && path === path2){
                old(self).balance_of(to, path, coin) + amt
            } else {
                old(self).balance_of(acct_id2, path2, coin2)
            }
        )
    )]
    fn transfer_tokens(
        &mut self,
        from: AccountID,
        to: AccountID,
        path: Path,
        coin: Coin,
        amt: u32
    ) -> bool {
        if(self.transfer_tokens_pre(from, to, path, coin, amt)) {
            self.adjust_amount(from, path, coin, 0 - (amt as i32));
            self.adjust_amount(to, path, coin, amt as i32);
            true
        } else {
            false
        }
    }

    #[requires(amt < i32::MAX as u32)]
    #[ensures(result == (old(self.balance_of(to, path, coin)) >= amt))]
    #[ensures(result ==> forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          self.balance_of(acct_id2, path2, coin2) ==
            if(acct_id2 == to && coin == coin2 && path === path2) {
                old(self).balance_of(to, path, coin) - amt
            } else {
                old(self).balance_of(acct_id2, path2, coin2)
            }
        )
    )]
    fn burn_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool {
        if(self.balance_of(to, path, coin) >= amt) {
            self.adjust_amount(to, path, coin, (0 - amt as i32));
            true
        } else {
            false
        }
    }

    #[requires(amt <= i32::MAX as u32)]
    #[ensures(result)]
    #[ensures(forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
          self.balance_of(acct_id2, path2, coin2) ==
            if(acct_id2 == to && coin == coin2 && path === path2) {
                old(self).balance_of(to, path, coin) + amt
            } else {
                old(self).balance_of(acct_id2, path2, coin2)
            }
        )
    )]
    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool {
        self.adjust_amount(to, path, coin, amt as i32);
        true
    }
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

#[requires(amount < i32::MAX as u32)]
#[ensures(
    old(send_will_burn(
       bank,
       path,
       source_port,
       source_channel,
       sender,
       coin,
       amount)) ==> (result == Some(
mk_packet(
            source_port,
            source_channel,
            FungibleTokenPacketData {
                path,
                coin,
                sender,
                receiver,
                amount
            })
    ) &&
        forall(|acct_id2: AccountID, coin2: Coin, path2: Path|
                bank.balance_of(acct_id2, path2, coin2) ==
                    if(acct_id2 == sender && coin == coin2 && path === path2) {
                        old(bank).balance_of(sender, path, coin) - amount
                    } else {
                        old(bank).balance_of(acct_id2, path2, coin2)
                    }
                )
))]
#[ensures(
    old(send_will_transfer(
       bank,
       path,
       source_port,
       source_channel,
       sender,
       ctx.escrow_address(source_channel),
       coin,
       amount)) ==> result == Some(
mk_packet(
            source_port,
            source_channel,
            FungibleTokenPacketData {
                path,
                coin,
                sender,
                receiver,
                amount
            })
    ) && transfer_post(
        bank,
        old(bank),
        sender,
        old(ctx.escrow_address(source_channel)),
        path,
        coin,
        amount
    )
)]
#[ensures(
    forall(
        |channel: ChannelEnd|
           ctx.escrow_address(channel) == old(ctx.escrow_address(channel))
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
        let escrow_address = ctx.escrow_address(source_channel);
        bank.transfer_tokens(
            sender,
            escrow_address,
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
        prusti_assert!(amount <= i32::MAX as u32);
        Some(mk_packet(source_port, source_channel, data))
    } else {
        None
    }
}

fn refund_tokens<B: Bank>(ctx: &Ctx, bank: &mut B, packet: Packet) {
    let FungibleTokenPacketData{ path, coin, sender, amount, ..} = packet.data;
    if !path.starts_with(packet.source_port, packet.source_channel) {
        let escrow_address = ctx.escrow_address(packet.source_channel);
        prusti_assume!(amount <= i32::MAX as u32);
        bank.transfer_tokens(
            escrow_address,
            sender,
            path,
            coin,
            amount
        );
    } else {
        prusti_assume!(amount <= i32::MAX as u32);
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
        result.success && adjusted(
            bank,
            old(bank),
            old(packet.data.receiver),
            old(packet.data.path.prepend_prefix(packet.dest_port, packet.dest_channel)),
            old(packet.data.coin),
            old(packet.data.amount) as i32
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
        )))
              ==> (result.success &&
          transfer_post(
              bank,
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
    prusti_assume!(amount <= i32::MAX as u32);
    let success = if packet_is_source(packet) {
        let escrow_address = ctx.escrow_address(packet.dest_channel);
        bank.transfer_tokens(
            escrow_address,
            receiver,
            path.drop_prefix(packet.source_port, packet.source_channel),
            coin,
            amount
        )
    } else {
        prusti_assume!(u32::MAX -
            bank.balance_of(
                receiver,
                path.prepend_prefix(packet.dest_port, packet.dest_channel),
                coin
            ) >= amount);
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

#[requires(amount < i32::MAX as u32)]
#[requires(!path.starts_with(source_port, source_channel))]
#[requires(bank1.transfer_tokens_pre(sender, ctx1.escrow_address(source_channel), path, coin, amount))]
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
    prusti_assert!(packet.is_some());
    let packet = packet.unwrap();
    prusti_assume!(packet.dest_port == dest_port);
    prusti_assume!(packet.dest_channel == dest_channel);

    let ack = on_recv_packet(ctx2, bank2, packet);
    prusti_assert!(ack.success);
    on_acknowledge_packet(ctx1, bank1, ack, packet);

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


pub fn main(){}
