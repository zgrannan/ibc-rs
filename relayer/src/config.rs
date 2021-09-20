//! Relayer configuration

pub mod reload;
pub mod types;

use std::collections::{HashMap, HashSet};
use std::{fmt, fs, fs::File, io::Write, path::Path, time::Duration};

use serde_derive::{Deserialize, Serialize};
use tendermint_light_client::types::TrustThreshold;

use ibc::ics24_host::identifier::{ChainId, ChannelId, PortId};
use ibc::timestamp::ZERO_DURATION;

use crate::config::types::{MaxMsgNum, MaxTxSize};
use crate::error::Error;
#[cfg(feature="prusti")]
use prusti_contracts::*;

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiPartialEq))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,PartialEq,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
pub struct GasPrice {
    pub price: f64,
    pub denom: String,
}

#[cfg(feature="prusti")]
impl <'de> serde::Deserialize<'de> for GasPrice {
    #[cfg_attr(feature="prusti", trusted_skip)]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de> {
        todo!()
    }
}

impl GasPrice {
    #[cfg_attr(feature="prusti_fast", trusted)]
    pub const fn new(price: f64, denom: String) -> Self {
        Self { price, denom }
    }
}

impl fmt::Display for GasPrice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.price, self.denom)
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), serde(
    rename_all = "lowercase",
    tag = "policy",
    content = "list",
    deny_unknown_fields
))]
pub enum PacketFilter {
    Allow(ChannelsSpec),
    Deny(ChannelsSpec),
    AllowAll,
}

impl Default for PacketFilter {
    /// By default, allows all channels & ports.
    fn default() -> Self {
        Self::AllowAll
    }
}

impl PacketFilter {
    /// Returns true if the packets can be relayed on the channel with [`PortId`] and [`ChannelId`],
    /// false otherwise.
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn is_allowed(&self, port_id: &PortId, channel_id: &ChannelId) -> bool {
        match self {
            PacketFilter::Allow(spec) => spec.contains(&(port_id.clone(), channel_id.clone())),
            PacketFilter::Deny(spec) => !spec.contains(&(port_id.clone(), channel_id.clone())),
            PacketFilter::AllowAll => true,
        }
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,Default,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Default,Serialize))]
#[cfg_attr(not(feature="prusti"), serde(deny_unknown_fields))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
pub struct ChannelsSpec(HashSet<(PortId, ChannelId)>);

impl ChannelsSpec {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn contains(&self, channel_port: &(PortId, ChannelId)) -> bool {
        self.0.contains(channel_port)
    }
}

