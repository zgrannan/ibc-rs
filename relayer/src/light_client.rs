use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::chain::Chain;
use crate::error;

pub mod tendermint;

#[cfg(test)]
pub mod mock;

/// Defines what fraction of the total voting power of a known
/// and trusted validator set is sufficient for a commit to be
/// accepted going forward.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustThreshold {
    /// Numerator of the trust threshold fraction
    pub numerator: u64,
    /// Numerator of the trust threshold fraction
    pub denominator: u64,
}

impl TrustThreshold {
    /// Constant for a trust threshold of 2/3.
    pub const TWO_THIRDS: Self = Self {
        numerator: 2,
        denominator: 3,
    };

    /// Instantiate a [`TrustThreshold`] if the given denominator and
    /// numerator are valid.
    ///
    /// The parameters are valid if and only if `1/3 <= numerator/denominator <= 1`.
    /// In any other case we return `None`.
    pub fn new(numerator: u64, denominator: u64) -> Option<Self> {
        if numerator <= denominator && denominator > 0 && 3 * numerator >= denominator {
            Some(Self {
                numerator,
                denominator,
            })
        } else {
            None
        }
    }
}

/// Security parameters for the light client
#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityParams {
    /// Defines what fraction of the total voting power of a known
    /// and trusted validator set is sufficient for a commit to be
    /// accepted going forward.
    pub trust_threshold: TrustThreshold,

    /// How long a validator set is trusted for (must be shorter than the chain's
    /// unbonding period)
    pub trusting_period: Duration,

    /// Correction parameter dealing with only approximately synchronized clocks.
    /// The local clock should always be ahead of timestamps from the blockchain; this
    /// is the maximum amount that the local clock may drift behind a timestamp from the
    /// blockchain.
    pub clock_drift: Duration,
}

/// Defines a light block from the point of view of the relayer.
pub trait LightBlock<C: Chain>: Send + Sync {
    fn signed_header(&self) -> &C::Header;
}

/// Defines a client from the point of view of the relayer.
pub trait LightClient<C: Chain>: Send + Sync {
    /// Fetch a header from the chain at the given height and verify it
    fn verify(
        &mut self,
        trusted: ibc::Height,
        target: ibc::Height,
        params: SecurityParams,
    ) -> Result<C::LightBlock, error::Error>;

    /// Fetch a header from the chain at the given height, without verifying it
    fn fetch(&mut self, height: ibc::Height) -> Result<C::LightBlock, error::Error>;
}
