#![allow(dead_code, unused)]
use std::collections::HashMap;
use prusti_contracts::*;

#[derive(Eq, PartialEq, Hash)]
struct AccountID(u32);
#[derive(Eq, PartialEq, Hash)]
struct Coin(u32);
#[derive(Eq, PartialEq, Hash)]
struct ChannelEnd(u32);
#[derive(Eq, PartialEq, Hash)]
struct Port(u32);

#[derive(Eq, PartialEq, Hash)]
enum Path {
    Empty(),
    Cons {
        port: Port,
        channel_end: ChannelEnd,
        tail: Box<Path>
    }
}


struct Bank {
    balances: HashMap<AccountID, HashMap<Coin,HashMap<Path, u32>>>
}

impl Bank {

    fn update_balance(&mut self, acct_id: AccountID, path: Path, coin: Coin, amt: i32) {
        let mut acct_map = self.balances.remove(&acct_id).unwrap_or(HashMap::new());
        let mut coin_map = acct_map.remove(&coin).unwrap_or(HashMap::new());
        let old_value = coin_map.remove(&path).unwrap_or(0);
        coin_map.insert(path, ((old_value as i32) + amt) as u32);
    }

    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: i32) -> bool {
        true
    }
}

pub fn main(){}
