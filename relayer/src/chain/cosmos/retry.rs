use core::future::Future;
use core::pin::Pin;
use core::time::Duration;
use ibc_proto::google::protobuf::Any;
use std::thread;
use tendermint::abci::Code;
use tendermint_rpc::endpoint::broadcast::tx_sync::Response;
use tracing::{debug, error, span, warn, Level};
use crate::chain::cosmos::query::account::refresh_account;
use crate::chain::cosmos::tx::estimate_fee_and_send_tx;
use crate::chain::cosmos::types::account::Account;
use crate::chain::cosmos::types::config::TxConfig;
use crate::config::types::Memo;
use crate::error::Error;
use crate::keyring::KeyEntry;
use crate::sdk_error::sdk_error_from_tx_sync_error_code;
use crate::telemetry;
const MAX_ACCOUNT_SEQUENCE_RETRY: u64 = 1;
const BACKOFF_MULTIPLIER_ACCOUNT_SEQUENCE_RETRY: u64 = 300;
const INCORRECT_ACCOUNT_SEQUENCE_ERR: u32 = 32;
/// Try to `send_tx` with retry on account sequence error.
/// An account sequence error can occur if the account sequence that
/// the relayer caches becomes outdated. This may happen if the relayer
/// wallet is used concurrently elsewhere, or when tx fees are mis-configured
/// leading to transactions hanging in the mempool.
///
/// Account sequence mismatch error can occur at two separate steps:
///   1. as Err variant, propagated from the `estimate_gas` step.
///   2. as an Ok variant, with an Code::Err response, propagated from
///     the `broadcast_tx_sync` step.
///
/// We treat both cases by re-fetching the account sequence number
/// from the full node.
/// Upon case #1, we do not retry submitting the same tx (retry happens
/// nonetheless at the worker `step` level). Upon case #2, we retry
/// submitting the same transaction.
#[prusti_contracts::trusted]
pub async fn send_tx_with_account_sequence_retry(
    config: &TxConfig,
    key_entry: &KeyEntry,
    account: &mut Account,
    tx_memo: &Memo,
    messages: Vec<Any>,
    retry_counter: u64,
) -> Result<Response, Error> {
    crate::time!("send_tx_with_account_sequence_retry");
    let _span = span!(
        Level::ERROR, "send_tx_with_account_sequence_retry", id = % config.chain_id
    )
        .entered();
    telemetry!(msg_num, & config.chain_id, messages.len() as u64);
    do_send_tx_with_account_sequence_retry(
            config,
            key_entry,
            account,
            tx_memo,
            messages,
            retry_counter,
        )
        .await
}
#[prusti_contracts::trusted]
fn do_send_tx_with_account_sequence_retry<'a>(
    config: &'a TxConfig,
    key_entry: &'a KeyEntry,
    account: &'a mut Account,
    tx_memo: &'a Memo,
    messages: Vec<Any>,
    retry_counter: u64,
) -> Pin<Box<dyn Future<Output = Result<Response, Error>> + 'a>> {
    Box::pin(async move {
        debug!(
            "sending {} messages using account sequence {}", messages.len(), account
            .sequence,
        );
        let tx_result = estimate_fee_and_send_tx(
                config,
                key_entry,
                account,
                tx_memo,
                messages.clone(),
            )
            .await;
        match tx_result {
            Err(e) if mismatch_account_sequence_number_error_requires_refresh(&e) => {
                warn!(
                    "failed at estimate_gas step mismatching account sequence: dropping the tx & refreshing account sequence number"
                );
                refresh_account(&config.grpc_address, &key_entry.account, account)
                    .await?;
                Err(e)
            }
            Ok(
                response,
            ) if response.code == Code::Err(INCORRECT_ACCOUNT_SEQUENCE_ERR) => {
                if retry_counter < MAX_ACCOUNT_SEQUENCE_RETRY {
                    let retry_counter = retry_counter + 1;
                    warn!(
                        "failed at broadcast step with incorrect account sequence. retrying ({}/{})",
                        retry_counter, MAX_ACCOUNT_SEQUENCE_RETRY
                    );
                    let backoff = retry_counter
                        * BACKOFF_MULTIPLIER_ACCOUNT_SEQUENCE_RETRY;
                    thread::sleep(Duration::from_millis(backoff));
                    refresh_account(&config.grpc_address, &key_entry.account, account)
                        .await?;
                    do_send_tx_with_account_sequence_retry(
                            config,
                            key_entry,
                            account,
                            tx_memo,
                            messages,
                            retry_counter + 1,
                        )
                        .await
                } else {
                    error!(
                        "failed due to account sequence errors. the relayer wallet may be used elsewhere concurrently."
                    );
                    Ok(response)
                }
            }
            Ok(response) => {
                match response.code {
                    Code::Ok => {
                        debug!("broadcast_tx_sync: {:?}", response);
                        account.sequence.increment_mut();
                        Ok(response)
                    }
                    Code::Err(code) => {
                        error!(
                            "broadcast_tx_sync: {:?}: diagnostic: {:?}", response,
                            sdk_error_from_tx_sync_error_code(code)
                        );
                        Ok(response)
                    }
                }
            }
            Err(e) => Err(e),
        }
    })
}
/// Determine whether the given error yielded by `tx_simulate`
/// indicates hat the current sequence number cached in Hermes
/// is smaller than the full node's version of the s.n. and therefore
/// account needs to be refreshed.
#[prusti_contracts::trusted]
fn mismatch_account_sequence_number_error_requires_refresh(e: &Error) -> bool {
    use crate::error::ErrorDetail::*;
    match e.detail() {
        GrpcStatus(detail) => detail.is_account_sequence_mismatch_that_requires_refresh(),
        _ => false,
    }
}

