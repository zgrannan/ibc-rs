use std::cmp::Ordering;

use crate::chain::mock::MockChain;
use crate::chain::Chain;
use crate::error::Error;
use crate::error::Kind;
use ibc::{downcast, ics02_client::{client_misbehaviour::{AnyMisbehaviour, Misbehaviour}, header::AnyHeader}, mock::{header::MockHeader, host::HostType, misbehaviour::Misbehaviour as MockMisbehaviour}};
use ibc::ics07_tendermint::header::{Header as TmHeader, Header};
use ibc::ics07_tendermint::misbehaviour::Misbehaviour as TmMisbehaviour;

use ibc::ics02_client::events::UpdateClient;
use ibc::ics24_host::identifier::ChainId;
use ibc::mock::host::HostBlock;
use ibc::Height;
//use tendermint::Time;


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

    // /// Returns a LightBlock at the requested height `h`.
    // fn light_block(&self, h: Height) -> HostBlock {
    //     HostBlock::generate_block(self.chain_id.clone(), HostType::Mock, h.revision_height)
    // }

       /// Returns a LightBlock at the requested height `h`.
       fn light_block(&self, h: Height) -> <MockChain as Chain>::LightBlock {
        HostBlock::generate_tm_block(self.chain_id.clone(), h.revision_height)
    }

    /// TODO - move to light client supervisor/ library
    pub fn incompatible_headers(ibc_client_header: &TmHeader, chain_header: &TmHeader) -> bool {
        // let ibc_client_height = ibc_client_header.height;
        // let chain_header_height = chain_header.height;

        let ibc_client_height = ibc_client_header.signed_header.header.height;
        let chain_header_height = chain_header.signed_header.header.height;

        match ibc_client_height.cmp(&&chain_header_height) {
            Ordering::Equal => ibc_client_header == chain_header,
            Ordering::Greater => {
                ibc_client_header.signed_header.header.time
                    <= chain_header.signed_header.header.time
            }
            Ordering::Less => false,
        }

        // match ibc_client_height.cmp(&&chain_header_height) {
        //     Ordering::Equal => ibc_client_header == chain_header,
        //     Ordering::Greater => {
        //         ibc_client_header.timestamp
        //             <= chain_header.timestamp
        //     }
        //     Ordering::Less => false,
        // }
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

    // fn verify_to_target(&self, height: Height) -> Result<<MockChain as Chain>::LightBlock, Error> {
    //     //unimplemented!() // 
    //     Ok(self.light_block(height))
    // }

    fn verify_to_target(&self, height: Height) -> Result<<MockChain as Chain>::LightBlock, Error> {
        //unimplemented!() // 
        Ok(self.light_block(height))
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


        crate::time!("mock light client build_misbehaviour");

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

        // set the target height to the minimum between the update height and latest chain height
        let target_height = std::cmp::min(*update.consensus_height(), latest_chain_height);
        let trusted_height = tm_ibc_client_header.trusted_height;

        let tm_chain_header = {
            assert!(trusted_height < latest_chain_height);
            let trusted_light_block = self.verify_to_target(trusted_height)?;
            let target_light_block = self.verify_to_target(target_height)?;

            TmHeader {
                trusted_height,
                signed_header: target_light_block.signed_header.clone(),
                validator_set: target_light_block.validators,
                trusted_validator_set: trusted_light_block.validators,
            }
        };

        let misbehaviour =
            if LightClient::incompatible_headers(&tm_ibc_client_header, &tm_chain_header) {
                Some(
                    AnyMisbehaviour::Tendermint(TmMisbehaviour {
                        client_id: update.client_id().clone(),
                        header1: tm_ibc_client_header,
                        header2: tm_chain_header,
                    })
                    .wrap_any(),
                )
            } else {
                None
            };

   

///////////

        // let update_header = update.header.clone().ok_or_else(|| {
        //     Kind::Misbehaviour(format!(
        //         "missing header in update client event {}",
        //         self.chain_id
        //     ))
        // })?;

        // let mock_ibc_client_header =
        //     downcast!(update_header => AnyHeader::Mock).ok_or_else(|| {
        //         Kind::Misbehaviour(format!(
        //             "header type incompatible for chain {}",
        //             self.chain_id
        //         ))
        //     }   )?;

        // // set the target height to the minimum between the update height and latest chain height
        // let target_height = std::cmp::min(*update.consensus_height(), latest_chain_height);
        // //let trusted_height = mock_ibc_client_header.trusted_height;

        // let mock_chain_target_block = self.light_block(target_height); 

        // let mock_chain_header = MockHeader{
        //     height: Height::new(0,mock_chain_target_block.signed_header.header().height.into()),  
        //     timestamp: 10,
        // };
        
        // let misbehaviour =
        //     if LightClient::incompatible_headers(&mock_ibc_client_header, &mock_chain_header) {
        //         Some(
        //             AnyMisbehaviour::Mock(Misbehaviour {
        //                 client_id: update.client_id().clone(),
        //                 header1: mock_ibc_client_header,
        //                 header2: mock_chain_header,
        //             }),
        //         )
        //     } else {
        //         None
        //     };

    Ok(misbehaviour)

       // Ok(None)
        //unimplemented!()
    }
}
