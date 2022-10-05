#![feature(box_patterns)]
#![allow(dead_code, unused)]
use std::convert::TryInto;
use prusti_contracts::*;

type AccountID = u32;

// #[derive(Copy, Clone, Eq, PartialEq, Hash)]
// struct AccountID(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Coin(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ChannelEnd(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Port(u32);

#[derive(Eq, PartialEq, Hash)]
#[invariant(self.amount <= i32::MAX as u32)]
struct FungibleTokenPacketData {
    path: Path,
    coin: Coin,
    sender: AccountID,
    receiver: AccountID,
    amount: u32
}

#[derive(Eq, PartialEq, Hash)]
struct Packet {
    source_port: Port,
    source_channel: ChannelEnd,
    dest_port: Port,
    dest_channel: ChannelEnd,
    data: FungibleTokenPacketData
}

#[trusted]
fn mk_packet(
    source_port: Port,
    source_channel: ChannelEnd,
    data: FungibleTokenPacketData
) -> Packet {
    unimplemented!()
}

#[derive(Copy, Clone, Eq, Hash)]
struct Path(u32);

// #[derive(Clone, Eq, Hash)]
// enum Path {
//     Empty(),
//     Cons {
//         head_port: Port,
//         head_channel: ChannelEnd,
//         tail: Box<Path>
//     }
// }

impl Path {
    #[pure]
    #[trusted]
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
    #[trusted]
    fn drop_prefix(&self, port: Port, channel: ChannelEnd) -> Path {
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
    fn balance_of(&self, acct_id: AccountID, path: &Path, coin: &Coin) -> u32;

    #[pure]
    fn escrow_address(&self, channel: ChannelEnd) -> AccountID;

    #[requires(amt >= 0 ==> u32::MAX - self.balance_of(acct_id, path, coin) >= (amt as u32))]
    #[requires(amt < 0 ==> ((0 - amt) as u32) <= self.balance_of(acct_id, path, coin))]
    #[ensures(
        forall(|acct_id2: AccountID, coin2: &Coin, path2: &Path|
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
    fn adjust_amount(&mut self, acct_id: AccountID, path: &Path, coin: &Coin, amt: i32) -> u32;

    #[pure]
    fn bank_transfer_tokens_pre(&self, from: AccountID, to: AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
        from != to && self.balance_of(from, path, coin) >= amt
    }

    #[requires(amt <= i32::MAX as u32)]
    #[requires(u32::MAX - self.balance_of(to, path, coin) >= amt)]
    #[requires(from != to)]
    fn transfer_tokens(&mut self, from: AccountID, to: AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
        if(self.bank_transfer_tokens_pre(from, to, path, coin, amt)) {
            self.adjust_amount(from, path, coin, 0 - (amt as i32));
            self.adjust_amount(to, path, coin, amt as i32);
            true
        } else {
            false
        }
    }

    #[requires(amt < i32::MAX as u32)]
    #[ensures(result == (old(self.balance_of(to, path, coin)) >= amt))]
    fn burn_tokens(&mut self, to: AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
        if(self.balance_of(to, path, coin) >= amt) {
            self.adjust_amount(to, path, coin, (0 - amt as i32));
            true
        } else {
            false
        }
    }

    #[requires(amt <= i32::MAX as u32)]
    #[requires(u32::MAX - self.balance_of(to, path, coin) >= amt)]
    #[ensures(result)]
    fn mint_tokens(&mut self, to: AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
        self.adjust_amount(to, path, coin, amt as i32);
        true
    }
}


#[requires(amount < i32::MAX as u32)]
#[requires(u32::MAX - bank.balance_of(bank.escrow_address(source_channel), path, coin) >= amount)]
#[requires(sender != bank.escrow_address(source_channel))]
fn send_fungible_tokens(
    bank: &mut dyn Bank,
    path: &Path,
    coin: &Coin,
    amount: u32,
    sender: AccountID,
    receiver: AccountID,
    source_port: Port,
    source_channel: ChannelEnd
) -> Option<Packet> {
    let success = if(!path.starts_with(source_port, source_channel)) {
        let escrow_address = bank.escrow_address(source_channel);
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
            path: path.clone(),
            coin: *coin,
            sender,
            receiver,
            amount
        };
        Some(mk_packet(source_port, source_channel, data))
    } else {
        None
    }
}

#[requires(
    u32::MAX - bank.balance_of(packet.data.sender, &packet.data.path, &packet.data.coin) >= packet.data.amount)
]
#[requires(bank.escrow_address(packet.source_channel) != packet.data.sender)]
fn refund_tokens<B: Bank>(bank: &mut B, packet: Packet) {
    let FungibleTokenPacketData{ path, coin, sender, amount, ..} = packet.data;
    if !path.starts_with(packet.source_port, packet.source_channel) {
        let escrow_address = bank.escrow_address(packet.source_channel);
        bank.transfer_tokens(
            escrow_address,
            sender,
            &path,
            &coin,
            amount
        );
    } else {
        bank.mint_tokens(
            sender,
            &path,
            &coin,
            amount
        );
    }
}

struct FungibleTokenPacketAcknowledgement {
    success: bool
}

#[requires(
    packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
    u32::MAX - bank.balance_of(
        packet.data.receiver,
        &packet.data.path.drop_prefix(packet.source_port, packet.source_channel),
        &packet.data.coin
    ) >= packet.data.amount)]
#[requires(bank.escrow_address(packet.dest_channel) != packet.data.receiver)]
#[requires(
    !packet.data.path.starts_with(packet.source_port, packet.source_channel) ==>
    u32::MAX - bank.balance_of(
        packet.data.receiver,
        &packet.data.path.prepend_prefix(packet.dest_port, packet.dest_channel),
        &packet.data.coin
    ) >= packet.data.amount)]
fn on_recv_packet(bank: &mut dyn Bank, packet: Packet) {
    let FungibleTokenPacketData{ path, coin, receiver, amount, ..} = packet.data;
    if path.starts_with(packet.source_port, packet.source_channel) {
        let escrow_address = bank.escrow_address(packet.dest_channel);
        bank.transfer_tokens(
            escrow_address,
            receiver,
            &path.drop_prefix(packet.source_port, packet.source_channel),
            &coin,
            amount
        );
    } else {
        bank.mint_tokens(
            receiver,
            &path.prepend_prefix(packet.dest_port, packet.dest_channel),
            &coin,
            amount
        );
    }
}

#[requires(
    u32::MAX - bank.balance_of(packet.data.sender, &packet.data.path, &packet.data.coin) >= packet.data.amount)
]
#[requires(bank.escrow_address(packet.source_channel) != packet.data.sender)]
fn on_acknowledge_packet<B: Bank>(
    bank: &mut B,
    ack: FungibleTokenPacketAcknowledgement,
    packet: Packet) {
    if(!ack.success) {
        refund_tokens(bank, packet);
    }
}


pub fn main(){}
