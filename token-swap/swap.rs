use std::collections::HashSet;
use std::collections::HashMap;
use prusti_contracts::*;
type CoinId = u32;
type AccountId = u32;
type Ics20Error = u32;

#[derive(Clone, Copy)]
struct PrefixedCoin {
    denom: CoinId,
    amount: u32
}

struct Bank {
    accounts: HashMap<AccountId, HashSet<PrefixedCoin>>
}

#[extern_spec]
impl<T: PartialEq, U: std::fmt::Debug + PartialEq> std::result::Result<T, U> {
    #[pure]
    #[ensures(matches!(&self, Ok(_)) == result)]
    pub fn is_ok(&self) -> bool;

    #[pure]
    #[requires(self.is_ok())]
    #[ensures(self == Ok(result))]
    pub fn unwrap(self) -> T;
}

impl Bank {

    #[pure]
    #[trusted]
    fn balance(&self, accountId: AccountId, coin_id: CoinId) -> u32 {
        unimplemented!()
    }

    #[requires(self.balance(acct1, amt.denom) >= amt.amount)]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: CoinId|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    fn send_coins_involution(&mut self, acct1: AccountId, acct2: AccountId, amt: PrefixedCoin) -> Result<(), Ics20Error> {
        let err = self.send_coins(acct1, acct2, amt);
        if(!err.is_ok()){
            return err;
        }
        self.send_coins(acct2, acct1, amt)
    }

    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: CoinId|
            (acct_id != from && acct_id != to) || coin_id != amt.denom ==>
                self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    #[ensures(
        result.is_ok() ==>
            forall(
            |coin_id: CoinId| coin_id == amt.denom ==>
                self.balance(to, coin_id) == old(&self).balance(to, coin_id) + amt.amount &&
                self.balance(from, coin_id) == old(&self).balance(from, coin_id) - amt.amount)

    )]
    #[trusted]
    fn send_coins(
        &mut self,
        from: AccountId,
        to: AccountId,
        amt: PrefixedCoin,
    ) -> Result<(), Ics20Error>{
        unimplemented!()
    }

    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: CoinId|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    fn mint_burn_involution(&mut self, acct: AccountId, amt: PrefixedCoin) -> Result<(), Ics20Error>{
        let err = self.mint_coins(acct, amt);
        if(!err.is_ok()){
           return err;
        }
        self.burn_coins(acct, amt)
    }

    /// This function to enable minting ibc tokens to a user account
    #[ensures(
        result.is_ok() ==> self.balance(account, amt.denom) == old(self).balance(account, amt.denom) + amt.amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: CoinId|
            (acct_id != account) || coin_id != amt.denom ==>
                self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    #[trusted]
    fn mint_coins(
        &mut self,
        account: AccountId,
        amt: PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    /// This function should enable burning of minted tokens in a user account
    #[requires(self.balance(account, amt.denom) >= amt.amount)]
    #[ensures(
        result.is_ok() ==> self.balance(account, amt.denom) == old(self).balance(account, amt.denom) - amt.amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: CoinId|
            (acct_id != account) || coin_id != amt.denom ==>
                self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    #[trusted]
    fn burn_coins(
        &mut self,
        account: AccountId,
        amt: PrefixedCoin,
    ) -> Result<(), Ics20Error> {
        Ok(())
    }
}

fn main(){}
