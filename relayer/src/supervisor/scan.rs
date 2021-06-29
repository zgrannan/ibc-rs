use std::{collections::BTreeMap, fmt};

use tracing::{debug, error};

use ibc::{
    ics02_client::client_state::{ClientState, IdentifiedAnyClientState},
    ics03_connection::connection::IdentifiedConnectionEnd,
    ics04_channel::channel::IdentifiedChannelEnd,
    ics24_host::identifier::{ChainId, ClientId, ConnectionId},
    Height,
};

use ibc_proto::cosmos::base::query::pagination;

use crate::{chain::handle::ChainHandle, config::Config, registry::Registry};

use super::RwArc;

#[derive(Clone, Debug)]
pub struct ClientScan {
    client: IdentifiedAnyClientState,
    connections: BTreeMap<ConnectionId, ConnectionScan>,
}

impl ClientScan {
    pub fn new(client: IdentifiedAnyClientState) -> Self {
        Self {
            client,
            connections: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionScan {
    connection: IdentifiedConnectionEnd,
    channels: Vec<IdentifiedChannelEnd>,
}

impl ConnectionScan {
    pub fn new(connection: IdentifiedConnectionEnd) -> Self {
        Self {
            connection,
            channels: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ChainScan {
    chain: Box<dyn ChainHandle>,
    clients: BTreeMap<ClientId, ClientScan>,
}

impl ChainScan {
    pub fn new(chain: Box<dyn ChainHandle>) -> Self {
        Self {
            chain,
            clients: BTreeMap::new(),
        }
    }

    fn connections(&self) -> Vec<(IdentifiedAnyClientState, IdentifiedConnectionEnd)> {
        let mut conns = Vec::new();

        for client in self.clients.values() {
            for connection in client.connections.values() {
                conns.push((client.client.clone(), connection.connection.clone()));
            }
        }

        conns
    }
}

#[derive(Clone, Debug, Default)]
pub struct Scan {
    chains: BTreeMap<ChainId, ChainScan>,
}

impl fmt::Display for Scan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (chain_id, chain) in &self.chains {
            writeln!(f, "# {}", chain_id)?;
            for (client_id, client) in &chain.clients {
                writeln!(f, "  * {}", client_id)?;
                for (connection_id, c) in &client.connections {
                    writeln!(
                        f,
                        "    - {} ({:?})",
                        connection_id, c.connection.connection_end.state
                    )?;

                    for channel in &c.channels {
                        writeln!(
                            f,
                            "      > {} ({:?})",
                            channel.channel_id, channel.channel_end.state
                        )?;
                    }
                }
            }
        }

        Ok(())
    }
}

pub struct ChainScanner<'a> {
    config: &'a RwArc<Config>,
    registry: &'a mut Registry,

    scan: Scan,
}

impl<'a> ChainScanner<'a> {
    pub fn new(config: &'a RwArc<Config>, registry: &'a mut Registry) -> Self {
        Self {
            config,
            registry,
            scan: Scan::default(),
        }
    }

    pub fn get(self) -> Scan {
        self.scan
    }

    pub fn scan(&mut self, chain_id: &ChainId) {
        if let Some(chain_scan) = self.scan_chain(chain_id.clone()) {
            let connections = chain_scan.connections();
            self.scan.chains.insert(chain_id.clone(), chain_scan);

            let counterparty_client_scans = self.scan_counterparties(connections);
            for (chain, client_scan) in counterparty_client_scans {
                let client_id = client_scan.client.client_id.clone();

                self.scan
                    .chains
                    .entry(chain.id())
                    .or_insert_with(|| ChainScan::new(chain))
                    .clients
                    .entry(client_id)
                    .or_insert_with(|| ClientScan::new(client_scan.client.clone()))
                    .connections
                    .extend(client_scan.connections);
            }
        }
    }

    fn scan_counterparties(
        &mut self,
        connections: Vec<(IdentifiedAnyClientState, IdentifiedConnectionEnd)>,
    ) -> Vec<(Box<dyn ChainHandle>, ClientScan)> {
        let mut new_clients = Vec::new();

        for (client, connection) in connections {
            let counterparty_chain = self.registry.get_or_spawn(&client.client_state.chain_id());

            let counterparty_chain = match counterparty_chain {
                Ok(chain) => chain,
                Err(_e) => continue,
            };

            let counterparty_client_id = connection.connection_end.counterparty().client_id();

            let counterparty_client_state =
                counterparty_chain.query_client_state(counterparty_client_id, Height::zero());

            let counterparty_client_state = match counterparty_client_state {
                Ok(client_state) => client_state,
                Err(_e) => continue,
            };

            let counterparty_client = IdentifiedAnyClientState {
                client_id: counterparty_client_id.clone(),
                client_state: counterparty_client_state,
            };

            let counterparty_client_scan =
                self.scan_client(counterparty_chain.as_ref(), counterparty_client);

            if let Some(counterparty_client_scan) = counterparty_client_scan {
                new_clients.push((counterparty_chain, counterparty_client_scan));
            }
        }

        new_clients
    }

    fn scan_chain(&mut self, chain_id: ChainId) -> Option<ChainScan> {
        let chain = match self.registry.get_or_spawn(&chain_id) {
            Ok(handle) => handle,
            Err(e) => {
                error!(
                    "skipping chain scan for chain {}, reason: failed to spawn chain runtime with error: {}",
                    chain_id, e
                );

                return None;
            }
        };

        let clients = query_clients(chain.as_ref())
            .into_iter()
            .flatten()
            .flat_map(|client| self.scan_client(chain.as_ref(), client))
            .map(|c| (c.client.client_id.clone(), c))
            .collect();

        Some(ChainScan { chain, clients })
    }

    fn scan_client(
        &self,
        chain: &dyn ChainHandle,
        client: IdentifiedAnyClientState,
    ) -> Option<ClientScan> {
        debug!(
            "scanning client {} on chain {}",
            client.client_id,
            chain.id()
        );

        let counterparty_chain_id = client.client_state.chain_id();

        let has_counterparty = self.has_chain(&counterparty_chain_id);
        if !has_counterparty {
            debug!(
                "skipping connections scan for client {} on chain {} has its counterparty ({}) is not present in config",
                client.client_id, chain.id(), counterparty_chain_id
            );

            return None;
        }

        let connections = query_client_connections(chain, &client)
            .into_iter()
            .flatten()
            .flat_map(|id| self.scan_connection(chain, id))
            .map(|c| (c.connection.connection_id.clone(), c))
            .collect();

        Some(ClientScan {
            client,
            connections,
        })
    }

    fn scan_connection(
        &self,
        chain: &dyn ChainHandle,
        connection_id: ConnectionId,
    ) -> Option<ConnectionScan> {
        let connection_end = match chain.query_connection(&connection_id, Height::zero()) {
            Ok(connection_end) => connection_end,
            Err(e) => {
                error!(
                    "skipping connection scan for chain {} and connection {}, reason: failed to query connection end: {}",
                    chain.id(), connection_id, e
                );

                return None;
            }
        };

        if !connection_end.is_open() {
            debug!(
                "connection {} not open, skip scan for channels over this connection",
                connection_id
            );

            return None;
        }

        let connection = IdentifiedConnectionEnd {
            connection_id,
            connection_end,
        };

        let channels = self.scan_connection_channels(chain, &connection);

        Some(ConnectionScan {
            connection,
            channels,
        })
    }

    fn scan_connection_channels(
        &self,
        chain: &dyn ChainHandle,
        connection: &IdentifiedConnectionEnd,
    ) -> Vec<IdentifiedChannelEnd> {
        let channels = query_connection_channels(chain, connection);
        channels.unwrap_or_else(Vec::new)
    }

    fn has_chain(&self, chain_id: &ChainId) -> bool {
        self.config
            .read()
            .expect("poisoned lock")
            .has_chain(chain_id)
    }
}

fn query_client_connections(
    chain: &dyn ChainHandle,
    client: &IdentifiedAnyClientState,
) -> Option<Vec<ConnectionId>> {
    use ibc_proto::ibc::core::connection::v1::QueryClientConnectionsRequest;

    let conns_req = QueryClientConnectionsRequest {
        client_id: client.client_id.to_string(),
    };

    match chain.query_client_connections(conns_req) {
        Ok(connections) => Some(connections),
        Err(e) => {
            error!(
                "skipping client scan for chain {}, reason: failed to query client connections for client {}: {}",
                chain.id(), client.client_id, e
            );

            None
        }
    }
}

fn query_clients(chain: &dyn ChainHandle) -> Option<Vec<IdentifiedAnyClientState>> {
    use ibc_proto::ibc::core::client::v1::QueryClientStatesRequest;

    let clients_req = QueryClientStatesRequest {
        pagination: pagination::all(),
    };

    match chain.query_clients(clients_req) {
        Ok(clients) => Some(clients),
        Err(e) => {
            error!(
                "skipping clients scan for chain {}, reason: failed to query clients with error: {}",
                chain.id(),
                e
            );

            None
        }
    }
}

fn query_connection_channels(
    chain: &dyn ChainHandle,
    connection: &IdentifiedConnectionEnd,
) -> Option<Vec<IdentifiedChannelEnd>> {
    use ibc_proto::ibc::core::channel::v1::QueryConnectionChannelsRequest;

    let chans_req = QueryConnectionChannelsRequest {
        connection: connection.connection_id.to_string(),
        pagination: pagination::all(),
    };

    match chain.query_connection_channels(chans_req) {
        Ok(channels) => Some(channels),
        Err(e) => {
            error!("skipping channels scan for chain {} and connection {}, reason: failed to query its channels: {}",
                    chain.id(), connection.connection_id, e
            );

            None
        }
    }
}
