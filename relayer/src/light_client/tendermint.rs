#[cfg(feature="prusti")]
use prusti_contracts::*;

use ibc::ics02_client::misbehaviour::AnyMisbehaviour;
use tendermint::block::signed_header::SignedHeader;
use tendermint_light_client::contracts::is_within_trust_period;

#[cfg(not(feature="prusti"))]
use tendermint_light_client::types::Time;

use std::convert::TryFrom;

use itertools::Itertools;

use tendermint_light_client::{
    components::{self, io::AtHeight},
    light_client::{LightClient as TmLightClient, Options as TmOptions},
    operations,
    state::State as LightClientState,
    store::{memory::MemoryStore, LightStore},
    types::Height as TMHeight,
    types::{LightBlock, PeerId, Status},
};
use tendermint_rpc as rpc;

use ibc::{
    downcast,
    handle_result,
    ics02_client::{
        client_state::AnyClientState,
        client_type::ClientType,
        events::UpdateClient,
        header::{AnyHeader, Header},
        misbehaviour::{Misbehaviour, MisbehaviourEvidence},
    },
    ics07_tendermint::{
        header::{headers_compatible, Header as TmHeader},
        misbehaviour::Misbehaviour as TmMisbehaviour,
    },
    ics24_host::identifier::ChainId,
};
use tracing::trace;

use crate::{chain::CosmosSdkChain, config::ChainConfig, error::Error};

use super::Verified;

pub struct LightClient {
    #[cfg(feature="prusti")]
    chain_id: u32,
    #[cfg(not(feature="prusti"))]
    chain_id: ChainId,
    peer_id: PeerId,
    io: components::io::ProdIo,
}

impl super::LightClient<CosmosSdkChain> for LightClient {
    #[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn header_and_minimal_set(
        &mut self,
        trusted: ibc::Height,
        target: ibc::Height,
        client_state: &AnyClientState,
    ) -> Result<Verified<TmHeader>, Error> {
        let Verified { target, supporting } = self.verify(trusted, target, client_state)?;
        let (target, supporting) = self.adjust_headers(trusted, target, supporting)?;
        Ok(Verified { target, supporting })
    }

    #[cfg_attr(feature="prusti", trusted_skip)]
    fn verify(
        &mut self,
        trusted: ibc::Height,
        target: ibc::Height,
        client_state: &AnyClientState,
    ) -> Result<Verified<LightBlock>, Error> {
        self.verify0(trusted, target, client_state)
    }

    #[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn fetch(&mut self, height: ibc::Height) -> Result<LightBlock, Error> {
        trace!(%height, "fetching header");

        let height = TMHeight::try_from(height.revision_height).map_err(Error::invalid_height)?;

        self.fetch_light_block(AtHeight::At(height))
    }

    /// Given a client update event that includes the header used in a client update,
    /// look for misbehaviour by fetching a header at same or latest height.
    ///
    /// ## TODO
    /// - [ ] Return intermediate headers as well
    // #[cfg_attr(feature="prusti", trusted_skip)]
    // #[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn check_misbehaviour(
        &mut self,
        update: UpdateClient,
        client_state: &AnyClientState,
    ) -> Result<Option<MisbehaviourEvidence>, Error> {
        self.check_misbehaviour(update, client_state)
    }
}

#[cfg(feature="prusti")]
#[extern_spec]
mod tendermint {
    mod Time {

        use prusti_contracts::*;
        use tendermint_light_client::types::Time;

        #[pure]
        pub fn now() -> Time;
    }
}

#[cfg(feature="prusti")]
type Duration = i32;

#[cfg(feature="prusti")]
type Time = i32;

#[cfg(not(feature="prusti"))]
type Duration = std::time::Duration;

#[cfg(feature="prusti")]
#[extern_spec]
impl ibc::ics02_client::client_state::AnyClientState {
    #[pure]
    #[ensures(result == cs_trusting_period(&self))]
    pub fn trusting_period(&self) -> Duration;
}

#[cfg(feature="prusti")]
#[extern_spec]
impl ibc::ics07_tendermint::header::Header {
    #[pure]
    pub fn time(&self) -> Time;
}

#[cfg_attr(feature="prusti", pure)]
pub fn cs_trusting_period(cs: &AnyClientState) -> i32 {
    match cs {
        AnyClientState::Tendermint(tm_state) => tm_state.trusting_period,
        _ => 0
    }
}

#[pure]
#[requires(r.is_ok())]
fn unwrap_verified(r: &Result<LightBlock, tendermint_light_client::errors::Error>) -> &LightBlock {
   match r {
      Ok(block) => block,
      Err(_) => unreachable!()
   }
}

