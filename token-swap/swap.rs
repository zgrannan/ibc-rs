#![allow(dead_code, unused)]
use std::collections::HashMap;
use std::convert::TryInto;
use prusti_contracts::*;

#[derive(Eq, PartialEq, Hash)]
struct AccountID(u32);
#[derive(Eq, PartialEq, Hash)]
struct Coin(u32);
#[derive(Eq, PartialEq, Hash)]
struct ChannelEnd(u32);
#[derive(Eq, PartialEq, Hash)]
struct Port(u32);

#[derive(Eq, Hash)]
enum Path {
    Empty(),
    Cons {
        port: Port,
        channel_end: ChannelEnd,
        tail: Box<Path>
    }
}

// Prusti does not like derived PartialEQ
impl PartialEq for Path {
    #[trusted]
    fn eq(&self, other: &Path) -> bool {
        unimplemented!()
    }

}


struct Bank {
    balances: HashMap<AccountID, HashMap<Coin,HashMap<Path, u32>>>
}

trait Bank {

    #[pure]
    #[trusted]
    fn balance(&self, acct_id: &AccountID, path: &Path, coin: &Coin) -> u32 {
        let acct_map = match self.balances.get(&acct_id) {
            Some(m) => m,
            None => return 0
        };
        let coin_map = match acct_map.get(&coin) {
            Some(m) => m,
            None => return 0
        };
        match coin_map.get(&path) {
            Some(amt) => *amt,
            None => 0
        }
    }

    #[requires(amt >= -4294967295)]
    #[requires(amt <= 4294967295)]
    fn update_balance(&mut self, acct_id: AccountID, path: Path, coin: Coin, amt: i64) -> bool {
        let mut acct_map = self.balances.remove(&acct_id).unwrap_or(HashMap::new());
        let mut coin_map = acct_map.remove(&coin).unwrap_or(HashMap::new());
        let old_value = coin_map.remove(&path).unwrap_or(0);
        let (success, new_value) = match ((old_value as i64) + amt).try_into() {
           Ok(nv) => (true, nv),
           Err(_) => (false, old_value)
        };
        coin_map.insert(path, new_value);
        acct_map.insert(coin, coin_map);
        self.balances.insert(acct_id, acct_map);
        success
    }

    fn mint_tokens(&mut self, to: AccountID, path: Path, coin: Coin, amt: u32) -> bool {
        self.update_balance(to, path, coin, amt as i64)
    }
}

pub fn main(){}
