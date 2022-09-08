
fn path_eq(path: &Path, other: &Path) -> bool {
        match (path, other) {
            (Path::Cons(a1, b1), Path::Cons(a2, b2)) =>
                *a1 == *a2 && *b1 == *b2,
            _ => true,
        }
}

// #[derive(PartialEq)]
enum Path {
    Empty(),
    Cons(u32, u32)
}

fn main(){}
