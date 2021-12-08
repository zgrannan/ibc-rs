#[cfg(feature="prusti")]
use prusti_contracts::*;
use std::collections::HashMap;
use std::convert::Infallible;
use std::convert::{TryFrom, TryInto};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tendermint_proto::Protobuf;

use ibc_proto::ibc::mock::ClientState as RawMockClientState;
use ibc_proto::ibc::mock::ConsensusState as RawMockConsensusState;

use crate::ics02_client::client_consensus::{AnyConsensusState, ConsensusState};
use crate::ics02_client::client_state::{AnyClientState, ClientState};
use crate::ics02_client::client_type::ClientType;
use crate::ics02_client::error::Error;
use crate::ics23_commitment::commitment::CommitmentRoot;
use crate::ics24_host::identifier::ChainId;
use crate::mock::header::MockHeader;
use crate::timestamp::Timestamp;
use crate::Height;

/// A mock of an IBC client record as it is stored in a mock context.
/// For testing ICS02 handlers mostly, cf. `MockClientContext`.
#[derive(Clone)]
#[cfg_attr(feature="prusti", derive(PartialEq, Eq))]
pub struct MockClientRecord {
    /// The type of this client.
    pub client_type: ClientType,

    /// The client state (representing only the latest height at the moment).
    pub client_state: Option<AnyClientState>,

    /// Mapping of heights to consensus states for this client.
    pub consensus_states: HashMap<Height, AnyConsensusState>,
}

#[extern_spec]
impl <K, V, S> std::collections::HashMap<K, V, S> {
    // #[pure]
    // fn get<'a>(&self, key: &K) -> Option<&'a V>
    //   where K: Eq,
    //         K: std::hash::Hash,
    //         S: std::hash::BuildHasher;

    #[pure]
    fn is_empty(&self) -> bool;

    #[pure]
    fn contains_key(&self, key: &K) -> bool
      where K: Eq,
            K: std::hash::Hash,
            S: std::hash::BuildHasher;
}

#[pure]
#[trusted]
fn get_highest_consensus_state(client: &MockClientRecord) -> Option<Height> {
    client.consensus_states.keys().max().cloned()
}


#[pure]
pub fn client_invariant(client: &MockClientRecord) -> bool {
    match &client.client_state {
        Some(cs) =>
            match get_highest_consensus_state(client) {
                Some(max_height) => cs.latest_height() == max_height,
                None => false
            },
        None => client.consensus_states.is_empty()
    }
}

/// A mock of a client state. For an example of a real structure that this mocks, you can see
/// `ClientState` of ics07_tendermint/client_state.rs.
// TODO: `MockClientState` should evolve, at the very least needs a `is_frozen` boolean field.
#[derive(Copy, Clone)]
#[cfg_attr(not(feature="prusti"), derive(Debug), derive(Deserialize), derive(Serialize), derive(PartialEq), derive(Eq))]
pub struct MockClientState(pub MockHeader);

impl Protobuf<RawMockClientState> for MockClientState {}

impl MockClientState {
    #[cfg_attr(feature="prusti", pure)]
    pub fn latest_height(&self) -> Height {
        (self.0).height
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn refresh_time(&self) -> Option<Duration> {
        None
    }
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn expired(&self, _elapsed: Duration) -> bool {
        false
    }
}

impl From<MockClientState> for AnyClientState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(mcs: MockClientState) -> Self {
        Self::Mock(mcs)
    }
}

impl TryFrom<RawMockClientState> for MockClientState {
    type Error = Error;

    #[cfg_attr(feature="prusti", trusted)]
    fn try_from(raw: RawMockClientState) -> Result<Self, Self::Error> {
        Ok(MockClientState(raw.header.unwrap().try_into()?))
    }
}

impl From<MockClientState> for RawMockClientState {
    #[cfg_attr(feature="prusti", trusted)]
    fn from(value: MockClientState) -> Self {
        RawMockClientState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.0.height().into()),
                timestamp: (value.0).timestamp.as_nanoseconds(),
            }),
        }
    }
}

impl ClientState for MockClientState {
    #[cfg_attr(feature="prusti", trusted)]
    fn chain_id(&self) -> ChainId {
        todo!()
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn client_type(&self) -> ClientType {
        ClientType::Mock
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn latest_height(&self) -> Height {
        self.0.height()
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn is_frozen(&self) -> bool {
        // TODO
        false
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn wrap_any(self) -> AnyClientState {
        AnyClientState::Mock(self)
    }
}

impl From<MockConsensusState> for MockClientState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(cs: MockConsensusState) -> Self {
        Self(cs.header)
    }
}

#[cfg_attr(not(feature="prusti"), derive(Debug))]
#[cfg_attr(feature="prusti_fast", derive(PrustiClone))]
#[cfg_attr(not(feature="prusti_fast"), derive(Clone))]
pub struct MockConsensusState {
    pub header: MockHeader,
    pub root: CommitmentRoot,
}

impl MockConsensusState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn new(header: MockHeader) -> Self {
        MockConsensusState {
            header,
            root: CommitmentRoot::from(vec![0]),
        }
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    pub fn timestamp(&self) -> Timestamp {
        self.header.timestamp
    }
}

impl Protobuf<RawMockConsensusState> for MockConsensusState {}

impl TryFrom<RawMockConsensusState> for MockConsensusState {
    type Error = Error;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn try_from(raw: RawMockConsensusState) -> Result<Self, Self::Error> {
        let raw_header = raw.header.ok_or_else(Error::missing_raw_consensus_state)?;

        Ok(Self {
            header: MockHeader::try_from(raw_header)?,
            root: CommitmentRoot::from(vec![0]),
        })
    }
}

impl From<MockConsensusState> for RawMockConsensusState {
    #[cfg_attr(feature="prusti", trusted)]
    fn from(value: MockConsensusState) -> Self {
        RawMockConsensusState {
            header: Some(ibc_proto::ibc::mock::Header {
                height: Some(value.header.height().into()),
                timestamp: value.header.timestamp.as_nanoseconds(),
            }),
        }
    }
}

impl From<MockConsensusState> for AnyConsensusState {
#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn from(mcs: MockConsensusState) -> Self {
        Self::Mock(mcs)
    }
}

impl ConsensusState for MockConsensusState {
    type Error = Infallible;

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn client_type(&self) -> ClientType {
        ClientType::Mock
    }

    // fn root(&self) -> &CommitmentRoot {
    //     &self.root
    // }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn validate_basic(&self) -> Result<(), Infallible> {
        Ok(())
    }

#[cfg_attr(feature="prusti_fast", trusted_skip)]
    fn wrap_any(self) -> AnyConsensusState {
        AnyConsensusState::Mock(self)
    }
}
