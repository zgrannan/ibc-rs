use std::convert::TryFrom;

use tendermint_rpc as rpc;

use tendermint_light_client::{
    components::{self, io::AtHeight},
    light_client::{LightClient as TmLightClient, Options as TmOptions},
    operations,
    state::State as ClientState,
    store::{memory::MemoryStore, LightStore},
    types::Height as TMHeight,
    types::{LightBlock, PeerId, Status, TrustThreshold as TmTrustThreshold},
};

use ibc::ics24_host::identifier::ChainId;

use crate::{
    chain::CosmosSdkChain,
    config::ChainConfig,
    error::{self, Error},
};

use super::{SecurityParams, TrustThreshold};

pub struct LightClient {
    chain_id: ChainId,
    peer_id: PeerId,
    options: TmOptions,
    io: components::io::ProdIo,
}

impl super::LightClient<CosmosSdkChain> for LightClient {
    fn verify(
        &mut self,
        trusted: ibc::Height,
        target: ibc::Height,
        params: SecurityParams,
    ) -> Result<LightBlock, Error> {
        let target_height = TMHeight::try_from(target.revision_height)
            .map_err(|e| error::Kind::InvalidHeight.context(e))?;

        let client = self.prepare_client(params);
        let mut state = self.prepare_state(trusted)?;

        let light_block = client
            .verify_to_target(target_height, &mut state)
            .map_err(|e| error::Kind::LightClient(self.chain_id.to_string()).context(e))?;

        Ok(light_block)
    }

    fn fetch(&mut self, height: ibc::Height) -> Result<LightBlock, Error> {
        let height = TMHeight::try_from(height.revision_height)
            .map_err(|e| error::Kind::InvalidHeight.context(e))?;

        self.fetch_light_block(AtHeight::At(height))
    }
}

impl LightClient {
    pub fn from_config(config: &ChainConfig) -> Result<Self, Error> {
        let options = TmOptions {
            trust_threshold: config.trust_threshold,
            trusting_period: config.trusting_period,
            clock_drift: config.clock_drift,
        };

        let rpc_client = rpc::HttpClient::new(config.rpc_addr.clone())
            .map_err(|e| error::Kind::LightClient(config.rpc_addr.to_string()).context(e))?;

        let peer = config.primary().ok_or_else(|| {
            error::Kind::LightClient(config.rpc_addr.to_string()).context("no primary peer")
        })?;

        let io = components::io::ProdIo::new(peer.peer_id, rpc_client, Some(peer.timeout));

        Ok(Self {
            chain_id: config.id.clone(),
            peer_id: peer.peer_id,
            options,
            io,
        })
    }

    fn prepare_client(&self, params: SecurityParams) -> TmLightClient {
        let clock = components::clock::SystemClock;
        let hasher = operations::hasher::ProdHasher;
        let verifier = components::verifier::ProdVerifier::default();
        let scheduler = components::scheduler::basic_bisecting_schedule;

        TmLightClient::new(
            self.peer_id,
            params.into(),
            clock,
            scheduler,
            verifier,
            hasher,
            self.io.clone(),
        )
    }

    fn prepare_state(&self, trusted: ibc::Height) -> Result<ClientState, Error> {
        let trusted_height = TMHeight::try_from(trusted.revision_height)
            .map_err(|e| error::Kind::InvalidHeight.context(e))?;

        let trusted_block = self.fetch_light_block(AtHeight::At(trusted_height))?;

        let mut store = MemoryStore::new();
        store.insert(trusted_block, Status::Trusted);

        Ok(ClientState::new(store))
    }

    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, Error> {
        use tendermint_light_client::components::io::Io;

        self.io.fetch_light_block(height).map_err(|e| {
            error::Kind::LightClient(self.chain_id.to_string())
                .context(e)
                .into()
        })
    }
}

impl From<SecurityParams> for TmOptions {
    fn from(params: SecurityParams) -> Self {
        Self {
            trust_threshold: params.trust_threshold.into(),
            trusting_period: params.trusting_period,
            clock_drift: params.clock_drift,
        }
    }
}

impl From<TrustThreshold> for TmTrustThreshold {
    fn from(t: TrustThreshold) -> Self {
        Self {
            numerator: t.numerator,
            denominator: t.denominator,
        }
    }
}
