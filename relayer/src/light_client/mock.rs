use tendermint_testgen::light_block::TmLightBlock;

use ibc::ics02_client::client_misbehaviour::{AnyMisbehaviour, Misbehaviour};
use ibc::ics02_client::events::UpdateClient;
use ibc::ics02_client::header::AnyHeader;
use ibc::ics07_tendermint::header::Header as TmHeader;
use ibc::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;
use ibc::ics24_host::identifier::ChainId;
use ibc::mock::host::HostBlock;
use ibc::Height;
use ibc::{downcast, ics02_client::client_state::AnyClientState};

use crate::chain::mock::MockChain;
use crate::chain::Chain;
use crate::error::{Error, Kind};

/// A light client serving a mock chain.
pub struct LightClient {
    chain_id: ChainId,
}

impl LightClient {
    pub fn new(chain: &MockChain) -> LightClient {
        LightClient {
            chain_id: chain.id().clone(),
        }
    }

    /// Returns a LightBlock at the requested height `h`.
    fn light_block(&self, h: Height) -> TmLightBlock {
        HostBlock::generate_tm_block(self.chain_id.clone(), h.revision_height)
    }
}

impl super::LightClient<MockChain> for LightClient {
    fn verify(
        &mut self,
        _trusted: Height,
        target: Height,
        _client_state: &AnyClientState,
    ) -> Result<TmLightBlock, Error> {
        Ok(self.light_block(target))
    }

    fn fetch(&mut self, height: Height) -> Result<TmLightBlock, Error> {
        Ok(self.light_block(height))
    }

    fn build_misbehaviour(
        &mut self,
        client_state: &AnyClientState,
        update: UpdateClient,
    ) -> Result<Option<AnyMisbehaviour>, Error> {
        let update_header = update.header.clone().ok_or_else(|| {
            Kind::Misbehaviour(format!(
                "missing header in update client event {}",
                self.chain_id
            ))
        })?;
        let tm_ibc_client_header =
            downcast!(update_header => AnyHeader::Tendermint).ok_or_else(|| {
                Kind::Misbehaviour(format!(
                    "header type incompatible for chain {}",
                    self.chain_id
                ))
            })?;

        // Get the latest chain height from context
        let latest_chain_height = ibc::Height::zero();

        // set the target height to the minimum between the update height and latest chain height
        let target_height = std::cmp::min(*update.consensus_height(), latest_chain_height);
        let trusted_height = tm_ibc_client_header.trusted_height;
        // TODO - check that a consensus state at trusted_height still exists on-chain,
        // currently we don't have access to Cosmos chain query from here

        let tm_witness_node_header = {
            assert!(trusted_height < latest_chain_height);
            let trusted_light_block = self.fetch(trusted_height.increment())?;
            let target_light_block = self.verify(trusted_height, target_height, &client_state)?;
            TmHeader {
                trusted_height,
                signed_header: target_light_block.signed_header.clone(),
                validator_set: target_light_block.validators,
                trusted_validator_set: trusted_light_block.validators,
            }
        };

        let misbehaviour = if !tm_witness_node_header.compatible_with(&tm_ibc_client_header) {
            Some(
                AnyMisbehaviour::Tendermint(TmMisbehaviour {
                    client_id: update.client_id().clone(),
                    header1: tm_ibc_client_header,
                    header2: tm_witness_node_header,
                })
                .wrap_any(),
            )
        } else {
            None
        };

        Ok(misbehaviour)
    }
}
