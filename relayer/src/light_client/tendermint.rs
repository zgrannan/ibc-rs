use itertools::Itertools;

use tendermint_light_client::{
    components::{self, io::AtHeight},
    light_client::LightClient as TmLightClient,
    state::State as LightClientState,
    store::{memory::MemoryStore, LightStore},
};
use tendermint_light_client_verifier::operations;
use tendermint_light_client_verifier::options::Options as TmOptions;
use tendermint_light_client_verifier::types::{Height as TMHeight, LightBlock, PeerId, Status};
use tendermint_light_client_verifier::ProdVerifier;
use tendermint_rpc as rpc;

use ibc::{
    clients::ics07_tendermint::{
        header::{headers_compatible, Header as TmHeader},
        misbehaviour::Misbehaviour as TmMisbehaviour,
    },
    core::{
        ics02_client::{
            client_state::AnyClientState,
            client_type::ClientType,
            events::UpdateClient,
            header::{AnyHeader, Header},
            misbehaviour::{Misbehaviour, MisbehaviourEvidence},
        },
        ics24_host::identifier::ChainId,
    },
    downcast,
};
use tracing::trace;

use crate::{config::ChainConfig, error::Error};

use super::Verified;

pub struct LightClient {
    chain_id: ChainId,
    peer_id: PeerId,
    io: components::io::ProdIo,
}

impl LightClient {
    pub fn from_config(config: &ChainConfig, peer_id: PeerId) -> Result<Self, Error> {
        let rpc_client = rpc::HttpClient::new(config.rpc_addr.clone())
            .map_err(|e| Error::rpc(config.rpc_addr.clone(), e))?;

        let io = components::io::ProdIo::new(peer_id, rpc_client, Some(config.rpc_timeout));

        Ok(Self {
            chain_id: config.id.clone(),
            peer_id,
            io,
        })
    }

    fn prepare_client(&self, client_state: &AnyClientState) -> Result<TmLightClient, Error> {
        let clock = components::clock::SystemClock;
        let hasher = operations::hasher::ProdHasher;
        let verifier = ProdVerifier::default();
        let scheduler = components::scheduler::basic_bisecting_schedule;

        let client_state =
            downcast!(client_state => AnyClientState::Tendermint).ok_or_else(|| {
                Error::client_type_mismatch(ClientType::Tendermint, client_state.client_type())
            })?;

        let params = TmOptions {
            trust_threshold: client_state
                .trust_level
                .try_into()
                .map_err(Error::light_client_state)?,
            trusting_period: client_state.trusting_period,
            clock_drift: client_state.max_clock_drift,
        };

        Ok(TmLightClient::new(
            self.peer_id,
            params,
            clock,
            scheduler,
            verifier,
            hasher,
            self.io.clone(),
        ))
    }

    fn prepare_state(&self, trusted: ibc::Height) -> Result<LightClientState, Error> {
        let trusted_height =
            TMHeight::try_from(trusted.revision_height).map_err(Error::invalid_height)?;

        let trusted_block = self.fetch_light_block(AtHeight::At(trusted_height))?;

        let mut store = MemoryStore::new();
        store.insert(trusted_block, Status::Trusted);

        Ok(LightClientState::new(store))
    }

    fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, Error> {
        use tendermint_light_client::components::io::Io;

        self.io
            .fetch_light_block(height)
            .map_err(|e| Error::light_client_io(self.chain_id.to_string(), e))
    }

}
