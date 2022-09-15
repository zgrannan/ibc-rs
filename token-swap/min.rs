use prusti_contracts::*;

struct Foo {
    pub inner: u32
}

impl Foo {

    #[pure]
    fn bar(&self) -> u32 {
        self.inner
    }

    #[requires(i >= 0 ==> u32::MAX - self.bar() >= i as u32)]
    #[ensures(result == if i >= 0 { old(self.bar()) + (i as u32)  } else {0})]
    fn convert(&mut self, i: i32) -> u32 {
        let old_inner = self.inner;
        self.inner = u32::MAX;
        if(i >= 0) {
            old_inner + (i as u32)
        } else {
            0
        }
    }
}

#[ensures(result == x + y)]
#[trusted]
fn add(x: u32, y: u32) -> u32 {
    unimplemented!()
}

#[requires(y >= 0 ==> u32::MAX - foo.bar() >= y as u32)]
fn go(foo: &mut Foo, y: i32) {
    foo.convert(y);
}

fn main(){}
