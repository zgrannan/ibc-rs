use alloc::sync::Arc;
use bytes::{Buf, Bytes};
use core::{
    convert::{TryFrom, TryInto},
    future::Future, str::FromStr, time::Duration,
};
use num_bigint::BigInt;
use std::thread;
use bitcoin::hashes::hex::ToHex;
use tendermint::block::Height as TmHeight;
use tendermint::{
    abci::{Event, Path as TendermintABCIPath},
    node::info::TxIndexStatus,
};
use tendermint_light_client_verifier::types::LightBlock as TmLightBlock;
use tendermint_proto::Protobuf;
use tendermint_rpc::{
    endpoint::broadcast::tx_sync::Response, endpoint::status, Client, HttpClient, Order,
};
use tokio::runtime::Runtime as TokioRuntime;
use tonic::{codegen::http::Uri, metadata::AsciiMetadataValue};
use tracing::{error, span, warn, Level};
use ibc::clients::ics07_tendermint::consensus_state::ConsensusState as TMConsensusState;
use ibc::clients::ics07_tendermint::header::Header as TmHeader;
use ibc::core::ics02_client::client_consensus::{
    AnyConsensusState, AnyConsensusStateWithHeight,
};
use ibc::core::ics02_client::client_state::{AnyClientState, IdentifiedAnyClientState};
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::error::Error as ClientError;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::events as ChannelEvents;
use ibc::core::ics04_channel::packet::{Packet, Sequence};
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ChainId, ClientId, ConnectionId};
use ibc::core::ics24_host::path::{
    AcksPath, ChannelEndsPath, ClientConsensusStatePath, ClientStatePath,
    CommitmentsPath, ConnectionsPath, ReceiptsPath, SeqRecvsPath,
};
use ibc::core::ics24_host::{
    ClientUpgradePath, Path, IBC_QUERY_PATH, SDK_UPGRADE_QUERY_PATH,
};
use ibc::events::IbcEvent;
use ibc::signer::Signer;
use ibc::Height as ICSHeight;
use ibc::{
    clients::ics07_tendermint::client_state::{AllowUpdate, ClientState},
    core::ics23_commitment::merkle::MerkleProof,
};
use ibc_proto::cosmos::staking::v1beta1::Params as StakingParams;
use crate::account::Balance;
use crate::chain::client::ClientSettings;
use crate::chain::cosmos::batch::{
    send_batched_messages_and_wait_check_tx, send_batched_messages_and_wait_commit,
};
use crate::chain::cosmos::encode::encode_to_bech32;
use crate::chain::cosmos::gas::{calculate_fee, mul_ceil};
use crate::chain::cosmos::query::account::get_or_fetch_account;
use crate::chain::cosmos::query::balance::query_balance;
use crate::chain::cosmos::query::denom_trace::query_denom_trace;
use crate::chain::cosmos::query::status::query_status;
use crate::chain::cosmos::query::tx::query_txs;
use crate::chain::cosmos::query::{
    abci_query, fetch_version_specs, packet_query, QueryResponse,
};
use crate::chain::cosmos::types::account::Account;
use crate::chain::cosmos::types::config::TxConfig;
use crate::chain::cosmos::types::gas::{default_gas_from_config, max_gas_from_config};
use crate::chain::endpoint::{ChainEndpoint, ChainStatus, HealthCheck};
use crate::chain::tracking::TrackedMsgs;
use crate::config::ChainConfig;
use crate::denom::DenomTrace;
use crate::error::Error;
use crate::event::monitor::{EventMonitor, EventReceiver, TxMonitorCmd};
use crate::keyring::{KeyEntry, KeyRing};
use crate::light_client::tendermint::LightClient as TmLightClient;
use crate::light_client::{LightClient, Verified};
use super::requests::{
    IncludeProof, QueryBlockRequest, QueryChannelClientStateRequest, QueryChannelRequest,
    QueryChannelsRequest, QueryClientConnectionsRequest, QueryClientStateRequest,
    QueryClientStatesRequest, QueryConnectionChannelsRequest, QueryConnectionRequest,
    QueryConnectionsRequest, QueryConsensusStateRequest, QueryConsensusStatesRequest,
    QueryHeight, QueryHostConsensusStateRequest, QueryNextSequenceReceiveRequest,
    QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementsRequest,
    QueryPacketCommitmentRequest, QueryPacketCommitmentsRequest,
    QueryPacketEventDataRequest, QueryPacketReceiptRequest, QueryTxRequest,
    QueryUnreceivedAcksRequest, QueryUnreceivedPacketsRequest,
    QueryUpgradedClientStateRequest, QueryUpgradedConsensusStateRequest,
};
pub mod batch;
pub mod client;
pub mod compatibility;
pub mod encode;
pub mod estimate;
pub mod gas;
pub mod query;
pub mod retry;
pub mod simulate;
pub mod tx;
pub mod types;
pub mod version;
pub mod wait;
/// fraction of the maximum block size defined in the Tendermint core consensus parameters.
pub const GENESIS_MAX_BYTES_MAX_FRACTION: f64 = 0.9;
pub struct CosmosSdkChain {
    config: ChainConfig,
    tx_config: TxConfig,
    rpc_client: HttpClient,
    grpc_addr: Uri,
    rt: Arc<TokioRuntime>,
    keybase: KeyRing,
    /// A cached copy of the account information
    account: Option<Account>,
}
impl CosmosSdkChain {
    /// Get a reference to the configuration for this chain.
    #[prusti_contracts::trusted]
    pub fn config(&self) -> &ChainConfig {
        &self.config
    }
    /// Performs validation of chain-specific configuration
    /// parameters against the chain's genesis configuration.
    ///
    /// Currently, validates the following:
    ///     - the configured `max_tx_size` is appropriate
    ///     - the trusting period is greater than zero
    ///     - the trusting period is smaller than the unbonding period
    ///     - the default gas is smaller than the max gas
    ///
    /// Emits a log warning in case any error is encountered and
    /// exits early without doing subsequent validations.
    #[prusti_contracts::trusted]
    pub fn validate_params(&self) -> Result<(), Error> {
        let unbonding_period = self.unbonding_period()?;
        let trusting_period = self.trusting_period(unbonding_period);
        if trusting_period <= Duration::ZERO {
            return Err(
                Error::config_validation_trusting_period_smaller_than_zero(
                    self.id().clone(),
                    trusting_period,
                ),
            );
        }
        if trusting_period >= unbonding_period {
            return Err(
                Error::config_validation_trusting_period_greater_than_unbonding_period(
                    self.id().clone(),
                    trusting_period,
                    unbonding_period,
                ),
            );
        }
        let max_gas = max_gas_from_config(&self.config);
        let default_gas = default_gas_from_config(&self.config);
        if default_gas > max_gas {
            return Err(
                Error::config_validation_default_gas_too_high(
                    self.id().clone(),
                    default_gas,
                    max_gas,
                ),
            );
        }
        let latest_height = TmHeight::try_from(
                self.query_chain_latest_height()?.revision_height(),
            )
            .map_err(Error::invalid_height)?;
        let result = self
            .block_on(self.rpc_client.consensus_params(latest_height))
            .map_err(|e| {
                Error::config_validation_json_rpc(
                    self.id().clone(),
                    self.config.rpc_addr.to_string(),
                    "/consensus_params".to_string(),
                    e,
                )
            })?;
        let max_bound = result.consensus_params.block.max_bytes;
        let max_allowed = mul_ceil(max_bound, GENESIS_MAX_BYTES_MAX_FRACTION);
        let max_tx_size = BigInt::from(self.max_tx_size());
        if max_tx_size > max_allowed {
            return Err(
                Error::config_validation_tx_size_out_of_bounds(
                    self.id().clone(),
                    self.max_tx_size(),
                    max_bound,
                ),
            );
        }
        let consensus_max_gas = result.consensus_params.block.max_gas;
        if consensus_max_gas >= 0 {
            let consensus_max_gas: u64 = consensus_max_gas
                .try_into()
                .expect("cannot over or underflow because it is positive");
            let max_gas = max_gas_from_config(&self.config);
            if max_gas > consensus_max_gas {
                return Err(
                    Error::config_validation_max_gas_too_high(
                        self.id().clone(),
                        max_gas,
                        result.consensus_params.block.max_gas,
                    ),
                );
            }
        }
        Ok(())
    }
    /// Query the chain staking parameters
    #[prusti_contracts::trusted]
    pub fn query_staking_params(&self) -> Result<StakingParams, Error> {
        crate::time!("query_staking_params");
        crate::telemetry!(query, self.id(), "query_staking_params");
        let mut client = self
            .block_on(
                ibc_proto::cosmos::staking::v1beta1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(ibc_proto::cosmos::staking::v1beta1::QueryParamsRequest {});
        let response = self
            .block_on(client.params(request))
            .map_err(Error::grpc_status)?;
        let params = response
            .into_inner()
            .params
            .ok_or_else(|| Error::grpc_response_param("no staking params".to_string()))?;
        Ok(params)
    }
    /// The unbonding period of this chain
    #[prusti_contracts::trusted]
    pub fn unbonding_period(&self) -> Result<Duration, Error> {
        crate::time!("unbonding_period");
        let unbonding_time = self
            .query_staking_params()?
            .unbonding_time
            .ok_or_else(|| {
                Error::grpc_response_param(
                    "no unbonding time in staking params".to_string(),
                )
            })?;
        Ok(Duration::new(unbonding_time.seconds as u64, unbonding_time.nanos as u32))
    }
    /// The number of historical entries kept by this chain
    #[prusti_contracts::trusted]
    pub fn historical_entries(&self) -> Result<u32, Error> {
        crate::time!("historical_entries");
        self.query_staking_params().map(|p| p.historical_entries)
    }
    /// Run a future to completion on the Tokio runtime.
    #[prusti_contracts::trusted]
    fn block_on<F: Future>(&self, f: F) -> F::Output {
        crate::time!("block_on");
        self.rt.block_on(f)
    }
    /// The maximum size of any transaction sent by the relayer to this chain
    #[prusti_contracts::trusted]
    fn max_tx_size(&self) -> usize {
        self.config.max_tx_size.into()
    }
    #[prusti_contracts::trusted]
    fn query(
        &self,
        data: impl Into<Path>,
        height_query: QueryHeight,
        prove: bool,
    ) -> Result<QueryResponse, Error> {
        crate::time!("query");
        let path = TendermintABCIPath::from_str(IBC_QUERY_PATH)
            .expect("Turning IBC query path constant into a Tendermint ABCI path");
        let height = TmHeight::try_from(height_query)?;
        let data = data.into();
        if !data.is_provable() & prove {
            return Err(Error::private_store());
        }
        let response = self
            .block_on(
                abci_query(
                    &self.rpc_client,
                    &self.config.rpc_addr,
                    path,
                    data.to_string(),
                    height,
                    prove,
                ),
            )?;
        if prove {}
        Ok(response)
    }
    /// Perform an ABCI query against the client upgrade sub-store.
    ///
    /// The data is returned in its raw format `Vec<u8>`, and is either the
    /// client state (if the target path is [`UpgradedClientState`]), or the
    /// client consensus state ([`UpgradedClientConsensusState`]).
    ///
    /// Note: This is a special query in that it will only succeed if the chain
    /// is halted after reaching the height proposed in a successful governance
    /// proposal to upgrade the chain. In this scenario, let P be the height at
    /// which the chain is planned to upgrade. We assume that the chain is
    /// halted at height P. Tendermint will be at height P (as reported by the
    /// /status RPC query), but the application will be at height P-1 (as
    /// reported by the /abci_info RPC query).
    ///
    /// Therefore, `query_height` needs to be P-1. However, the path specified
    /// in `query_data` needs to be constructed with height `P`, as this is how
    /// the chain will have stored it in its upgrade sub-store.
    #[prusti_contracts::trusted]
    fn query_client_upgrade_state(
        &self,
        query_data: ClientUpgradePath,
        query_height: ibc::Height,
    ) -> Result<(Vec<u8>, MerkleProof), Error> {
        let path = TendermintABCIPath::from_str(SDK_UPGRADE_QUERY_PATH)
            .expect(
                "Turning SDK upgrade query path constant into a Tendermint ABCI path",
            );
        let response: QueryResponse = self
            .block_on(
                abci_query(
                    &self.rpc_client,
                    &self.config.rpc_addr,
                    path,
                    Path::Upgrade(query_data).to_string(),
                    TmHeight::try_from(query_height.revision_height())
                        .map_err(Error::invalid_height)?,
                    true,
                ),
            )?;
        let proof = response.proof.ok_or_else(Error::empty_response_proof)?;
        Ok((response.value, proof))
    }
    #[prusti_contracts::trusted]
    fn key(&self) -> Result<KeyEntry, Error> {
        self.keybase().get_key(&self.config.key_name).map_err(Error::key_base)
    }
    #[prusti_contracts::trusted]
    fn trusting_period(&self, unbonding_period: Duration) -> Duration {
        self.config.trusting_period.unwrap_or(2 * unbonding_period / 3)
    }
    /// Query the chain status via an RPC query.
    ///
    /// Returns an error if the node is still syncing and has not caught up,
    /// ie. if `sync_info.catching_up` is `true`.
    #[prusti_contracts::trusted]
    fn chain_status(&self) -> Result<status::Response, Error> {
        let status = self
            .block_on(self.rpc_client.status())
            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
        if status.sync_info.catching_up {
            return Err(
                Error::chain_not_caught_up(
                    self.config.rpc_addr.to_string(),
                    self.config().id.clone(),
                ),
            );
        }
        Ok(status)
    }
    /// Query the chain's latest height
    #[prusti_contracts::trusted]
    pub fn query_chain_latest_height(&self) -> Result<ICSHeight, Error> {
        crate::time!("query_latest_height");
        crate::telemetry!(query, self.id(), "query_latest_height");
        let status = self
            .rt
            .block_on(query_status(self.id(), &self.rpc_client, &self.config.rpc_addr))?;
        Ok(status.height)
    }
    #[prusti_contracts::trusted]
    async fn do_send_messages_and_wait_commit(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<IbcEvent>, Error> {
        crate::time!("send_messages_and_wait_commit");
        let _span = span!(
            Level::DEBUG, "send_tx_commit", id = % tracked_msgs.tracking_id()
        )
            .entered();
        let proto_msgs = tracked_msgs.msgs;
        let key_entry = self.key()?;
        let account = get_or_fetch_account(
                &self.grpc_addr,
                &key_entry.account,
                &mut self.account,
            )
            .await?;
        send_batched_messages_and_wait_commit(
                &self.tx_config,
                self.config.max_msg_num,
                self.config.max_tx_size,
                &key_entry,
                account,
                &self.config.memo_prefix,
                proto_msgs,
            )
            .await
    }
    #[prusti_contracts::trusted]
    async fn do_send_messages_and_wait_check_tx(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<Response>, Error> {
        crate::time!("send_messages_and_wait_check_tx");
        let span = span!(
            Level::DEBUG, "send_tx_check", id = % tracked_msgs.tracking_id()
        );
        let _enter = span.enter();
        let proto_msgs = tracked_msgs.msgs;
        let key_entry = self.key()?;
        let account = get_or_fetch_account(
                &self.grpc_addr,
                &key_entry.account,
                &mut self.account,
            )
            .await?;
        send_batched_messages_and_wait_check_tx(
                &self.tx_config,
                self.config.max_msg_num,
                self.config.max_tx_size,
                &key_entry,
                account,
                &self.config.memo_prefix,
                proto_msgs,
            )
            .await
    }
}
impl ChainEndpoint for CosmosSdkChain {
    type LightBlock = TmLightBlock;
    type Header = TmHeader;
    type ConsensusState = TMConsensusState;
    type ClientState = ClientState;
    type LightClient = TmLightClient;
    #[prusti_contracts::trusted]
    fn bootstrap(config: ChainConfig, rt: Arc<TokioRuntime>) -> Result<Self, Error> {
        let rpc_client = HttpClient::new(config.rpc_addr.clone())
            .map_err(|e| Error::rpc(config.rpc_addr.clone(), e))?;
        let keybase = KeyRing::new(
                config.key_store_type,
                &config.account_prefix,
                &config.id,
            )
            .map_err(Error::key_base)?;
        let grpc_addr = Uri::from_str(&config.grpc_addr.to_string())
            .map_err(|e| Error::invalid_uri(config.grpc_addr.to_string(), e))?;
        let tx_config = TxConfig::try_from(&config)?;
        let chain = Self {
            config,
            rpc_client,
            grpc_addr,
            rt,
            keybase,
            account: None,
            tx_config,
        };
        Ok(chain)
    }
    #[prusti_contracts::trusted]
    fn init_light_client(&self) -> Result<Self::LightClient, Error> {
        use tendermint_light_client_verifier::types::PeerId;
        crate::time!("init_light_client");
        let peer_id: PeerId = self
            .rt
            .block_on(self.rpc_client.status())
            .map(|s| s.node_info.id)
            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
        let light_client = TmLightClient::from_config(&self.config, peer_id)?;
        Ok(light_client)
    }
    #[prusti_contracts::trusted]
    fn init_event_monitor(
        &self,
        rt: Arc<TokioRuntime>,
    ) -> Result<(EventReceiver, TxMonitorCmd), Error> {
        crate::time!("init_event_monitor");
        let (mut event_monitor, event_receiver, monitor_tx) = EventMonitor::new(
                self.config.id.clone(),
                self.config.websocket_addr.clone(),
                rt,
            )
            .map_err(Error::event_monitor)?;
        event_monitor.subscribe().map_err(Error::event_monitor)?;
        thread::spawn(move || event_monitor.run());
        Ok((event_receiver, monitor_tx))
    }
    #[prusti_contracts::trusted]
    fn shutdown(self) -> Result<(), Error> {
        Ok(())
    }
    #[prusti_contracts::trusted]
    fn id(&self) -> &ChainId {
        &self.config().id
    }
    #[prusti_contracts::trusted]
    fn keybase(&self) -> &KeyRing {
        &self.keybase
    }
    #[prusti_contracts::trusted]
    fn keybase_mut(&mut self) -> &mut KeyRing {
        &mut self.keybase
    }
    /// Does multiple RPC calls to the full node, to check for
    /// reachability and some basic APIs are available.
    ///
    /// Currently this checks that:
    ///     - the node responds OK to `/health` RPC call;
    ///     - the node has transaction indexing enabled;
    ///     - the SDK version is supported;
    ///
    /// Emits a log warning in case anything is amiss.
    /// Exits early if any health check fails, without doing any
    /// further checks.
    #[prusti_contracts::trusted]
    fn health_check(&self) -> Result<HealthCheck, Error> {
        if let Err(e) = do_health_check(self) {
            warn!("Health checkup for chain '{}' failed", self.id());
            warn!("    Reason: {}", e.detail());
            warn!("    Some Hermes features may not work in this mode!");
            return Ok(HealthCheck::Unhealthy(Box::new(e)));
        }
        if let Err(e) = self.validate_params() {
            warn!("Hermes might be misconfigured for chain '{}'", self.id());
            warn!("    Reason: {}", e.detail());
            warn!("    Some Hermes features may not work in this mode!");
            return Ok(HealthCheck::Unhealthy(Box::new(e)));
        }
        Ok(HealthCheck::Healthy)
    }
    /// Send one or more transactions that include all the specified messages.
    /// The `proto_msgs` are split in transactions such they don't exceed the configured maximum
    /// number of messages per transaction and the maximum transaction size.
    /// Then `send_tx()` is called with each Tx. `send_tx()` determines the fee based on the
    /// on-chain simulation and if this exceeds the maximum gas specified in the configuration file
    /// then it returns error.
    /// TODO - more work is required here for a smarter split maybe iteratively accumulating/ evaluating
    /// msgs in a Tx until any of the max size, max num msgs, max fee are exceeded.
    #[prusti_contracts::trusted]
    fn send_messages_and_wait_commit(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<IbcEvent>, Error> {
        let runtime = self.rt.clone();
        runtime.block_on(self.do_send_messages_and_wait_commit(tracked_msgs))
    }
    #[prusti_contracts::trusted]
    fn send_messages_and_wait_check_tx(
        &mut self,
        tracked_msgs: TrackedMsgs,
    ) -> Result<Vec<Response>, Error> {
        let runtime = self.rt.clone();
        runtime.block_on(self.do_send_messages_and_wait_check_tx(tracked_msgs))
    }
    /// Get the account for the signer
    #[prusti_contracts::trusted]
    fn get_signer(&mut self) -> Result<Signer, Error> {
        crate::time!("get_signer");
        let key = self
            .keybase()
            .get_key(&self.config.key_name)
            .map_err(|e| Error::key_not_found(self.config.key_name.clone(), e))?;
        let bech32 = encode_to_bech32(
            &key.address.to_hex(),
            &self.config.account_prefix,
        )?;
        bech32.parse().map_err(|e| Error::ics02(ClientError::signer(e)))
    }
    /// Get the chain configuration
    #[prusti_contracts::trusted]
    fn config(&self) -> ChainConfig {
        self.config.clone()
    }
    /// Get the signing key
    #[prusti_contracts::trusted]
    fn get_key(&mut self) -> Result<KeyEntry, Error> {
        crate::time!("get_key");
        let key = self
            .keybase()
            .get_key(&self.config.key_name)
            .map_err(|e| Error::key_not_found(self.config.key_name.clone(), e))?;
        Ok(key)
    }
    #[prusti_contracts::trusted]
    fn add_key(&mut self, key_name: &str, key: KeyEntry) -> Result<(), Error> {
        self.keybase_mut().add_key(key_name, key).map_err(Error::key_base)?;
        Ok(())
    }
    #[prusti_contracts::trusted]
    fn ibc_version(&self) -> Result<Option<semver::Version>, Error> {
        let version_specs = self
            .block_on(fetch_version_specs(self.id(), &self.grpc_addr))?;
        Ok(version_specs.ibc_go_version)
    }
    #[prusti_contracts::trusted]
    fn query_balance(&self, key_name: Option<String>) -> Result<Balance, Error> {
        let account = match key_name {
            Some(account) => {
                let key = self.keybase().get_key(&account).map_err(Error::key_base)?;
                key.account
            }
            _ => {
                let key = self.key()?;
                key.account
            }
        };
        let balance = self
            .block_on(
                query_balance(&self.grpc_addr, &account, &self.config.gas_price.denom),
            )?;
        Ok(balance)
    }
    #[prusti_contracts::trusted]
    fn query_denom_trace(&self, hash: String) -> Result<DenomTrace, Error> {
        let denom_trace = self.block_on(query_denom_trace(&self.grpc_addr, &hash))?;
        Ok(denom_trace)
    }
    #[prusti_contracts::trusted]
    fn query_commitment_prefix(&self) -> Result<CommitmentPrefix, Error> {
        crate::time!("query_commitment_prefix");
        crate::telemetry!(query, self.id(), "query_commitment_prefix");
        CommitmentPrefix::try_from(self.config().store_prefix.as_bytes().to_vec())
            .map_err(|_| Error::ics02(ClientError::empty_prefix()))
    }
    /// Query the application status
    #[prusti_contracts::trusted]
    fn query_application_status(&self) -> Result<ChainStatus, Error> {
        crate::time!("query_application_status");
        crate::telemetry!(query, self.id(), "query_application_status");
        let abci_info = self
            .block_on(self.rpc_client.abci_info())
            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
        let blocks = self
            .block_on(
                self
                    .rpc_client
                    .blockchain(abci_info.last_block_height, abci_info.last_block_height),
            )
            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?
            .block_metas;
        return if let Some(latest_app_block) = blocks.first() {
            let height = ICSHeight::new(
                    ChainId::chain_version(latest_app_block.header.chain_id.as_str()),
                    u64::from(abci_info.last_block_height),
                )
                .map_err(|_| Error::invalid_height_no_source())?;
            let timestamp = latest_app_block.header.time.into();
            Ok(ChainStatus { height, timestamp })
        } else {
            Err(Error::query("/blockchain endpoint for latest app. block".to_owned()))
        };
    }
    #[prusti_contracts::trusted]
    fn query_clients(
        &self,
        request: QueryClientStatesRequest,
    ) -> Result<Vec<IdentifiedAnyClientState>, Error> {
        crate::time!("query_clients");
        crate::telemetry!(query, self.id(), "query_clients");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::client::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.client_states(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let mut clients: Vec<IdentifiedAnyClientState> = response
            .client_states
            .into_iter()
            .filter_map(|cs| IdentifiedAnyClientState::try_from(cs).ok())
            .collect();
        clients.sort_by_cached_key(|c| client_id_suffix(&c.client_id).unwrap_or(0));
        Ok(clients)
    }
    #[prusti_contracts::trusted]
    fn query_client_state(
        &self,
        request: QueryClientStateRequest,
        include_proof: IncludeProof,
    ) -> Result<(AnyClientState, Option<MerkleProof>), Error> {
        crate::time!("query_client_state");
        crate::telemetry!(query, self.id(), "query_client_state");
        let res = self
            .query(
                ClientStatePath(request.client_id.clone()),
                request.height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        let client_state = AnyClientState::decode_vec(&res.value)
            .map_err(Error::decode)?;
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((client_state, Some(proof)))
            }
            IncludeProof::No => Ok((client_state, None)),
        }
    }
    #[prusti_contracts::trusted]
    fn query_upgraded_client_state(
        &self,
        request: QueryUpgradedClientStateRequest,
    ) -> Result<(AnyClientState, MerkleProof), Error> {
        crate::time!("query_upgraded_client_state");
        crate::telemetry!(query, self.id(), "query_upgraded_client_state");
        let upgrade_height = request.upgrade_height;
        let query_height = upgrade_height
            .decrement()
            .map_err(|_| Error::invalid_height_no_source())?;
        let (upgraded_client_state_raw, proof) = self
            .query_client_upgrade_state(
                ClientUpgradePath::UpgradedClientState(upgrade_height.revision_height()),
                query_height,
            )?;
        let client_state = AnyClientState::decode_vec(&upgraded_client_state_raw)
            .map_err(Error::conversion_from_any)?;
        Ok((client_state, proof))
    }
    #[prusti_contracts::trusted]
    fn query_upgraded_consensus_state(
        &self,
        request: QueryUpgradedConsensusStateRequest,
    ) -> Result<(AnyConsensusState, MerkleProof), Error> {
        crate::time!("query_upgraded_consensus_state");
        crate::telemetry!(query, self.id(), "query_upgraded_consensus_state");
        let upgrade_height = request.upgrade_height;
        let query_height = upgrade_height
            .decrement()
            .map_err(|_| Error::invalid_height_no_source())?;
        let (upgraded_consensus_state_raw, proof) = self
            .query_client_upgrade_state(
                ClientUpgradePath::UpgradedClientConsensusState(
                    upgrade_height.revision_height(),
                ),
                query_height,
            )?;
        let consensus_state = AnyConsensusState::decode_vec(
                &upgraded_consensus_state_raw,
            )
            .map_err(Error::conversion_from_any)?;
        Ok((consensus_state, proof))
    }
    /// Performs a query to retrieve the identifiers of all connections.
    #[prusti_contracts::trusted]
    fn query_consensus_states(
        &self,
        request: QueryConsensusStatesRequest,
    ) -> Result<Vec<AnyConsensusStateWithHeight>, Error> {
        crate::time!("query_consensus_states");
        crate::telemetry!(query, self.id(), "query_consensus_states");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::client::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.consensus_states(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let mut consensus_states: Vec<AnyConsensusStateWithHeight> = response
            .consensus_states
            .into_iter()
            .filter_map(|cs| TryFrom::try_from(cs).ok())
            .collect();
        consensus_states.sort_by(|a, b| a.height.cmp(&b.height));
        consensus_states.reverse();
        Ok(consensus_states)
    }
    #[prusti_contracts::trusted]
    fn query_consensus_state(
        &self,
        request: QueryConsensusStateRequest,
        include_proof: IncludeProof,
    ) -> Result<(AnyConsensusState, Option<MerkleProof>), Error> {
        crate::time!("query_consensus_state");
        crate::telemetry!(query, self.id(), "query_consensus_state");
        let res = self
            .query(
                ClientConsensusStatePath {
                    client_id: request.client_id.clone(),
                    epoch: request.consensus_height.revision_number(),
                    height: request.consensus_height.revision_height(),
                },
                request.query_height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        let consensus_state = AnyConsensusState::decode_vec(&res.value)
            .map_err(Error::decode)?;
        if !matches!(consensus_state, AnyConsensusState::Tendermint(_)) {
            return Err(
                Error::consensus_state_type_mismatch(
                    ClientType::Tendermint,
                    consensus_state.client_type(),
                ),
            );
        }
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((consensus_state, Some(proof)))
            }
            IncludeProof::No => Ok((consensus_state, None)),
        }
    }
    #[prusti_contracts::trusted]
    fn query_client_connections(
        &self,
        request: QueryClientConnectionsRequest,
    ) -> Result<Vec<ConnectionId>, Error> {
        crate::time!("query_client_connections");
        crate::telemetry!(query, self.id(), "query_client_connections");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::connection::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = match self.block_on(client.client_connections(request)) {
            Ok(res) => res.into_inner(),
            Err(e) if e.code() == tonic::Code::NotFound => return Ok(vec![]),
            Err(e) => return Err(Error::grpc_status(e)),
        };
        let ids = response
            .connection_paths
            .iter()
            .filter_map(|id| ConnectionId::from_str(id).ok())
            .collect();
        Ok(ids)
    }
    #[prusti_contracts::trusted]
    fn query_connections(
        &self,
        request: QueryConnectionsRequest,
    ) -> Result<Vec<IdentifiedConnectionEnd>, Error> {
        crate::time!("query_connections");
        crate::telemetry!(query, self.id(), "query_connections");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::connection::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.connections(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let connections = response
            .connections
            .into_iter()
            .filter_map(|co| IdentifiedConnectionEnd::try_from(co).ok())
            .collect();
        Ok(connections)
    }
    #[prusti_contracts::trusted]
    fn query_connection(
        &self,
        request: QueryConnectionRequest,
        include_proof: IncludeProof,
    ) -> Result<(ConnectionEnd, Option<MerkleProof>), Error> {
        crate::time!("query_connection");
        crate::telemetry!(query, self.id(), "query_connection");
        async fn do_query_connection(
            chain: &CosmosSdkChain,
            connection_id: &ConnectionId,
            height_query: QueryHeight,
        ) -> Result<ConnectionEnd, Error> {
            use ibc_proto::ibc::core::connection::v1 as connection;
            use tonic::IntoRequest;
            let mut client = connection::query_client::QueryClient::connect(
                    chain.grpc_addr.clone(),
                )
                .await
                .map_err(Error::grpc_transport)?;
            let mut request = connection::QueryConnectionRequest {
                connection_id: connection_id.to_string(),
            }
                .into_request();
            let height_param = AsciiMetadataValue::try_from(height_query)?;
            request.metadata_mut().insert("x-cosmos-block-height", height_param);
            let response = client
                .connection(request)
                .await
                .map_err(|e| {
                    if e.code() == tonic::Code::NotFound {
                        Error::connection_not_found(connection_id.clone())
                    } else {
                        Error::grpc_status(e)
                    }
                })?;
            match response.into_inner().connection {
                Some(raw_connection) => {
                    let connection_end = raw_connection
                        .try_into()
                        .map_err(Error::ics03)?;
                    Ok(connection_end)
                }
                None => Err(Error::connection_not_found(connection_id.clone())),
            }
        }
        match include_proof {
            IncludeProof::Yes => {
                let res = self
                    .query(
                        ConnectionsPath(request.connection_id.clone()),
                        request.height,
                        true,
                    )?;
                let connection_end = ConnectionEnd::decode_vec(&res.value)
                    .map_err(Error::decode)?;
                Ok((
                    connection_end,
                    Some(res.proof.ok_or_else(Error::empty_response_proof)?),
                ))
            }
            IncludeProof::No => {
                self
                    .block_on(async {
                        do_query_connection(self, &request.connection_id, request.height)
                            .await
                    })
                    .map(|conn_end| (conn_end, None))
            }
        }
    }
    #[prusti_contracts::trusted]
    fn query_connection_channels(
        &self,
        request: QueryConnectionChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error> {
        crate::time!("query_connection_channels");
        crate::telemetry!(query, self.id(), "query_connection_channels");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.connection_channels(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let channels = response
            .channels
            .into_iter()
            .filter_map(|ch| IdentifiedChannelEnd::try_from(ch).ok())
            .collect();
        Ok(channels)
    }
    #[prusti_contracts::trusted]
    fn query_channels(
        &self,
        request: QueryChannelsRequest,
    ) -> Result<Vec<IdentifiedChannelEnd>, Error> {
        crate::time!("query_channels");
        crate::telemetry!(query, self.id(), "query_channels");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.channels(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let channels = response
            .channels
            .into_iter()
            .filter_map(|ch| IdentifiedChannelEnd::try_from(ch).ok())
            .collect();
        Ok(channels)
    }
    #[prusti_contracts::trusted]
    fn query_channel(
        &self,
        request: QueryChannelRequest,
        include_proof: IncludeProof,
    ) -> Result<(ChannelEnd, Option<MerkleProof>), Error> {
        crate::time!("query_channel");
        crate::telemetry!(query, self.id(), "query_channel");
        let res = self
            .query(
                ChannelEndsPath(request.port_id, request.channel_id),
                request.height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        let channel_end = ChannelEnd::decode_vec(&res.value).map_err(Error::decode)?;
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((channel_end, Some(proof)))
            }
            IncludeProof::No => Ok((channel_end, None)),
        }
    }
    #[prusti_contracts::trusted]
    fn query_channel_client_state(
        &self,
        request: QueryChannelClientStateRequest,
    ) -> Result<Option<IdentifiedAnyClientState>, Error> {
        crate::time!("query_channel_client_state");
        crate::telemetry!(query, self.id(), "query_channel_client_state");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.channel_client_state(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let client_state: Option<IdentifiedAnyClientState> = response
            .identified_client_state
            .map_or_else(|| None, |proto_cs| proto_cs.try_into().ok());
        Ok(client_state)
    }
    #[prusti_contracts::trusted]
    fn query_packet_commitment(
        &self,
        request: QueryPacketCommitmentRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        let res = self
            .query(
                CommitmentsPath {
                    port_id: request.port_id,
                    channel_id: request.channel_id,
                    sequence: request.sequence,
                },
                request.height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((res.value, Some(proof)))
            }
            IncludeProof::No => Ok((res.value, None)),
        }
    }
    /// Queries the packet commitment hashes associated with a channel.
    #[prusti_contracts::trusted]
    fn query_packet_commitments(
        &self,
        request: QueryPacketCommitmentsRequest,
    ) -> Result<(Vec<Sequence>, ICSHeight), Error> {
        crate::time!("query_packet_commitments");
        crate::telemetry!(query, self.id(), "query_packet_commitments");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.packet_commitments(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let mut commitment_sequences: Vec<Sequence> = response
            .commitments
            .into_iter()
            .map(|v| v.sequence.into())
            .collect();
        commitment_sequences.sort_unstable();
        let height = response
            .height
            .and_then(|raw_height| raw_height.try_into().ok())
            .ok_or_else(|| Error::grpc_response_param("height".to_string()))?;
        Ok((commitment_sequences, height))
    }
    #[prusti_contracts::trusted]
    fn query_packet_receipt(
        &self,
        request: QueryPacketReceiptRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        let res = self
            .query(
                ReceiptsPath {
                    port_id: request.port_id,
                    channel_id: request.channel_id,
                    sequence: request.sequence,
                },
                request.height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((res.value, Some(proof)))
            }
            IncludeProof::No => Ok((res.value, None)),
        }
    }
    /// Queries the unreceived packet sequences associated with a channel.
    #[prusti_contracts::trusted]
    fn query_unreceived_packets(
        &self,
        request: QueryUnreceivedPacketsRequest,
    ) -> Result<Vec<Sequence>, Error> {
        crate::time!("query_unreceived_packets");
        crate::telemetry!(query, self.id(), "query_unreceived_packets");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let mut response = self
            .block_on(client.unreceived_packets(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        response.sequences.sort_unstable();
        Ok(response.sequences.into_iter().map(|seq| seq.into()).collect())
    }
    #[prusti_contracts::trusted]
    fn query_packet_acknowledgement(
        &self,
        request: QueryPacketAcknowledgementRequest,
        include_proof: IncludeProof,
    ) -> Result<(Vec<u8>, Option<MerkleProof>), Error> {
        let res = self
            .query(
                AcksPath {
                    port_id: request.port_id,
                    channel_id: request.channel_id,
                    sequence: request.sequence,
                },
                request.height,
                matches!(include_proof, IncludeProof::Yes),
            )?;
        match include_proof {
            IncludeProof::Yes => {
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((res.value, Some(proof)))
            }
            IncludeProof::No => Ok((res.value, None)),
        }
    }
    /// Queries the packet acknowledgment hashes associated with a channel.
    #[prusti_contracts::trusted]
    fn query_packet_acknowledgements(
        &self,
        request: QueryPacketAcknowledgementsRequest,
    ) -> Result<(Vec<Sequence>, ICSHeight), Error> {
        crate::time!("query_packet_acknowledgements");
        crate::telemetry!(query, self.id(), "query_packet_acknowledgements");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let response = self
            .block_on(client.packet_acknowledgements(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        let acks_sequences = response
            .acknowledgements
            .into_iter()
            .map(|v| v.sequence.into())
            .collect();
        let height = response
            .height
            .and_then(|raw_height| raw_height.try_into().ok())
            .ok_or_else(|| Error::grpc_response_param("height".to_string()))?;
        Ok((acks_sequences, height))
    }
    /// Queries the unreceived acknowledgements sequences associated with a channel.
    #[prusti_contracts::trusted]
    fn query_unreceived_acknowledgements(
        &self,
        request: QueryUnreceivedAcksRequest,
    ) -> Result<Vec<Sequence>, Error> {
        crate::time!("query_unreceived_acknowledgements");
        crate::telemetry!(query, self.id(), "query_unreceived_acknowledgements");
        let mut client = self
            .block_on(
                ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                    self.grpc_addr.clone(),
                ),
            )
            .map_err(Error::grpc_transport)?;
        let request = tonic::Request::new(request.into());
        let mut response = self
            .block_on(client.unreceived_acks(request))
            .map_err(Error::grpc_status)?
            .into_inner();
        response.sequences.sort_unstable();
        Ok(response.sequences.into_iter().map(|seq| seq.into()).collect())
    }
    #[prusti_contracts::trusted]
    fn query_next_sequence_receive(
        &self,
        request: QueryNextSequenceReceiveRequest,
        include_proof: IncludeProof,
    ) -> Result<(Sequence, Option<MerkleProof>), Error> {
        crate::time!("query_next_sequence_receive");
        crate::telemetry!(query, self.id(), "query_next_sequence_receive");
        match include_proof {
            IncludeProof::Yes => {
                let res = self
                    .query(
                        SeqRecvsPath(request.port_id, request.channel_id),
                        request.height,
                        true,
                    )?;
                if res.value.len() != 8 {
                    return Err(Error::query("next_sequence_receive".into()));
                }
                let seq: Sequence = Bytes::from(res.value).get_u64().into();
                let proof = res.proof.ok_or_else(Error::empty_response_proof)?;
                Ok((seq, Some(proof)))
            }
            IncludeProof::No => {
                let mut client = self
                    .block_on(
                        ibc_proto::ibc::core::channel::v1::query_client::QueryClient::connect(
                            self.grpc_addr.clone(),
                        ),
                    )
                    .map_err(Error::grpc_transport)?;
                let request = tonic::Request::new(request.into());
                let response = self
                    .block_on(client.next_sequence_receive(request))
                    .map_err(Error::grpc_status)?
                    .into_inner();
                Ok((Sequence::from(response.next_sequence_receive), None))
            }
        }
    }
    /// This function queries transactions for events matching certain criteria.
    /// 1. Client Update request - returns a vector with at most one update client event
    /// 2. Packet event request - returns at most one packet event for each sequence specified
    ///    in the request.
    ///    Note - there is no way to format the packet query such that it asks for Tx-es with either
    ///    sequence (the query conditions can only be AND-ed).
    ///    There is a possibility to include "<=" and ">=" conditions but it doesn't work with
    ///    string attributes (sequence is emmitted as a string).
    ///    Therefore, for packets we perform one tx_search for each sequence.
    ///    Alternatively, a single query for all packets could be performed but it would return all
    ///    packets ever sent.
    #[prusti_contracts::trusted]
    fn query_txs(&self, request: QueryTxRequest) -> Result<Vec<IbcEvent>, Error> {
        crate::time!("query_txs");
        crate::telemetry!(query, self.id(), "query_txs");
        self.block_on(
            query_txs(self.id(), &self.rpc_client, &self.config.rpc_addr, request),
        )
    }
    #[prusti_contracts::trusted]
    fn query_blocks(
        &self,
        request: QueryBlockRequest,
    ) -> Result<(Vec<IbcEvent>, Vec<IbcEvent>), Error> {
        crate::time!("query_blocks");
        crate::telemetry!(query, self.id(), "query_blocks");
        match request {
            QueryBlockRequest::Packet(request) => {
                crate::time!("query_blocks: query block packet events");
                let mut begin_block_events: Vec<IbcEvent> = vec![];
                let mut end_block_events: Vec<IbcEvent> = vec![];
                for seq in &request.sequences {
                    let response = self
                        .block_on(
                            self
                                .rpc_client
                                .block_search(
                                    packet_query(&request, *seq),
                                    1,
                                    1,
                                    Order::Ascending,
                                ),
                        )
                        .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
                    assert!(
                        response.blocks.len() <= 1,
                        "block_results: unexpected number of blocks"
                    );
                    if let Some(block)
                        = response.blocks.first().map(|first| &first.block)
                    {
                        let response_height = ICSHeight::new(
                                self.id().version(),
                                u64::from(block.header.height),
                            )
                            .map_err(|_| Error::invalid_height_no_source())?;
                        if let QueryHeight::Specific(query_height) = request.height {
                            if response_height > query_height {
                                continue;
                            }
                        }
                        let response = self
                            .block_on(self.rpc_client.block_results(block.header.height))
                            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
                        begin_block_events
                            .append(
                                &mut response
                                    .begin_block_events
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter_map(|ev| filter_matching_event(ev, &request, *seq))
                                    .collect(),
                            );
                        end_block_events
                            .append(
                                &mut response
                                    .end_block_events
                                    .unwrap_or_default()
                                    .into_iter()
                                    .filter_map(|ev| filter_matching_event(ev, &request, *seq))
                                    .collect(),
                            );
                    }
                }
                Ok((begin_block_events, end_block_events))
            }
        }
    }
    #[prusti_contracts::trusted]
    fn query_host_consensus_state(
        &self,
        request: QueryHostConsensusStateRequest,
    ) -> Result<Self::ConsensusState, Error> {
        let height = match request.height {
            QueryHeight::Latest => TmHeight::from(0u32),
            QueryHeight::Specific(ibc_height) => {
                TmHeight::try_from(ibc_height.revision_height())
                    .map_err(Error::invalid_height)?
            }
        };
        let rpc_call = match height.value() {
            0 => self.rpc_client.latest_block(),
            _ => self.rpc_client.block(height),
        };
        let response = self
            .block_on(rpc_call)
            .map_err(|e| Error::rpc(self.config.rpc_addr.clone(), e))?;
        Ok(response.block.header.into())
    }
    #[prusti_contracts::trusted]
    fn build_client_state(
        &self,
        height: ICSHeight,
        settings: ClientSettings,
    ) -> Result<Self::ClientState, Error> {
        let ClientSettings::Tendermint(settings) = settings;
        let unbonding_period = self.unbonding_period()?;
        let trusting_period = settings
            .trusting_period
            .unwrap_or_else(|| self.trusting_period(unbonding_period));
        ClientState::new(
                self.id().clone(),
                settings.trust_threshold,
                trusting_period,
                unbonding_period,
                settings.max_clock_drift,
                height,
                self.config.proof_specs.clone(),
                vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
                AllowUpdate {
                    after_expiry: true,
                    after_misbehaviour: true,
                },
            )
            .map_err(Error::ics07)
    }
    #[prusti_contracts::trusted]
    fn build_consensus_state(
        &self,
        light_block: Self::LightBlock,
    ) -> Result<Self::ConsensusState, Error> {
        crate::time!("build_consensus_state");
        Ok(TMConsensusState::from(light_block.signed_header.header))
    }
    #[prusti_contracts::trusted]
    fn build_header(
        &self,
        trusted_height: ICSHeight,
        target_height: ICSHeight,
        client_state: &AnyClientState,
        light_client: &mut Self::LightClient,
    ) -> Result<(Self::Header, Vec<Self::Header>), Error> {
        crate::time!("build_header");
        let Verified { target, supporting } = light_client
            .header_and_minimal_set(trusted_height, target_height, client_state)?;
        Ok((target, supporting))
    }
}
#[prusti_contracts::trusted]
fn filter_matching_event(
    event: Event,
    request: &QueryPacketEventDataRequest,
    seq: Sequence,
) -> Option<IbcEvent> {
    fn matches_packet(
        request: &QueryPacketEventDataRequest,
        seq: Sequence,
        packet: &Packet,
    ) -> bool {
        packet.source_port == request.source_port_id
            && packet.source_channel == request.source_channel_id
            && packet.destination_port == request.destination_port_id
            && packet.destination_channel == request.destination_channel_id
            && packet.sequence == seq
    }
    if event.type_str != request.event_id.as_str() {
        return None;
    }
    let ibc_event = ChannelEvents::try_from_tx(&event)?;
    match ibc_event {
        IbcEvent::SendPacket(
            ref send_ev,
        ) if matches_packet(request, seq, &send_ev.packet) => Some(ibc_event),
        IbcEvent::WriteAcknowledgement(
            ref ack_ev,
        ) if matches_packet(request, seq, &ack_ev.packet) => Some(ibc_event),
        _ => None,
    }
}
/// Returns the suffix counter for a CosmosSDK client id.
/// Returns `None` if the client identifier is malformed
/// and the suffix could not be parsed.
#[prusti_contracts::trusted]
fn client_id_suffix(client_id: &ClientId) -> Option<u64> {
    client_id.as_str().split('-').last().and_then(|e| e.parse::<u64>().ok())
}
#[prusti_contracts::trusted]
fn do_health_check(chain: &CosmosSdkChain) -> Result<(), Error> {
    let chain_id = chain.id();
    let grpc_address = chain.grpc_addr.to_string();
    let rpc_address = chain.config.rpc_addr.to_string();
    chain
        .block_on(chain.rpc_client.health())
        .map_err(|e| {
            Error::health_check_json_rpc(
                chain_id.clone(),
                rpc_address.clone(),
                "/health".to_string(),
                e,
            )
        })?;
    if chain.historical_entries()? == 0 {
        return Err(Error::no_historical_entries(chain_id.clone()));
    }
    let status = chain.chain_status()?;
    if status.node_info.other.tx_index != TxIndexStatus::On {
        return Err(Error::tx_indexing_disabled(chain_id.clone()));
    }
    if !status.node_info.network.as_str().eq(chain_id.as_str()) {
        error!(
            "/status endpoint from chain id '{}' reports network identifier to be '{}': this is usually a sign of misconfiguration, check your config.toml",
            chain_id, status.node_info.network
        );
    }
    let version_specs = chain
        .block_on(fetch_version_specs(&chain.config.id, &chain.grpc_addr))?;
    if let Err(diagnostic) = compatibility::run_diagnostic(&version_specs) {
        return Err(
            Error::sdk_module_version(
                chain_id.clone(),
                grpc_address,
                diagnostic.to_string(),
            ),
        );
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use ibc::{
        core::{
            ics02_client::client_state::{AnyClientState, IdentifiedAnyClientState},
            ics02_client::client_type::ClientType, ics24_host::identifier::ClientId,
        },
        mock::client_state::MockClientState, mock::header::MockHeader, Height,
    };
    use crate::{chain::cosmos::client_id_suffix, config::GasPrice};
    use super::calculate_fee;
    #[test]
    #[prusti_contracts::trusted]
    fn mul_ceil() {
        assert_eq!(super::mul_ceil(300_000, 0.001), 301.into());
        assert_eq!(super::mul_ceil(300_004, 0.001), 301.into());
        assert_eq!(super::mul_ceil(300_040, 0.001), 301.into());
        assert_eq!(super::mul_ceil(300_400, 0.001), 301.into());
        assert_eq!(super::mul_ceil(304_000, 0.001), 305.into());
        assert_eq!(super::mul_ceil(340_000, 0.001), 341.into());
        assert_eq!(super::mul_ceil(340_001, 0.001), 341.into());
    }
    /// Before https://github.com/informalsystems/ibc-rs/pull/1568,
    /// this test would have panic'ed with:
    ///
    /// thread 'chain::cosmos::tests::fee_overflow' panicked at 'attempt to multiply with overflow'
    #[test]
    #[prusti_contracts::trusted]
    fn fee_overflow() {
        let gas_amount = 90000000000000_u64;
        let gas_price = GasPrice {
            price: 1000000000000.0,
            denom: "uatom".to_string(),
        };
        let fee = calculate_fee(gas_amount, &gas_price);
        assert_eq!(& fee.amount, "90000000000000000000000000");
    }
    #[test]
    #[prusti_contracts::trusted]
    fn sort_clients_id_suffix() {
        let mut clients: Vec<IdentifiedAnyClientState> = vec![
            IdentifiedAnyClientState::new(ClientId::new(ClientType::Tendermint, 4)
            .unwrap(),
            AnyClientState::Mock(MockClientState::new(MockHeader::new(Height::new(0, 1)
            .unwrap(),))),),
            IdentifiedAnyClientState::new(ClientId::new(ClientType::Tendermint, 1)
            .unwrap(),
            AnyClientState::Mock(MockClientState::new(MockHeader::new(Height::new(0, 1)
            .unwrap(),))),),
            IdentifiedAnyClientState::new(ClientId::new(ClientType::Tendermint, 7)
            .unwrap(),
            AnyClientState::Mock(MockClientState::new(MockHeader::new(Height::new(0, 1)
            .unwrap(),))),),
        ];
        clients.sort_by_cached_key(|c| client_id_suffix(&c.client_id).unwrap_or(0));
        assert_eq!(client_id_suffix(& clients.first().unwrap().client_id).unwrap(), 1);
        assert_eq!(client_id_suffix(& clients[1].client_id).unwrap(), 4);
        assert_eq!(client_id_suffix(& clients.last().unwrap().client_id).unwrap(), 7);
    }
}

