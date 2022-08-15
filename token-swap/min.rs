use prusti_contracts::*;

#[derive(Clone, Copy)]
struct Inner(u32);

#[derive(Clone, Copy)]
struct Outer {
    inner: Inner,
}

#[ensures(out.inner === out.inner)]
fn go(out: Outer) {
}


fn main(){}
