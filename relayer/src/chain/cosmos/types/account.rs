use core::fmt;
use ibc_proto::cosmos::auth::v1beta1::BaseAccount;
/// Wrapper for account number and sequence number.
///
/// More fields may be added later.
#[derive(Clone, Debug, PartialEq)]
pub struct Account {
    pub number: AccountNumber,
    pub sequence: AccountSequence,
}
impl From<BaseAccount> for Account {
    #[prusti_contracts::trusted]
    fn from(value: BaseAccount) -> Self {
        Self {
            number: AccountNumber::new(value.account_number),
            sequence: AccountSequence::new(value.sequence),
        }
    }
}
/// Newtype for account numbers
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountNumber(u64);
impl AccountNumber {
    #[prusti_contracts::trusted]
    pub fn new(number: u64) -> Self {
        Self(number)
    }
    #[prusti_contracts::trusted]
    pub fn to_u64(self) -> u64 {
        self.0
    }
}
impl fmt::Display for AccountNumber {
    #[prusti_contracts::trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
/// Newtype for account sequence numbers
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AccountSequence(u64);
impl AccountSequence {
    #[prusti_contracts::trusted]
    pub fn new(sequence: u64) -> Self {
        Self(sequence)
    }
    #[prusti_contracts::trusted]
    pub fn to_u64(self) -> u64 {
        self.0
    }
    #[prusti_contracts::trusted]
    pub fn increment(self) -> Self {
        Self(self.0 + 1)
    }
    #[prusti_contracts::trusted]
    pub fn increment_mut(&mut self) {
        self.0 += 1;
    }
}
impl fmt::Display for AccountSequence {
    #[prusti_contracts::trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

