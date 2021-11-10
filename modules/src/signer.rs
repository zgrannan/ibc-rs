use std::{convert::Infallible, fmt::Display, str::FromStr};

#[cfg(feature="prusti")]
use prusti_contracts::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Hash)]
#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti", derive(PrustiDebug))]
pub struct Signer(String);

impl Signer {
#[cfg_attr(feature="prusti", trusted)]
    pub fn new(s: impl ToString) -> Self {
unreachable!() //         Self(s.to_string())
    }

// #[cfg_attr(feature="prusti", trusted)]
//     pub fn as_str(&self) -> &str {
// unreachable!() //         &self.0
//     }
}

impl Display for Signer {
#[cfg_attr(feature="prusti", trusted)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
unreachable!() //         write!(f, "{}", self.0)
    }
}

impl From<String> for Signer {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for Signer {
    type Err = Infallible;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
