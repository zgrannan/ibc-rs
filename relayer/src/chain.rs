pub mod client;
pub mod cosmos;
pub mod counterparty;
pub mod endpoint;
pub mod handle;
pub mod requests;
pub mod runtime;
pub mod tracking;
#[cfg(test)]
pub mod mock;
use serde::{de::Error, Deserialize, Serialize};
#[derive(Copy, Clone, Debug, Serialize)]
/// Types of chains the relayer can relay to and from
pub enum ChainType {
    /// Chains based on the Cosmos SDK
    CosmosSdk,
    /// Mock chain used for testing
    #[cfg(test)]
    Mock,
}
impl<'de> Deserialize<'de> for ChainType {
    #[prusti_contracts::trusted]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let original = String::deserialize(deserializer)?;
        let s = original.to_ascii_lowercase().replace('-', "");
        match s.as_str() {
            "cosmossdk" => Ok(Self::CosmosSdk),
            #[cfg(test)]
            "mock" => Ok(Self::Mock),
            _ => Err(D::Error::unknown_variant(&original, &["cosmos-sdk"])),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Copy, Clone, Debug, Serialize, Deserialize)]
    pub struct Config {
        tpe: ChainType,
    }
    #[prusti_contracts::trusted]
    fn parse(variant: &str) -> Result<ChainType, toml::de::Error> {
        toml::from_str::<Config>(&format!("tpe = '{variant}'")).map(|e| e.tpe)
    }
    #[test]
    #[prusti_contracts::trusted]
    fn deserialize() {
        use ChainType::*;
        assert!(matches!(parse("CosmosSdk"), Ok(CosmosSdk)));
        assert!(matches!(parse("cosmossdk"), Ok(CosmosSdk)));
        assert!(matches!(parse("cosmos-sdk"), Ok(CosmosSdk)));
        assert!(matches!(parse("mock"), Ok(Mock)));
        assert!(matches!(parse("hello-world"), Err(_)));
    }
}

