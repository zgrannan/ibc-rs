use prusti_contracts::*;

#[derive(Copy, Clone, PartialEq, Eq)]
struct AccountID(u32);

#[pure]
fn check_eq(from: &AccountID) -> bool {
    from != from
}

fn go(from: &AccountID) -> bool {
    check_eq(from)
}

pub fn main(){}
