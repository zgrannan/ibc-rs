use std::cmp::Ordering;
use std::convert::TryFrom;
#[cfg(feature="prusti")]
use prusti_contracts::*;
use std::num::ParseIntError;
use std::str::FromStr;

use flex_error::{define_error, TraceError};
use serde_derive::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use ibc_proto::ibc::core::client::v1::Height as RawHeight;

use crate::ics02_client::error::Error;

#[derive(Eq, PartialEq, Copy, Clone, Hash)]
#[cfg_attr(not(feature="prusti"), derive(Deserialize), derive(Serialize))]
pub struct Height {
    /// Previously known as "epoch"
    pub revision_number: u64,

    /// The height of a block
    pub revision_height: u64,
}

impl Height {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn new(revision_number: u64, revision_height: u64) -> Self {
        Self {
            revision_number,
            revision_height,
        }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn zero() -> Height {
        Self {
            revision_number: 0,
            revision_height: 0,
        }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn is_zero(&self) -> bool {
        self.revision_height == 0
    }

    #[cfg_attr(feature="prusti", requires(u64::MAX - self.revision_height >= delta))]
    pub fn add(&self, delta: u64) -> Height {
        Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height + delta,
        }
    }

    #[cfg_attr(feature="prusti", requires(self.revision_height < u64::MAX))]
    pub fn increment(&self) -> Height {
        self.add(1)
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn sub(&self, delta: u64) -> Result<Height, Error> {
        if self.revision_height <= delta {
            return Err(Error::invalid_height_result());
        }

        Ok(Height {
            revision_number: self.revision_number,
            revision_height: self.revision_height - delta,
        })
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn decrement(&self) -> Result<Height, Error> {
        self.sub(1)
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn with_revision_height(self, revision_height: u64) -> Height {
        Height {
            revision_height,
            ..self
        }
    }
}

impl Default for Height {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn default() -> Self {
        Self::zero()
    }
}

impl PartialOrd for Height {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Height {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.revision_number < other.revision_number {
            Ordering::Less
        } else if self.revision_number > other.revision_number {
            Ordering::Greater
        } else if self.revision_height < other.revision_height {
            Ordering::Less
        } else if self.revision_height > other.revision_height {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl Protobuf<RawHeight> for Height {}

impl From<RawHeight> for Height {
    fn from(raw: RawHeight) -> Self {
        Height {
            revision_number: raw.revision_number,
            revision_height: raw.revision_height,
        }
    }
}

impl From<Height> for RawHeight {
    fn from(ics_height: Height) -> Self {
        RawHeight {
            revision_number: ics_height.revision_number,
            revision_height: ics_height.revision_height,
        }
    }
}

impl std::fmt::Debug for Height {
    #[cfg_attr(feature="prusti", trusted_skip)]
    // warning: [Prusti: unsupported feature] cast statements that create loans are not supported
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.debug_struct("Height")
            .field("revision", &self.revision_number)
            .field("height", &self.revision_height)
            .finish()
    }
}

/// Custom debug output to omit the packet data
impl std::fmt::Display for Height {
#[cfg_attr(feature="prusti", trusted)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
panic!("No") //         write!(f, "{}-{}", self.revision_number, self.revision_height)
    }
}

define_error! {
    HeightError {
        HeightConversion
            { height: String }
            [ TraceError<ParseIntError> ]
            | e | {
                format_args!("cannot convert into a `Height` type from string {0}",
                    e.height)
            },
    }
}

impl TryFrom<&str> for Height {
    type Error = HeightError;

#[cfg_attr(feature="prusti", trusted)]
    fn try_from(value: &str) -> Result<Self, Self::Error> {
panic!("No") // panic!("No") //         let split: Vec<&str> = value.split('-').collect();
// //         Ok(Height {
// //             revision_number: split[0]
// //                 .parse::<u64>()
// //                 .map_err(|e| HeightError::height_conversion(value.to_owned(), e))?,
// //             revision_height: split[1]
// //                 .parse::<u64>()
// //                 .map_err(|e| HeightError::height_conversion(value.to_owned(), e))?,
// //         })
    }
}

impl From<Height> for String {
    #[cfg_attr(feature="prusti", trusted_skip)]
    // [Prusti: unsupported feature] unsupported constant type Ref('_#12r, [&str; 2], Not)
    fn from(height: Height) -> Self {
        format!("{}-{}", height.revision_number, height.revision_number)
    }
}

impl FromStr for Height {
    type Err = HeightError;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Height::try_from(s)
    }
}
