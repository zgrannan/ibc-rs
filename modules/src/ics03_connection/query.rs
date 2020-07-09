use tendermint_rpc::endpoint::abci_query::AbciQuery;

use tendermint::abci;

use crate::ics23_commitment::{CommitmentPath, CommitmentProof, CommitmentPrefix};
use crate::ics24_host::identifier::{ConnectionId, ClientId};

use crate::error;
use crate::ics03_connection::connection::{ConnectionEnd, Counterparty};
use crate::path::{ConnectionPath, Path};
use crate::query::{IbcQuery, IbcResponse};
use crate::Height;

// Protobuf
use crate::ics03_connection::proto_ibc_connection;
use prost::Message;
use bytes::Bytes;
use std::str::FromStr;

pub struct QueryConnection {
    pub chain_height: Height,
    pub connection_id: ConnectionId,
    pub connection_path: ConnectionPath,
    pub prove: bool,
}

impl QueryConnection {
    pub fn new(chain_height: Height, connection_id: ConnectionId, prove: bool) -> Self {
        Self {
            chain_height,
            connection_id: connection_id.clone(),
            connection_path: ConnectionPath::new(connection_id),
            prove,
        }
    }
}

impl IbcQuery for QueryConnection {
    type Response = ConnectionResponse;

    fn path(&self) -> abci::Path {
        "/store/ibc/key".parse().unwrap()
    }

    fn height(&self) -> Height {
        self.chain_height
    }

    fn prove(&self) -> bool {
        self.prove
    }

    fn data(&self) -> Vec<u8> {
        self.connection_path.to_key().into()
    }
}

pub struct ConnectionResponse {
    pub connection: IdentifiedConnectionEnd,
    pub proof: Option<CommitmentProof>,
    pub proof_path: CommitmentPath,
    pub proof_height: Height,
}

impl ConnectionResponse {
    pub fn new(
        connection_id: ConnectionId,
        connection: ConnectionEnd,
        abci_proof: Option<CommitmentProof>,
        proof_height: Height,
    ) -> Self {
        let proof_path = CommitmentPath::from_path(ConnectionPath::new(connection_id.clone()));
        let identified_connection_end = IdentifiedConnectionEnd::new(connection, connection_id);
        ConnectionResponse {
            connection: identified_connection_end,
            proof: abci_proof,
            proof_path,
            proof_height,
        }
    }
}

impl IbcResponse<QueryConnection> for ConnectionResponse {
    fn from_abci_response(
        query: QueryConnection,
        response: AbciQuery,
    ) -> Result<Self, error::Error> {

        let connection = proto_unmarshal(response.value);
        match connection {
            Ok(conn) => {
                Ok(ConnectionResponse::new(
                    query.connection_id,
                    conn,
                    response.proof,
                    response.height.into(),
                ))
            },
            Err(e) => {
                println!("Error proto un-marshall: {:?}", e);
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct IdentifiedConnectionEnd {
    connection_end: ConnectionEnd,
    connection_id: ConnectionId,
}

impl IdentifiedConnectionEnd {
    pub fn new(connection_end: ConnectionEnd, connection_id: ConnectionId) -> Self {
        IdentifiedConnectionEnd {
            connection_end,
            connection_id,
        }
    }
}

fn proto_unmarshal(bytes: Vec<u8>) -> Result<ConnectionEnd, error::Error> {
    let buf = Bytes::from(bytes);
    let decoded = proto_ibc_connection::ConnectionEnd::decode(buf);
    match decoded {
        Ok(conn) => {
            let client_id = ClientId::from_str(&conn.client_id).unwrap();
            let prefix = CommitmentPrefix{};
            let counterparty = Counterparty::new(client_id.to_string(), conn.id, prefix).unwrap();
            let connection_end = ConnectionEnd::new(client_id, counterparty, conn.versions).unwrap();
            Ok(connection_end)
        },
        Err(e) => {
            println!("Error proto un-marshall: {:?}", e);
            todo!()
        }
    }
}

fn amino_unmarshal_binary_length_prefixed<T>(_bytes: &[u8]) -> Result<T, error::Error> {
    todo!()
}