/// Defaults for various fields
pub mod default {
    use super::*;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn filter() -> bool {
        false
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn clear_packets_interval() -> u64 {
        100
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn rpc_timeout() -> Duration {
        Duration::from_secs(10)
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn trusting_period() -> Duration {
        Duration::from_secs(336 * 60 * 60) // 336 hours ~ 14 days
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn clock_drift() -> Duration {
        Duration::from_secs(5)
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn connection_delay() -> Duration {
        ZERO_DURATION
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,Default,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Default,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), serde(deny_unknown_fields))]
pub struct Config {
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub global: GlobalConfig,
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub telemetry: TelemetryConfig,
    #[cfg_attr(not(feature="prusti"), serde(default = "Vec::new", skip_serializing_if = "Vec::is_empty"))]
    pub chains: Vec<ChainConfig>,
}

impl Config {

    #[cfg(feature="prusti")]
    #[trusted]
    pub fn has_chain(&self, id: &ChainId) -> bool {
        todo!()
    }

    #[cfg(not(feature="prusti"))]
    pub fn has_chain(&self, id: &ChainId) -> bool {
        self.chains.iter().any(|c| c.id == *id)
    }

    #[cfg(feature="prusti")]
    #[trusted]
    pub fn find_chain(&self, id: &ChainId) -> Option<&ChainConfig> {
        todo!()
    }

    #[cfg(not(feature="prusti"))]
    pub fn find_chain(&self, id: &ChainId) -> Option<&ChainConfig> {
        self.chains.iter().find(|c| c.id == *id)
    }

    #[cfg(feature="prusti")]
    #[trusted]
    pub fn find_chain_mut(&mut self, id: &ChainId) -> Option<&mut ChainConfig> {
        todo!()
    }

    #[cfg(not(feature="prusti"))]
    pub fn find_chain_mut(&mut self, id: &ChainId) -> Option<&mut ChainConfig> {
        self.chains.iter_mut().find(|c| c.id == *id)
    }

    /// Returns true if filtering is disabled or if packets are allowed on
    /// the channel [`PortId`] [`ChannelId`] on [`ChainId`].
    /// Returns false otherwise.
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn packets_on_channel_allowed(
        &self,
        chain_id: &ChainId,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> bool {
        if !self.global.filter {
            return true;
        }

        match self.find_chain(chain_id) {
            None => false,
            Some(chain_config) => chain_config.packet_filter.is_allowed(port_id, channel_id),
        }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn handshake_enabled(&self) -> bool {
        self.global.strategy == Strategy::HandshakeAndPackets
    }

    #[cfg_attr(feature="prusti", trusted_skip)]
    pub fn chains_map(&self) -> HashMap<&ChainId, &ChainConfig> {
        self.chains.iter().map(|c| (&c.id, c)).collect()
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiPartialEq,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,PartialEq,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
pub enum Strategy {
    #[cfg_attr(not(feature="prusti"), serde(rename = "packets"))]
    Packets,

    #[cfg_attr(not(feature="prusti"), serde(rename = "all"))]
    HandshakeAndPackets,
}

impl Default for Strategy {
    fn default() -> Self {
        Self::Packets
    }
}

/// Log levels are wrappers over [`tracing_core::Level`].
///
/// [`tracing_core::Level`]: https://docs.rs/tracing-core/0.1.17/tracing_core/struct.Level.html
#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiPartialEq,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,PartialEq,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), serde(rename_all = "lowercase"))]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl fmt::Display for LogLevel {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), serde(default, deny_unknown_fields))]
pub struct GlobalConfig {
    pub strategy: Strategy,
    pub log_level: LogLevel,
    #[cfg_attr(not(feature="prusti"), serde(default = "default::filter"))]
    pub filter: bool,
    #[cfg_attr(not(feature="prusti"), serde(default = "default::clear_packets_interval"))]
    pub clear_packets_interval: u64,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            strategy: Strategy::default(),
            log_level: LogLevel::default(),
            filter: default::filter(),
            clear_packets_interval: default::clear_packets_interval(),
        }
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug,PrustiSerialize))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug,Serialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize))]
#[cfg_attr(feature="prusti", derive(PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), serde(deny_unknown_fields))]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
}

impl Default for TelemetryConfig {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn default() -> Self {
        Self {
            enabled: false,
            host: "127.0.0.1".to_string(),
            port: 3001,
        }
    }
}

#[cfg_attr(feature="prusti", derive(PrustiClone,PrustiDebug))]
#[cfg_attr(not(feature="prusti"), derive(Clone,Debug))]
#[cfg_attr(feature="prusti", derive(PrustiSerialize,PrustiDeserialize))]
#[cfg_attr(not(feature="prusti"), derive(Deserialize, Serialize))]
#[cfg_attr(not(feature="prusti"), serde(deny_unknown_fields))]
pub struct ChainConfig {
    pub id: ChainId,
    pub rpc_addr: tendermint_rpc::Url,
    pub websocket_addr: tendermint_rpc::Url,
    pub grpc_addr: tendermint_rpc::Url,
    #[cfg_attr(not(feature="prusti"), serde(default = "default::rpc_timeout", with = "humantime_serde"))]
    pub rpc_timeout: Duration,
    pub account_prefix: String,
    pub key_name: String,
    pub store_prefix: String,
    pub max_gas: Option<u64>,
    pub gas_adjustment: Option<f64>,
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub max_msg_num: MaxMsgNum,
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub max_tx_size: MaxTxSize,
    #[cfg_attr(not(feature="prusti"), serde(default = "default::clock_drift", with = "humantime_serde"))]
    pub clock_drift: Duration,
    #[cfg_attr(not(feature="prusti"), serde(default = "default::trusting_period", with = "humantime_serde"))]
    pub trusting_period: Duration,

    // these two need to be last otherwise we run into `ValueAfterTable` error when serializing to TOML
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub trust_threshold: TrustThreshold,
    pub gas_price: GasPrice,
    #[cfg_attr(not(feature="prusti"), serde(default))]
    pub packet_filter: PacketFilter,
}

/// Attempt to load and parse the TOML config file as a `Config`.
#[cfg_attr(feature="prusti", trusted_skip)]
pub fn load(path: impl AsRef<Path>) -> Result<Config, Error> {
    let config_toml = std::fs::read_to_string(&path).map_err(Error::config_io)?;

    let config = toml::from_str::<Config>(&config_toml[..]).map_err(Error::config_decode)?;

    Ok(config)
}

/// Serialize the given `Config` as TOML to the given config file.
#[cfg_attr(feature="prusti_fast", trusted_skip)]
pub fn store(config: &Config, path: impl AsRef<Path>) -> Result<(), Error> {
    let mut file = if path.as_ref().exists() {
        fs::OpenOptions::new().write(true).truncate(true).open(path)
    } else {
        File::create(path)
    }
    .map_err(Error::config_io)?;

    store_writer(config, &mut file)
}

/// Serialize the given `Config` as TOML to the given writer.
#[cfg_attr(feature="prusti_fast", trusted_skip)]
pub(crate) fn store_writer(config: &Config, mut writer: impl Write) -> Result<(), Error> {
    let toml_config = toml::to_string_pretty(&config).map_err(Error::config_encode)?;

    writeln!(writer, "{}", toml_config).map_err(Error::config_io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{load, store_writer};
    use test_env_log::test;

    #[test]
    fn parse_valid_config() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/config/fixtures/relayer_conf_example.toml"
        );

        let config = load(path);
        println!("{:?}", config);
        assert!(config.is_ok());
    }

    #[test]
    fn serialize_valid_config() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/config/fixtures/relayer_conf_example.toml"
        );

        let config = load(path).expect("could not parse config");

        let mut buffer = Vec::new();
        store_writer(&config, &mut buffer).unwrap();
    }
}