#[cfg(feature="prusti")]
#[extern_spec]
impl tendermint_light_client::light_client::LightClient {

    #[ensures(result.options.trusting_period == options.trusting_period)]
    pub fn new(
       options: TmOptions
    ) -> tendermint_light_client::light_client::LightClient;

    #[ensures(
        result.is_ok() ==>
        is_within_trust_period(
          unwrap_verified(&result),
          self.options.trusting_period,
          0
        )
    )]
    pub fn verify_to_target(
        &self,
        target_height: tendermint::block::Height,
        state: &mut tendermint_light_client::state::State,
    ) -> Result<LightBlock, tendermint_light_client::errors::Error>;
}

#[cfg(feature="prusti")]
#[extern_spec]
mod tendermint_light_client {
    mod contracts {

        use prusti_contracts::*;
        use tendermint_light_client::types::Time;
        use tendermint_light_client::types::LightBlock;

        #[pure]
        pub fn is_within_trust_period(
            light_block: &LightBlock,
            trusting_period: i32,
            now: u32) -> bool;
    }
}

#[trusted]
#[ensures(v)]
fn assume(v: bool) {

}
#[trusted]
#[requires(v)]
fn assert(v: bool) {

}

#[cfg(feature="prusti")]
#[pure]
#[requires(matches!(v, Ok(_)))]
fn get_options(v: &Result<TmLightClient, Error>) -> TmOptions {
  match v {
      Ok(r) => r.options,
      Err(_) => unreachable!()
  }
}

#[cfg(feature="prusti")]
#[requires(v.is_ok())]
#[pure]
fn get_verified_target(v: &Result<Verified<LightBlock>, Error>) -> LightBlock {
  match v {
      Ok(r) => r.target,
      Err(_) => unreachable!()
  }
}

#[pure]
#[trusted]
fn get_verified_supporting_header_time(m: &Vec<LightBlock>, i: usize) -> Time {
    m[i].signed_header.header.time
}

#[trusted]
#[ensures(target.signed_header.header.time > get_verified_supporting_header_time(&result, 0))]
fn get_supporting(target: &LightBlock, state: &LightClientState) -> Vec<LightBlock> {
    // Collect the verification trace for the target block
    let target_trace = state.get_trace(target.height());

    // Compute the minimal supporting set, sorted by ascending height
    target_trace
        .into_iter()
        // .filter(|lb| lb.height() != target.height())
        .unique_by(LightBlock::height)
        .sorted_by_key(LightBlock::height)
        .collect_vec()
}

impl LightClient {

    // #[ensures(result.is_ok() ==>
    //     is_within_trust_period(
    //       &get_verified_target(&result),
    //       client_state.trusting_period(),
    //       0
    //     )
    //  )]
    #[ensures(verify_spec(&result))]
    fn verify0(
        &mut self,
        trusted: ibc::Height,
        target: ibc::Height,
        client_state: &AnyClientState,
    ) -> Result<Verified<LightBlock>, Error> {
        // trace!(%trusted, %target, "light client verification");

        let target_height =
           match TMHeight::try_from(target.revision_height).map_err(Error::invalid_height) {
               Err(e) => return Err(e),
               Ok(th) => th
           };

        let client = match self.prepare_client(client_state) {
           Err(e) => return Err(e),
           Ok(th) => th
        };

        let mut state = match self.prepare_state(trusted) {
           Err(e) => return Err(e),
           Ok(th) => th
        };

        // Verify the target header
        let target = match client.verify_to_target(target_height, &mut state) {
            Ok(t) => t,
            Err(e) => return Err(Error::light_client(self.chain_id.to_string(), e))
        };

        // Collect the verification trace for the target block
        // Compute the minimal supporting set, sorted by ascending height
        let supporting = get_supporting(&target, &state);

        Ok(Verified { target, supporting })
    }


