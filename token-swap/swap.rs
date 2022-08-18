#![allow(dead_code, unused)]
use std::collections::HashMap;
use prusti_contracts::*;

type AccountId = u32;
type Address = u32;
type Channel = u32;
type CoinId = u32;
type Height = u32;
type Ics20Error = u32;
type Path = u32;
type Port = u32;

#[derive(Clone, Copy)]
pub struct Packet {
    data: FungibleTokenPacketData,
    dest_port: Port,
    dest_channel: Channel,
    source_port: Port,
    source_channel: Channel
}

#[derive(Clone, Copy)]
pub struct FungibleTokenPacketData {
    sender: AccountId,
    receiver: AccountId,
    denom: PrefixedCoin,
    amount: u32
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PrefixedCoin {
   path: Path,
   base: CoinId
}

pub struct Bank {
    accounts: HashMap<(AccountId, PrefixedCoin), u32>
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

    #[trusted]
    #[requires(
        forall(|acct_id: AccountId, cid: CoinId|
           (acct_id != account_add && acct_id != account_remove) || (cid != coin_id) ==>
               self.total_account_balance(acct_id, cid) == prev.total_account_balance(acct_id, cid)
        ))
    ]
    #[requires(
        self.total_account_balance(account_add, coin_id) ==
        prev.total_account_balance(account_add, coin_id) + amt
    )]
    #[requires(
        self.total_account_balance(account_remove, coin_id) ==
        prev.total_account_balance(account_remove, coin_id) - amt
    )]
    #[ensures(
        forall(|cid: CoinId| self.total_balance(coin_id) == prev.total_balance(coin_id))
    )]
    pub fn total_balance_move_lemma(&self,
      prev: &Bank,
      coin_id: CoinId,
      account_add: AccountId,
      account_remove: AccountId,
      amt: u32
    ) {}

    #[trusted]
    #[requires(prev.balance(acct, denom) >= amt)]
    #[requires(self.balance(acct, denom) == prev.balance(acct, denom) - amt)]
    #[requires(
        forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
            (acct_id != acct && !(coin_id === denom) ==>
                self.balance(acct_id, coin_id) == prev.balance(acct_id, coin_id))
    ))]
    #[ensures(self.total_account_balance(acct, denom.base) == prev.total_account_balance(acct, denom.base) - amt)]
    pub fn sub_denom_lemma(&self,
       prev: &Bank,
       acct: AccountId,
       denom: PrefixedCoin,
       amt: u32
    ) {}

    #[pure]
    #[trusted]
    pub fn total_balance(&self, coin_id: CoinId) -> u32 {
        let mut result = 0;
        for ((_, denom), amt) in &self.accounts {
            if denom.base == coin_id {
                result += amt;
            }
        }
        return result;
    }

    #[pure]
    #[trusted]
    pub fn total_account_balance(&self, account_id: AccountId, coin_id: CoinId) -> u32 {
        let mut result = 0;
        for ((acct, denom), amt) in &self.accounts {
            if *acct == account_id && denom.base == coin_id {
                result += amt;
            }
        }
        return result;
    }

    #[pure]
    #[trusted]
    pub fn balance(&self, account_id: AccountId, denom: PrefixedCoin) -> u32 {
        *self.accounts.get(&(account_id, denom)).unwrap_or(&0)
    }

    #[requires(self.balance(acct1, coin) >= amt)]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    pub fn send_coins_involution(
        &mut self,
        acct1: AccountId,
        acct2: AccountId,
        coin: PrefixedCoin,
        amt: u32) -> Result<(), Ics20Error> {
        let err = self.send_coins(acct1, acct2, coin, amt);
        if !err.is_ok() {
            return err;
        }
        self.send_coins(acct2, acct1, coin, amt)
    }

    predicate! {
        pub fn send_coins_post(
            &self,
            old_self: &Self,
            from: AccountId,
            to: AccountId,
            denom: PrefixedCoin,
            amount: u32
        ) -> bool {
        forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
            (acct_id != from && !(acct_id === to)) || !(coin_id === denom) ==>
                self.balance(acct_id, coin_id) == old_self.balance(acct_id, coin_id)) &&
        forall(
                    |coin_id: PrefixedCoin| coin_id === denom ==>
                        self.balance(to, coin_id) == old_self.balance(to, coin_id) + amount &&
                        self.balance(from, coin_id) == old_self.balance(from, coin_id) - amount)
        }
    }

    #[ensures(
        result.is_ok() ==> self.send_coins_post(old(self), from, to, denom, amount)
    )]
    #[trusted]
    pub fn send_coins(
        &mut self,
        from: AccountId,
        to: AccountId,
        denom: PrefixedCoin,
        amount: u32,
    ) -> Result<(), Ics20Error>{
        unimplemented!()
    }

    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    pub fn mint_burn_involution(&mut self, acct: AccountId, coin: PrefixedCoin, amount: u32) -> Result<(), Ics20Error>{
        let err = self.mint_coins(acct, coin, amount);
        if !err.is_ok() {
           return err;
        }
        self.burn_coins(acct, coin, amount)
    }

    /// This function to enable minting ibc tokens to a user account
    #[ensures(
        result.is_ok() ==> self.balance(account, old(coin)) == old(self).balance(account, old(coin)) + amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
            !(acct_id === account) || !(coin_id === old(coin)) ==>
                self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    #[trusted]
    pub fn mint_coins(
        &mut self,
        account: AccountId,
        coin: PrefixedCoin,
        amount: u32
    ) -> Result<(), Ics20Error> {
        Ok(())
    }

    /// This function should enable burning of minted tokens in a user account
    #[ensures(result.is_ok() == (self.balance(account, old(coin)) == amount))]
    #[ensures(
        result.is_ok() ==> self.balance(account, old(coin)) == old(self).balance(account, old(coin)) - amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedCoin|
            !(acct_id === account) || !(coin_id === old(coin)) ==>
                self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    #[trusted]
    pub fn burn_coins(
        &mut self,
        account: AccountId,
        coin: PrefixedCoin,
        amount: u32
    ) -> Result<(), Ics20Error> {
        Ok(())
    }
}

