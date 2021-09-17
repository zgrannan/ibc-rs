use ibc::ics02_client::misbehaviour::MisbehaviourEvidence;
use tendermint_testgen::light_block::TmLightBlock;

use ibc::ics02_client::client_state::AnyClientState;
use ibc::ics02_client::events::UpdateClient;
use ibc::ics07_tendermint::header::Header as TmHeader;
use ibc::ics24_host::identifier::ChainId;
use ibc::mock::host::HostBlock;
use ibc::Height;

use crate::chain::mock::MockChain;
use crate::chain::Chain;
use crate::error::Error;

use super::Verified;

/// A light client serving a mock chain.
pub struct LightClient {
    chain_id: ChainId,
}

impl LightClient {
#[cfg_attr(feature="prusti_fast", trusted)]
    pub fn new(chain: &MockChain) -> LightClient {
        LightClient {
            chain_id: chain.id().clone(),
        }
    }

    /// Returns a LightBlock at the requested height `h`.
#[cfg_attr(feature="prusti_fast", trusted)]
    fn light_block(&self, h: Height) -> TmLightBlock {
        HostBlock::generate_tm_block(self.chain_id.clone(), h.revision_height)
    }
}

impl super::LightClient<MockChain> for LightClient {
#[cfg_attr(feature="prusti_fast", trusted)]
    fn verify(
        &mut self,
        _trusted: Height,
        target: Height,
        _client_state: &AnyClientState,
    ) -> Result<Verified<TmLightBlock>, Error> {
        Ok(Verified {
            target: self.light_block(target),
            supporting: Vec::new(),
        })
    }

#[cfg_attr(feature="prusti_fast", trusted)]
    fn fetch(&mut self, height: Height) -> Result<TmLightBlock, Error> {
        Ok(self.light_block(height))
    }

#[cfg_attr(feature="prusti_fast", trusted)]
    fn check_misbehaviour(
        &mut self,
        _update: UpdateClient,
        _client_state: &AnyClientState,
    ) -> Result<Option<MisbehaviourEvidence>, Error> {
        unimplemented!()
    }

#[cfg_attr(feature="prusti_fast", trusted)]
    fn header_and_minimal_set(
        &mut self,
        trusted_height: Height,
        target_height: Height,
        client_state: &AnyClientState,
    ) -> Result<Verified<TmHeader>, Error> {
        let Verified { target, supporting } =
            self.verify(trusted_height, target_height, client_state)?;

        assert!(supporting.is_empty());

        let succ_trusted = self.fetch(trusted_height.increment())?;

        let target = TmHeader {
            signed_header: target.signed_header,
            validator_set: target.validators,
            trusted_height,
            trusted_validator_set: succ_trusted.validators,
        };

        Ok(Verified {
            target,
            supporting: Vec::new(),
        })
    }
}