    #[ensures(check_misbehaviour_spec(old(client_state), &result))]
    fn check_misbehaviour0(
        &mut self,
        update: UpdateClient,
        client_state: &AnyClientState,
    ) -> Result<Option<MisbehaviourEvidence>, Error> {
        // crate::time!("light client check_misbehaviour");

        let update_header = match update.header.clone() {
           Some(header) => header,
           None =>
            return Err(Error::misbehaviour("missing header in update client event".to_string()))
        };

        let update_header = match update_header {
            AnyHeader::Tendermint(header) => header,
            _ => return Err(Error::misbehaviour("header type incompatible for chain".to_string()))
        };

        /*
        let latest_chain_block = handle_result!(self.fetch_light_block(AtHeight::Highest));
        let latest_chain_height =
            ibc::Height::new(0, latest_chain_block.height().into());
        */
        let latest_chain_height = update_header.trusted_height;

        // set the target height to the minimum between the update height and latest chain height
        let target_height = std::cmp::min(update.consensus_height(), latest_chain_height);
        let trusted_height = update_header.trusted_height;

        // TODO - check that a consensus state at trusted_height still exists on-chain,
        // currently we don't have access to Cosmos chain query from here

        if trusted_height >= latest_chain_height {
            // Can happen with multiple FLA attacks, we return no evidence and hope to catch this in
            // the next iteration. e.g:
            // existing consensus states: 1000, 900, 300, 200 (only known by the caller)
            // latest_chain_height = 300
            // target_height = 1000
            // trusted_height = 900
            return Ok(None);
        }


        let Verified { target, supporting } =
            handle_result!(self.verify0(trusted_height, target_height, client_state));

        assert(target.signed_header.header.time > get_verified_supporting_header_time(&supporting, 0));

        if !headers_compatible(&target.signed_header, &update_header.signed_header) {
            let (witness, supporting) = handle_result!(self.adjust_headers(trusted_height, target, supporting));

            let misbehaviour = AnyMisbehaviour::Tendermint(TmMisbehaviour {
                client_id: 0,
                header1: update_header,
                header2: witness,
            });

            let result = MisbehaviourEvidence {
                misbehaviour,
                supporting_headers: to_any_headers(supporting)
            };

            Ok(Some(result))
        } else {
            Ok(None)
        }
    }
}

impl LightClient {
    #[cfg_attr(feature="prusti", trusted_skip)]
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

    // #[ensures(result.is_ok() ==> get_options(&result).trusting_period == client_state.trusting_period())]
    #[cfg_attr(feature="prusti", trusted_skip)]
    fn prepare_client(&self, client_state: &AnyClientState) -> Result<TmLightClient, Error> {
        // let clock = components::clock::SystemClock;
        // let hasher = operations::hasher::ProdHasher;
        // let verifier = components::verifier::ProdVerifier::default();
        // let scheduler = components::scheduler::basic_bisecting_schedule;

        let tcs = match client_state {
            AnyClientState::Tendermint(cs) => cs,
            _ =>  return Err(Error::client_type_mismatch(ClientType::Tendermint, client_state.client_type()))
        };

        let params = TmOptions {
            trust_threshold: tcs.trust_level,
            trusting_period: tcs.trusting_period,
            clock_drift: tcs.max_clock_drift,
        };

        assert(params.trusting_period == client_state.trusting_period());


        // Ok(TmLightClient::new(
        //     self.peer_id,
        //     params,
        //     clock,
        //     scheduler,
        //     verifier,
        //     hasher,
        //     self.io.clone(),
        // ))

        Ok(TmLightClient::new(params))
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn prepare_state(&self, trusted: ibc::Height) -> Result<LightClientState, Error> {
        let trusted_height =
            TMHeight::try_from(trusted.revision_height).map_err(Error::invalid_height)?;

        let trusted_block = self.fetch_light_block(AtHeight::At(trusted_height))?;

        let mut store = MemoryStore::new();
        store.insert(trusted_block, Status::Trusted);

        Ok(LightClientState::new(store))
    }

    // #[cfg_attr(feature="prusti", trusted_skip)]
    // fn fetch_light_block(&self, height: AtHeight) -> Result<LightBlock, Error> {
    //     use tendermint_light_client::components::io::Io;

    //     self.io
    //         .fetch_light_block(height)
    //         .map_err(|e| Error::light_client_io(self.chain_id.to_string(), e))
    // }



    #[ensures(adjust_headers_spec(old(&target), &result))]
    // #[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn adjust_headers(
        &mut self,
        trusted_height: ibc::Height,
        target: LightBlock,
        supporting: Vec<LightBlock>,
    ) -> Result<(TmHeader, Vec<TmHeader>), Error> {
        use super::LightClient;

        // trace!(
        //     trusted = %trusted_height, target = %target.height(),
        //     "adjusting headers with {} supporting headers", supporting.len()
        // );

        // Get the light block at trusted_height + 1 from chain.
        //
        // NOTE: This is needed to get the next validator set. While there is a next validator set
        //       in the light block at trusted height, the proposer is not known/set in this set.
        let trusted_validator_set = handle_result!(self.fetch(trusted_height.increment())).validators;

        let mut supporting_headers = Vec::with_capacity(supporting.len());

        let mut current_trusted_height = trusted_height;
        let mut current_trusted_validators = trusted_validator_set.clone();

        let mut i = 0;
        while i < supporting.len() {
            let support = supporting[i];
            let header = TmHeader {
                signed_header: support.signed_header.clone(),
                validator_set: support.validators,
                trusted_height: current_trusted_height,
                trusted_validator_set: current_trusted_validators,
            };

            // This header is now considered to be the currently trusted header
            current_trusted_height = header.height();

            // Therefore we can now trust the next validator set, see NOTE above.
            current_trusted_validators = handle_result!(self.fetch(header.height().increment())).validators;

            supporting_headers.push(header);
            i += 1;
        }

        // a) Set the trusted height of the target header to the height of the previous
        // supporting header if any, or to the initial trusting height otherwise.
        //
        // b) Set the trusted validators of the target header to the validators of the successor to
        // the last supporting header if any, or to the initial trusted validators otherwise.
        /*
        let (latest_trusted_height, latest_trusted_validator_set) = if supporting_headers.len() > 0 {
            let prev_header = &supporting_headers[i - 1];
            let prev_succ = handle_result!(self.fetch(prev_header.height().increment()));
            (prev_header.height(), prev_succ.validators)
        } else {
            (trusted_height, trusted_validator_set)
        };
        */
        let (latest_trusted_height, latest_trusted_validator_set) =
            (trusted_height, trusted_validator_set);
        let target_header = TmHeader {
            signed_header: target.signed_header,
            validator_set: target.validators,
            trusted_height: latest_trusted_height,
            trusted_validator_set: latest_trusted_validator_set,
        };

        assume(target.signed_header.header.time > get_header_time(&supporting_headers, 0));

        Ok((target_header, supporting_headers))
    }
}

