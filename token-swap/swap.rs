use std::collections::HashSet;
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
struct Packet {
    data: FungibleTokenPacketData,
    dest_port: Port,
    dest_channel: Channel,
    source_port: Port,
    source_channel: Channel
}

#[derive(Clone, Copy)]
struct FungibleTokenPacketData {
    sender: AccountId,
    receiver: AccountId,
    denom: PrefixedDenom,
    amount: u32
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct PrefixedDenom {
   path: Path,
   base: CoinId
}

#[derive(Clone, Copy)]
struct PrefixedCoin {
    denom: PrefixedDenom,
    amount: u32
}

struct Bank {
    accounts: HashMap<(AccountId, PrefixedDenom), u32>
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
    fn total_balance_move_lemma(&self,
      prev: &Bank,
      coin_id: CoinId,
      account_add: AccountId,
      account_remove: AccountId,
      amt: u32
    ) {}

    fn sub_denom_lemma(&self,
       prev: &Bank,
       denom: PrefixedDenom
    ) {}

    #[pure]
    #[trusted]
    fn total_balance(&self, coin_id: CoinId) -> u32 {
        let mut result = 0;
        for (((_, denom), amt)) in &self.accounts {
            if(denom.base == coin_id) {
                result += amt;
            }
        }
        return result;
    }

    #[pure]
    #[trusted]
    fn total_account_balance(&self, account_id: AccountId, coin_id: CoinId) -> u32 {
        let mut result = 0;
        for (((acct, denom), amt)) in &self.accounts {
            if(*acct == account_id && denom.base == coin_id) {
                result += amt;
            }
        }
        return result;
    }

    #[pure]
    #[trusted]
    fn balance(&self, account_id: AccountId, denom: PrefixedDenom) -> u32 {
        *self.accounts.get(&(account_id, denom)).unwrap_or(&0)
    }

    #[requires(self.balance(acct1, amt.denom) >= amt.amount)]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedDenom|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    fn send_coins_involution(&mut self, acct1: AccountId, acct2: AccountId, amt: PrefixedCoin) -> Result<(), Ics20Error> {
        let err = self.send_coins(acct1, acct2, amt.denom, amt.amount);
        if !err.is_ok() {
            return err;
        }
        self.send_coins(acct2, acct1, amt.denom, amt.amount)
    }

    predicate! {
        fn send_coins_post(
            &self,
            old_self: &Self,
            from: AccountId,
            to: AccountId,
            denom: PrefixedDenom,
            amount: u32
        ) -> bool {
        forall(
            |acct_id: AccountId, coin_id: PrefixedDenom|
            (acct_id != from && !(acct_id === to)) || !(coin_id === denom) ==>
                self.balance(acct_id, coin_id) == old_self.balance(acct_id, coin_id)) &&
        forall(
                    |coin_id: PrefixedDenom| coin_id === denom ==>
                        self.balance(to, coin_id) == old_self.balance(to, coin_id) + amount &&
                        self.balance(from, coin_id) == old_self.balance(from, coin_id) - amount)
        }
    }

    #[ensures(
        result.is_ok() ==> self.send_coins_post(old(self), from, to, denom, amount)
    )]
    #[trusted]
    fn send_coins(
        &mut self,
        from: AccountId,
        to: AccountId,
        denom: PrefixedDenom,
        amount: u32,
    ) -> Result<(), Ics20Error>{
        unimplemented!()
    }

    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedDenom|
               self.balance(acct_id, coin_id) == old(self).balance(acct_id, coin_id))
    )]
    fn mint_burn_involution(&mut self, acct: AccountId, amt: PrefixedCoin) -> Result<(), Ics20Error>{
        let err = self.mint_coins(acct, amt);
        if !err.is_ok() {
           return err;
        }
        self.burn_coins(acct, amt)
    }

    /// This function to enable minting ibc tokens to a user account
    #[ensures(
        result.is_ok() ==> self.balance(account, old(amt.denom)) == old(self).balance(account, old(amt.denom)) + amt.amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedDenom|
            !(acct_id === account) || !(coin_id === old(amt.denom)) ==>
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
    #[ensures(result.is_ok() == (self.balance(account, old(amt.denom)) == amt.amount))]
    #[ensures(
        result.is_ok() ==> self.balance(account, old(amt.denom)) == old(self).balance(account, old(amt.denom)) - amt.amount
    )]
    #[ensures(
        result.is_ok() ==> forall(
            |acct_id: AccountId, coin_id: PrefixedDenom|
            !(acct_id === account) || !(coin_id === old(amt.denom)) ==>
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

struct App {
   bank: Bank
}

#[pure]
#[trusted]
fn is_prefix(source_port: Port, channel: Channel, denomination: PrefixedDenom) -> bool {
    unimplemented!()
}

#[pure]
#[trusted]
fn drop_prefix(source_port: Port, channel: Channel, denomination: PrefixedDenom) -> PrefixedDenom {
    unimplemented!()
}

#[pure]
#[trusted]
fn with_prefix(source_port: Port, channel: Channel, denomination: PrefixedDenom) -> PrefixedDenom {
    unimplemented!()
}

#[pure]
#[trusted]
fn channel_escrow_addresses(source_channel: Channel) -> Address {
    unimplemented!()
}

impl App {
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
    fn send_fungible_tokens(
        &mut self,
        denomination: PrefixedDenom,
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
                PrefixedCoin {
                    denom: denomination,
                    amount
                }
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

    fn on_recv_packet(
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
                PrefixedCoin {
                   denom: prefixed,
                   amount: data.amount
                }
            );
            return mint_result.is_ok();
        }
    }

    #[requires(
        is_prefix(packet.source_port, packet.source_channel, packet.data.denom)
               ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    fn refund_tokens(
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
                PrefixedCoin { denom: data.denom, amount: data.amount }
            );
        }
    }

    #[requires(
        (!success && is_prefix(packet.source_port, packet.source_channel, packet.data.denom))
               ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    fn on_acknowledge_packet(&mut self, packet: Packet, success: bool) {
        if !success {
           self.refund_tokens(packet);
        }
    }

    #[requires(
        is_prefix(packet.source_port, packet.source_channel, packet.data.denom)
          ==> self.bank.balance(packet.data.sender, packet.data.denom) >= packet.data.amount)
    ]
    fn on_timeout_packet(&mut self, packet: Packet) {
        self.refund_tokens(packet);
    }

    // This function cannot be called, because channel is unordered.
    #[requires(false)]
    fn on_timeout_packet_close(packet: Packet) {

    }

    #[ensures(
        forall(
            |coin_id: CoinId|
                (&self.bank).total_balance(coin_id) == old(&self.bank).total_balance(coin_id)
    ))]
    fn example_run(
        &mut self,
        denomination: PrefixedDenom,
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
        assert!(old(self.bank.total_account_balance(sender, denomination.base)) >= amount);
        assert!(
            self.bank.total_account_balance(sender, denomination.base) ==
                old(self.bank.total_account_balance(sender, denomination.base)) - amount);
    }
}

fn main(){}
