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
use crate::telemetry;

// Maximum number of retries for send_tx in the case of
// an account sequence mismatch at broadcast step.
const MAX_ACCOUNT_SEQUENCE_RETRY: u64 = 1;

// Backoff multiplier to apply while retrying in the case
// of account sequence mismatch.
const BACKOFF_MULTIPLIER_ACCOUNT_SEQUENCE_RETRY: u64 = 300;

// The error "incorrect account sequence" is defined as the unique error code 32 in cosmos-sdk:
// https://github.com/cosmos/cosmos-sdk/blob/v0.44.0/types/errors/errors.go#L115-L117
const INCORRECT_ACCOUNT_SEQUENCE_ERR: u32 = 32;

/// Determine whether the given error yielded by `tx_simulate`
/// indicates hat the current sequence number cached in Hermes
/// may be out-of-sync with the full node's version of the s.n.
fn mismatching_account_sequence_number(e: &Error) -> bool {
    use crate::error::ErrorDetail::*;

    match e.detail() {
        GrpcStatus(detail) => detail.is_account_sequence_mismatch(),
        _ => false,
    }
}
