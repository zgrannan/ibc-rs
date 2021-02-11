mod executor;
use executor::{step, IBCTestExecutor};

const CHAIN_IDS: &[&str] = &["chainA", "chainB"];
const MAX_CHAIN_HEIGHT: u64 = 7;
const MAX_CLIENTS_PER_CHAIN: u64 = 1;
const MAX_CONNECTIONS_PER_CHAIN: u64 = 1;

struct IBC;

impl IBC {
    fn heights() -> impl Iterator<Item = u64> {
        1..=MAX_CHAIN_HEIGHT
    }

    fn client_ids() -> impl Iterator<Item = u64> {
        0..MAX_CLIENTS_PER_CHAIN
    }

    fn connection_ids() -> impl Iterator<Item = u64> {
        0..MAX_CONNECTIONS_PER_CHAIN
    }

    fn create_client_actions(chain_id: &str) -> impl Iterator<Item = step::Action> + '_ {
        Self::heights().map(move |height| step::Action::ICS02CreateClient {
            chain_id: chain_id.to_string(),
            client_state: height,
            consensus_state: height,
        })
    }

    fn update_client_actions(chain_id: &str) -> impl Iterator<Item = step::Action> + '_ {
        Self::client_ids().flat_map(move |client_id| {
            Self::heights().map(move |height| step::Action::ICS02UpdateClient {
                chain_id: chain_id.to_string(),
                client_id,
                header: height,
            })
        })
    }

    fn all_actions() -> impl Iterator<Item = step::Action> {
        CHAIN_IDS.into_iter().flat_map(|chain_id| {
            Self::create_client_actions(chain_id).chain(Self::update_client_actions(chain_id))
        })
    }
}

impl stateright::Model for IBC {
    type State = IBCTestExecutor;
    type Action = step::Action;

    fn init_states(&self) -> Vec<Self::State> {
        let mut state = executor::IBCTestExecutor::new();
        // initialize all chains with height 0
        CHAIN_IDS.into_iter().for_each(|chain_id| {
            let initial_height = 0;
            state.init_chain_context(chain_id.to_string(), initial_height);
        });
        vec![state]
    }

    fn actions(&self, _state: &Self::State, actions: &mut Vec<Self::Action>) {
        // in every state, it's always possible to perform all actions
        actions.extend(Self::all_actions())
    }

    fn next_state(
        &self,
        previous_state: &Self::State,
        action: Self::Action,
    ) -> Option<Self::State> {
        let mut next_state = previous_state.clone();
        // simply apply the action and ignore its result
        let _result = next_state.apply(action);
        Some(next_state)
    }

    fn properties(&self) -> Vec<stateright::Property<Self>> {
        vec![stateright::Property::sometimes(
            "reach max height",
            |_, state: &Self::State| {
                CHAIN_IDS.into_iter().all(|chain_id| {
                    use ibc::ics03_connection::context::ConnectionReader;
                    state
                        .chain_context(chain_id.to_string())
                        .host_current_height()
                        .revision_height
                        == MAX_CHAIN_HEIGHT
                });
                true
            },
        )]
    }
}

#[test]
fn stateright() {
    // use stateright::Model;
    // IBC.checker().spawn_bfs().join().assert_properties()
}
