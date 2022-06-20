//! Cosmos-specific client settings.

use core::time::Duration;

use tracing::warn;

use ibc::core::ics02_client::trust_threshold::TrustThreshold;

use crate::config::ChainConfig;

/// Cosmos-specific client parameters for the `build_client_state` operation.
#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub max_clock_drift: Duration,
    pub trusting_period: Option<Duration>,
    pub trust_threshold: TrustThreshold,
}

impl Settings {
}

/// The client state clock drift must account for destination
/// chain block frequency and clock drift on source and dest.
/// https://github.com/informalsystems/ibc-rs/issues/1445
fn calculate_client_state_drift(
    src_chain_config: &ChainConfig,
    dst_chain_config: &ChainConfig,
) -> Duration {
    src_chain_config.clock_drift + dst_chain_config.clock_drift + dst_chain_config.max_block_time
}
