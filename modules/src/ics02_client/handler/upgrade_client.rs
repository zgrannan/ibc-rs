//! Protocol logic specific to processing ICS2 messages of type `MsgUpgradeAnyClient`.
//!
use crate::events::IbcEvent;
use crate::handler::{HandlerOutput, HandlerResult};
use crate::ics02_client::client_consensus::AnyConsensusState;
use crate::ics02_client::client_def::{AnyClient, ClientDef};
use crate::ics02_client::client_state::AnyClientState;
use crate::ics02_client::client_state::ClientState;
use crate::ics02_client::context::ClientReader;
use crate::ics02_client::error::{Error, Kind};
use crate::ics02_client::events::Attributes;
use crate::ics02_client::handler::ClientResult;
use crate::ics02_client::msgs::upgrade_client::MsgUpgradeAnyClient;
use crate::ics24_host::identifier::ClientId;

/// The result following the successful processing of a `MsgUpgradeAnyClient` message.
/// This data type should be used with a qualified name `upgrade_client::Result` to avoid ambiguity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Result {
    pub client_id: ClientId,
    pub client_state: AnyClientState,
    pub consensus_state: AnyConsensusState,
}

pub fn process(
    ctx: &dyn ClientReader,
    msg: MsgUpgradeAnyClient,
) -> HandlerResult<ClientResult, Error> {
    let mut output = HandlerOutput::builder();
    let MsgUpgradeAnyClient { client_id, .. } = msg;

    // Read client state from the host chain store.
    let client_state = ctx
        .client_state(&client_id)
        .ok_or_else(|| Kind::ClientNotFound(client_id.clone()))?;

    if client_state.is_frozen() {
        return Err(Kind::ClientFrozen(client_id).into());
    }

    let upgrade_client_state = msg.client_state.clone();

    if client_state.latest_height() >= upgrade_client_state.latest_height() {
        return Err(Kind::LowUgradeHeight(
            client_state.latest_height(),
            upgrade_client_state.latest_height(),
        )
        .into());
    }

    let client_type = ctx
        .client_type(&client_id)
        .ok_or_else(|| Kind::ClientNotFound(client_id.clone()))?;

    let client_def = AnyClient::from_client_type(client_type);

    let (new_client_state, new_consensus_state) = client_def
        .verify_upgrade_and_update_state(
            &upgrade_client_state,
            &msg.consensus_state,
            msg.proof_upgrade_client.clone(),
            msg.proof_upgrade_consensus_state,
        )
        .map_err(|e| Kind::UpgradeVerificationFailure.context(e.to_string()))?;

    // Not implemented yet: https://github.com/informalsystems/ibc-rs/issues/722
    // todo!()

    let result = ClientResult::Upgrade(Result {
        client_id: client_id.clone(),
        client_state: new_client_state,
        consensus_state: new_consensus_state,
    });
    let event_attributes = Attributes {
        client_id,
        ..Default::default()
    };

    output.emit(IbcEvent::UpgradeClient(event_attributes.into()));
    Ok(output.with_result(result))
}
