use ibc::events::IbcEvent;
use ibc_proto::google::protobuf::Any;
use prost::Message;
use tendermint_rpc::endpoint::broadcast::tx_sync::Response;

use crate::chain::cosmos::types::account::Account;
use crate::chain::cosmos::types::config::TxConfig;
use crate::chain::cosmos::types::tx::TxSyncResult;
use crate::chain::cosmos::wait::wait_for_block_commits;
use crate::config::types::{MaxMsgNum, MaxTxSize, Memo};
use crate::error::Error;
use crate::keyring::KeyEntry;

fn batch_messages(
    max_msg_num: MaxMsgNum,
    max_tx_size: MaxTxSize,
    messages: Vec<Any>,
) -> Result<Vec<Vec<Any>>, Error> {
    let max_message_count = max_msg_num.to_usize();
    let max_tx_size = max_tx_size.into();

    let mut batches = vec![];

    let mut current_count = 0;
    let mut current_size = 0;
    let mut current_batch = vec![];

    for message in messages.into_iter() {
        current_count += 1;
        current_size += message.encoded_len();
        current_batch.push(message);

        if current_count >= max_message_count || current_size >= max_tx_size {
            let insert_batch = current_batch.drain(..).collect();
            batches.push(insert_batch);
            current_count = 0;
            current_size = 0;
        }
    }

    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    Ok(batches)
}
