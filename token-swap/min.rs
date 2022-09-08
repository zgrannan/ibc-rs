#[derive(PartialEq)]
enum Path {
    Empty(),
    Cons {
        port: u32,
        channel_end: u32
    }
}

fn main(){}
