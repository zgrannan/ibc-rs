use serde::{Deserialize, Serialize};
use tracing::trace;

use ibc::{
    ics02_client::client_state::{ClientState, IdentifiedAnyClientState},
    ics03_connection::connection::IdentifiedConnectionEnd,
    ics04_channel::channel::IdentifiedChannelEnd,
    ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId},
    Height,
};

use crate::supervisor::Error;

use super::handle::ChainHandle;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelConnectionClient {
    pub channel: IdentifiedChannelEnd,
    pub connection: IdentifiedConnectionEnd,
    pub client: IdentifiedAnyClientState,
}

impl ChannelConnectionClient {
    pub fn new(
        channel: IdentifiedChannelEnd,
        connection: IdentifiedConnectionEnd,
        client: IdentifiedAnyClientState,
    ) -> Self {
        Self {
            channel,
            connection,
            client,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConnectionClient {
    pub connection: IdentifiedConnectionEnd,
    pub client: IdentifiedAnyClientState,
}

impl ConnectionClient {
    pub fn new(connection: IdentifiedConnectionEnd, client: IdentifiedAnyClientState) -> Self {
        Self { connection, client }
    }
}

pub fn channel_connection_client(
    chain: &dyn ChainHandle,
    port_id: &PortId,
    channel_id: &ChannelId,
) -> Result<ChannelConnectionClient, Error> {
    trace!(
        chain_id = %chain.id(),
        port_id = %port_id,
        channel_id = %channel_id,
        "getting counterparty chain"
    );

    let channel_end = chain
        .query_channel(port_id, channel_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    if channel_end.state_matches(&ibc::ics04_channel::channel::State::Uninitialized) {
        return Err(Error::ChannelUninitialized(channel_id.clone(), chain.id()));
    }

    let connection_id = channel_end
        .connection_hops()
        .first()
        .ok_or_else(|| Error::MissingConnectionHops(channel_id.clone(), chain.id()))?;

    let connection_end = chain
        .query_connection(&connection_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;


    // if connection_end.state_matches(&ibc::ics03_connection::connection::State::Uninitialized) {
    //     return Err(Error::ConnectionNotOpen(
    //                 connection_id.clone(),
    //                 channel_id.clone(),
    //                 chain.id()));
    // }   
    if !connection_end.is_open() {
        return Err(Error::ConnectionNotOpen(
            connection_id.clone(),
            channel_id.clone(),
            chain.id(),
        ));
    }

    let client_id = connection_end.client_id();
    let client_state = chain
        .query_client_state(&client_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    trace!(
        chain_id=%chain.id(), port_id=%port_id, channel_id=%channel_id,
        "counterparty chain: {}", client_state.chain_id()
    );

    let client = IdentifiedAnyClientState::new(client_id.clone(), client_state);
    let connection = IdentifiedConnectionEnd::new(connection_id.clone(), connection_end);
    let channel = IdentifiedChannelEnd::new(port_id.clone(), channel_id.clone(), channel_end);

    Ok(ChannelConnectionClient::new(channel, connection, client))
}

pub fn get_counterparty_chain(
    src_chain: &dyn ChainHandle,
    src_channel_id: &ChannelId,
    src_port_id: &PortId,
) -> Result<ChainId, Error> {
    channel_connection_client(src_chain, src_port_id, src_channel_id)
        .map(|c| c.client.client_state.chain_id())
}

pub fn get_counterparty_chain_for_channel(
    chain: &dyn ChainHandle,
    channel: IdentifiedChannelEnd,
) -> Result<ChainId, Error> {
    let connection_id = channel
        .channel_end
        .connection_hops()
        .first()
        .ok_or_else(|| Error::MissingConnectionHops(channel.channel_id.clone(), chain.id()))?;

    let connection_end = chain
        .query_connection(&connection_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;
    let client_id = connection_end.client_id();
    let client_state = chain
        .query_client_state(&client_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    Ok(client_state.chain_id())
}

pub fn get_counterparty_chain_for_connection(
    client_id: ClientId,
    src_chain: &dyn ChainHandle,
) -> Result<ChainId, Error> {
    let client_state = src_chain
        .query_client_state(&client_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    trace!(
        chain_id=%src_chain.id(),
        "counterparty chain: {}", client_state.chain_id()
    );

    //Ok(IdentifiedAnyClientState::new(client_id.clone(), client_state))
    Ok(client_state.chain_id())
}

pub fn connection_client(
    chain: &dyn ChainHandle,
    connection_id: &ConnectionId,
) -> Result<ConnectionClient, Error> {
    trace!(
        chain_id = %chain.id(),
        connection_id = %connection_id,
        "getting counterparty chain"
    );

    let connection_end = chain
        .query_connection(&connection_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    // if !connection_end.is_open() {
    //     return Err(Error::ConnectionNotOpen(
    //         connection_id.clone(),
    //         channel_id.clone(),
    //         chain.id(),
    //     ));
    // }

    let client_id = connection_end.client_id();
    let client_state = chain
        .query_client_state(&client_id, Height::zero())
        .map_err(|e| Error::QueryFailed(format!("{}", e)))?;

    trace!(
        chain_id=%chain.id(), connection_id=%connection_id,
        "counterparty chain: {}", client_state.chain_id()
    );

    let client = IdentifiedAnyClientState::new(client_id.clone(), client_state);
    let connection = IdentifiedConnectionEnd::new(connection_id.clone(), connection_end);

    Ok(ConnectionClient::new(connection, client))
}
