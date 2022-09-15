#![feature(box_patterns)]
#![allow(dead_code, unused)]
use std::convert::TryInto;
use prusti_contracts::*;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct AccountID(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Coin(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct ChannelEnd(u32);
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Port(u32);

// #[extern_spec]
// impl std::boxed::Box {
//     #[pure]
//     fn new<T>(t: T) -> Box<T>;
// }

#[derive(Clone, Eq, Hash)]
enum Path {
    Empty(),
    Cons {
        head_port: Port,
        head_channel: ChannelEnd,
        tail: Box<Path>
    }
}

impl Path {
    #[pure]
    #[trusted]
    fn prepend_prefix(&self, port: Port, channel: ChannelEnd) -> &Path {
        unimplemented!()
    }

    #[pure]
    fn starts_with(&self, port: &Port, channel: &ChannelEnd) -> bool {
        match self {
            Path::Empty() => false,
            Path::Cons{ head_port, head_channel, .. } => port == head_port && channel == head_channel
        }
    }

    #[pure]
    #[requires(self.starts_with(port, channel))]
    #[trusted]
    fn drop_prefix(&self, port: &Port, channel: &ChannelEnd) -> &Path {
        match self {
            Path::Empty() => unreachable!(),
            Path::Cons{ box tail, .. } => tail
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

trait Bank {

    #[pure]
    fn balance_of(&self, acct_id: &AccountID, path: &Path, coin: &Coin) -> u32;

    #[requires(amt >= 0 ==> u32::MAX - self.balance_of(acct_id, path, coin) >= (amt as u32))]
    #[requires(amt < 0 ==> ((0 - amt) as u32) <= self.balance_of(acct_id, path, coin))]
    #[ensures(
        forall(|acct_id2: &AccountID, coin2: &Coin, path2: &Path|
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
    fn adjust_amount(&mut self, acct_id: &AccountID, path: &Path, coin: &Coin, amt: i32) -> u32;

}

#[requires(amt < i32::MAX as u32)]
#[ensures(result)]
fn mint_tokens(bank: &mut dyn Bank, to: &AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
    bank.adjust_amount(to, path, coin, amt as i32);
    true
}

#[requires(amt < i32::MAX as u32)]
#[ensures(result == (bank.balance_of(to, path, coin) >= amt))]
fn burn_tokens(bank: &mut dyn Bank, to: &AccountID, path: &Path, coin: &Coin, amt: u32) -> bool {
    if(bank.balance_of(to, path, coin) >= amt) {
        bank.adjust_amount(to, path, coin, (0 - amt as i32));
        true
    } else {
        false
    }
}

pub fn main(){}