// #[pure]
// fn get_witness(m: &AnyMisbehaviour) -> &TmHeader {
//    match m {
//        AnyMisbehaviour::Tendermint(t) => &t.header1,
//        _ => unreachable!()
//    }
// }

#[pure]
fn check_misbehaviour_spec(client_state: &AnyClientState, r: &Result<Option<MisbehaviourEvidence>, Error>) -> bool {
    match r {
        Ok(Some(m)) => misbehaviour_invariant(m), // header_within_trust_period(&get_witness(&m.misbehaviour), client_state.trusting_period(), 0),
        _ => true
    }
}

#[pure]
#[trusted]
fn get_header_time(headers: &Vec<TmHeader>, index: usize) -> Time {
    headers[index].signed_header.header.time
}

#[pure]
#[trusted]
fn get_supporting_header_time(headers: &Vec<AnyHeader>, index: usize) -> Time {
    match &headers[index] {
        AnyHeader::Tendermint(header) => header.signed_header.header.time,
        _ => unreachable!()
    }
}

#[pure]
#[requires(matches!(&m.misbehaviour, AnyMisbehaviour::Tendermint(_)))]
fn misbehaviour_invariant(m: &MisbehaviourEvidence) -> bool {
   let supporting_header_time = get_supporting_header_time(&m.supporting_headers, 0);
   let witness_time = match &m.misbehaviour {
       AnyMisbehaviour::Tendermint(t) => t.header2.signed_header.header.time,
       _ => unreachable!()
   };
   witness_time > supporting_header_time
}

#[pure]
fn header_within_trust_period(header: &TmHeader, trusting_period: Duration, now: Time) -> bool {
    true
    // let header_time = header.signed_header.header.time;
    // header_time > now - trusting_period
}

#[pure]
fn adjust_headers_spec(target: &LightBlock, r: &Result<(TmHeader, Vec<TmHeader>), Error>) -> bool {
    match r {
        Ok((header, sup)) => header.signed_header == target.signed_header &&
            header.signed_header.header.time > get_header_time(sup, 0),
        _ => true
    }
}

#[trusted]
#[ensures(get_header_time(old(&vec), 0) == get_supporting_header_time(&result, 0))]
fn to_any_headers(vec: Vec<TmHeader>) -> Vec<AnyHeader> {
    vec.into_iter().map(TmHeader::wrap_any).collect()
}


#[pure]
fn verify_spec(r: &Result<Verified<LightBlock>, Error>) -> bool {
   match r {
       Ok(v) => v.target.signed_header.header.time > get_verified_supporting_header_time(&v.supporting, 0),
       _ => true
   }
}
