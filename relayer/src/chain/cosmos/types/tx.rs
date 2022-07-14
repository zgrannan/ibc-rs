use ibc::events::IbcEvent;
use ibc_proto::cosmos::tx::v1beta1::{AuthInfo, TxBody};
use tendermint_rpc::endpoint::broadcast::tx_sync::Response;
pub struct SignedTx {
    pub body: TxBody,
    pub body_bytes: Vec<u8>,
    pub auth_info: AuthInfo,
    pub auth_info_bytes: Vec<u8>,
    pub signatures: Vec<Vec<u8>>,
}
pub enum TxStatus {
    Pending { message_count: usize },
    ReceivedResponse,
}
pub struct TxSyncResult {
    pub response: Response,
    pub events: Vec<IbcEvent>,
    pub status: TxStatus,
}

