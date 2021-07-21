use std::{convert::Infallible, fmt::Display, str::FromStr};

use prusti_contracts::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Signer(String);

impl Signer {
    pub fn new(s: impl ToString) -> Self {
panic!("No") //         Self(s.to_string())
    }

#[trusted]
    pub fn as_str(&self) -> &str {
panic!("No") //         &self.0
    }
}

impl Display for Signer {
#[trusted]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Signer {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl FromStr for Signer {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
