#[cfg(feature="prusti")]
use prusti_contracts::*;

pub trait SplitResults: Iterator {
#[cfg_attr(feature="prusti_fast", trusted)]
    fn split_results<T, E>(self) -> (Vec<T>, Vec<E>)
    where
        Self: Iterator<Item = Result<T, E>> + Sized,
    {
        let mut oks = vec![];
        let mut errs = vec![];

        for item in self {
            match item {
                Ok(ok) => oks.push(ok),
                Err(err) => errs.push(err),
            }
        }

        (oks, errs)
    }
}

impl<T> SplitResults for T where T: Iterator {}
