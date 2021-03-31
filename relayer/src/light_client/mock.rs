use std::cmp::Ordering;

use crate::chain::mock::MockChain;
use crate::chain::Chain;
use crate::error::Error;
use crate::error::Kind;
use ibc::{downcast, ics02_client::{client_misbehaviour::AnyMisbehaviour, header::AnyHeader}, mock::{header::MockHeader, host::HostType}};
use ibc::ics02_client::events::UpdateClient;
use ibc::ics24_host::identifier::ChainId;
use ibc::mock::host::HostBlock;
use ibc::Height;

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
    fn light_block(&self, h: Height) -> HostBlock {
        HostBlock::generate_block(self.chain_id.clone(), HostType::Mock, h.revision_height)
    }

    //    /// Returns a LightBlock at the requested height `h`.
    //    fn light_block(&self, h: Height) -> <MockChain as Chain>::LightBlock {
    //     HostBlock::generate_tm_block(self.chain_id.clone(), h.revision_height)
    // }

    /// TODO - move to light client supervisor/ library
    pub fn incompatible_headers(ibc_client_header: &MockHeader, chain_header: &MockHeader) -> bool {
        let ibc_client_height = ibc_client_header.height;
        let chain_header_height = chain_header.height;

        match ibc_client_height.cmp(&&chain_header_height) {
            Ordering::Equal => ibc_client_header == chain_header,
            Ordering::Greater => {
                ibc_client_header.timestamp
                    <= chain_header.timestamp
            }
            Ordering::Less => false,
        }
    }

}

#[allow(unused_variables)]
impl super::LightClient<MockChain> for LightClient {
    fn latest_trusted(&self) -> Result<Option<<MockChain as Chain>::LightBlock>, Error> {
        unimplemented!()
    }

    fn verify_to_latest(&self) -> Result<<MockChain as Chain>::LightBlock, Error> {
        unimplemented!()
    }

    fn verify_to_target(&self, height: Height) -> Result<<MockChain as Chain>::LightBlock, Error> {
        unimplemented!() // Ok(self.light_block(height))
    }

    fn get_minimal_set(
        &self,
        latest_client_state_height: Height,
        target_height: Height,
    ) -> Result<Vec<Height>, Error> {
        unimplemented!()
    }

    fn build_misbehaviour(
        &self,
        update: UpdateClient,
        latest_chain_height: Height,
    ) -> Result<Option<AnyMisbehaviour>, Error> {



        let update_header = update.header.clone().ok_or_else(|| {
            Kind::Misbehaviour(format!(
                "missing header in update client event {}",
                self.chain_id
            ))
        })?;

        let mock_ibc_client_header =
            downcast!(update_header => AnyHeader::Mock).ok_or_else(|| {
                Kind::Misbehaviour(format!(
                    "header type incompatible for chain {}",
                    self.chain_id
                ))
            })?;

        // set the target height to the minimum between the update height and latest chain height
        let target_height = std::cmp::min(*update.consensus_height(), latest_chain_height);
        //let trusted_height = mock_ibc_client_header.trusted_height;

        let mock_chain_target_block = self.light_block(target_height); 

        let mock_chain_header = 
        match mock_chain_target_block {
            HostBlock::Mock(b) => Some(MockHeader{height: b.height.clone(), timestamp: b.timestamp}),
            HostBlock::SyntheticTendermint(_) => None,
        }; 
 
        if mock_chain_header.is_none() {
            return Err(Kind::Misbehaviour(format!(
                "header type incompatible for chain {}",
                self.chain_id
            )).into())
        }

        //{
            //assert!(trusted_height < latest_chain_height);
            //let trusted_light_block = self.verify_to_target(trusted_height)?;
            //let target_light_block = self.verify_to_target(target_height)?;

            // TmHeader {
            //     trusted_height,
            //     signed_header: target_light_block.signed_header.clone(),
            //     validator_set: target_light_block.validators,
            //     trusted_validator_set: trusted_light_block.validators,
            // }
        //};

        let misbehaviour =
            if LightClient::incompatible_headers(&mock_ibc_client_header, &mock_chain_header.unwrap()) {
                Some(
                    AnyMisbehaviour::Mock(MockMisbehaviour {
                        client_id: update.client_id().clone(),
                        header1: mock_client_header,
                        header2: mock_chain_header,
                    })
                    .wrap_any(),
                )
            } else {
                None
            };

        Ok(misbehaviour)

        Ok(None)
        //unimplemented!()
    }

        /// TODO - move to light client supervisor/ library
        // pub fn incompatible_headers(ibc_client_header: &MockHeader, chain_header: &MockHeader) -> bool {
        //     let ibc_client_height = ibc_client_header.signed_header.header.height;
        //     let chain_header_height = chain_header.signed_header.header.height;
    
        //     match ibc_client_height.cmp(&&chain_header_height) {
        //         Ordering::Equal => ibc_client_header == chain_header,
        //         Ordering::Greater => {
        //             ibc_client_header.signed_header.header.time
        //                 <= chain_header.signed_header.header.time
        //         }
        //         Ordering::Less => false,
        //     }
        // }
}
