//! Data structures and logic to set up IBC client's parameters.

use crate::chain::cosmos;
use crate::config::ChainConfig;

/// Client parameters for the `build_create_client` operation.
///
/// The parameters are specialized for each supported chain type.
#[derive(Clone, Debug)]
pub enum ClientSettings {
    Tendermint(cosmos::client::Settings),
}

impl ClientSettings {
}
