use flex_error::define_error;
use prusti_contracts::*;

define_error! {
    Error {
        UnknownPort
            | _ | { format_args!("port unknown") }
    }
}