struct App {
   bank: Bank
}

#[pure]
#[trusted]
pub fn is_prefix(source_port: Port, channel: Channel, denomination: PrefixedCoin) -> bool {
    unimplemented!()
}

#[pure]
#[trusted]
pub fn drop_prefix(source_port: Port, channel: Channel, denomination: PrefixedCoin) -> PrefixedCoin {
    unimplemented!()
}

#[pure]
#[trusted]
pub fn with_prefix(source_port: Port, channel: Channel, denomination: PrefixedCoin) -> PrefixedCoin {
    unimplemented!()
}

#[pure]
#[trusted]
pub fn channel_escrow_addresses(source_channel: Channel) -> Address {
    unimplemented!()
}

impl App {
    #[ensures(
        result.is_ok() ==> old(self.bank.balance(sender, denomination)) >= amount
    )]
    #[ensures(
        result.is_ok() && is_prefix(source_port, source_channel, denomination) ==>
            self.bank.send_coins_post(
                old(&self.bank),
                sender,
                channel_escrow_addresses(source_channel),
                denomination,
                amount
            )
    )]
    pub fn send_fungible_tokens(
        &mut self,
        denomination: PrefixedCoin,
        amount: u32,
        sender: AccountId,
        receiver: AccountId,
        source_port: Port,
        source_channel: Channel
    ) -> Result<FungibleTokenPacketData, Ics20Error> {
        let source = is_prefix(source_port, source_channel, denomination);
        let result = if source {
            let escrow_account = channel_escrow_addresses(source_channel);
            self.bank.send_coins(
                sender,
                escrow_account,
                denomination,
                amount
            )
        } else {
            self.bank.burn_coins(
                sender,
                denomination,
                amount
            )
        };
        if !result.is_ok() {
            return Err(0);
        } else {
            return Ok(FungibleTokenPacketData {
                sender,
                receiver,
                denom: denomination,
                amount
            });
        }
    }

    pub fn on_recv_packet(
        &mut self,
        packet: Packet
     ) -> bool {
        let data = packet.data;
        let source = is_prefix(packet.source_port, packet.source_channel, data.denom);
        if source {
            let escrow_account = channel_escrow_addresses(packet.dest_channel);
            let send_result = self.bank.send_coins(
                escrow_account,
                data.receiver,
                drop_prefix(packet.source_port, packet.source_channel, data.denom),
                data.amount
            );
            return send_result.is_ok();
        } else {
            let prefixed = with_prefix(
                packet.dest_port,
                packet.dest_channel,
                data.denom
            );
            let mint_result = self.bank.mint_coins(
                data.receiver,
                prefixed,
                data.amount
            );
            return mint_result.is_ok();
        }
    }

    #[requires(
        is_prefix(packet.source_port, packet.source_channel, packet.data.denom)
               ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    pub fn refund_tokens(
        &mut self,
        packet: Packet
    ) {
        let data = packet.data;
        let source = is_prefix(packet.source_port, packet.source_channel, data.denom);
        if source {
            let escrow_account = channel_escrow_addresses(packet.source_channel);
            let send_result = self.bank.send_coins(
                escrow_account,
                data.sender,
                data.denom,
                data.amount
            );
        } else {
            self.bank.mint_coins(
                data.sender,
                data.denom,
                data.amount
            );
        }
    }

    #[requires(
        (!success && is_prefix(packet.source_port, packet.source_channel, packet.data.denom))
               ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    pub fn on_acknowledge_packet(&mut self, packet: Packet, success: bool) {
        if !success {
           self.refund_tokens(packet);
        }
    }

    #[requires(
        is_prefix(packet.source_port, packet.source_channel, packet.data.denom)
          ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    pub fn on_timeout_packet(&mut self, packet: Packet) {
        self.refund_tokens(packet);
    }

    // This function cannot be called, because channel is unordered.
    #[requires(false)]
    pub fn on_timeout_packet_close(packet: Packet) {

    }

    #[ensures(
        forall(
            |coin_id: CoinId|
                (&self.bank).total_balance(coin_id) == old(&self.bank).total_balance(coin_id)
    ))]
    pub fn example_run(
        &mut self,
        denomination: PrefixedCoin,
        amount: u32,
        sender: AccountId,
        receiver: AccountId,
        source_port: Port,
        source_channel: Channel,
        dest_port: Port,
        dest_channel: Channel,
    ){
        let send_result = self.send_fungible_tokens(
            denomination,
            amount,
            sender,
            receiver,
            source_port,
            source_channel
        );
        if !send_result.is_ok() {
            return
        }

        assert!(old(self.bank.balance(sender, denomination)) >= amount);

        // The total quantity of denomination.base for `sender` has been decreased by amount
        self.bank.sub_denom_lemma(old(&self.bank), sender, denomination, amount);


        assert!(old(self.bank.total_account_balance(sender, denomination.base)) >= amount);
        assert!(
            self.bank.total_account_balance(sender, denomination.base) ==
                old(self.bank.total_account_balance(sender, denomination.base)) - amount);
    }
}

pub fn main(){}
