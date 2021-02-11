mod executor;
use executor::{step, IBCTestExecutor};

const CHAIN_IDS: &[&str] = &["chainA", "chainB"];
const MAX_CHAIN_HEIGHT: u64 = 3;
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

    fn connection_open_init_actions(chain_id: &str) -> impl Iterator<Item = step::Action> + '_ {
        Self::client_ids().flat_map(move |client_id| {
            Self::client_ids().map(move |counterparty_client_id| {
                step::Action::ICS03ConnectionOpenInit {
                    chain_id: chain_id.to_string(),
                    client_id,
                    counterparty_client_id,
                }
            })
        })
    }

    fn connection_open_try_actions(chain_id: &str) -> impl Iterator<Item = step::Action> + '_ {
        Self::connection_ids()
            .map(Some)
            .chain(None)
            .flat_map(move |previous_connection_id| {
                Self::client_ids().flat_map(move |client_id| {
                    Self::heights().flat_map(move |height| {
                        Self::client_ids().flat_map(move |counterparty_client_id| {
                            Self::connection_ids().map(move |counterparty_connection_id| {
                                step::Action::ICS03ConnectionOpenTry {
                                    chain_id: chain_id.to_string(),
                                    previous_connection_id,
                                    client_id,
                                    client_state: height,
                                    counterparty_client_id,
                                    counterparty_connection_id,
                                }
                            })
                        })
                    })
                })
            })
    }

    fn all_actions() -> impl Iterator<Item = step::Action> {
        CHAIN_IDS.into_iter().flat_map(|chain_id| {
            Self::create_client_actions(chain_id)
                .chain(Self::update_client_actions(chain_id))
                .chain(Self::connection_open_init_actions(chain_id))
                .chain(Self::connection_open_try_actions(chain_id))
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
        vec![
            eventually_all_chains_reach_max_height(),
            eventually_some_connection_reaches_is_in_try_open_state(),
        ]
    }
}

fn eventually_all_chains_reach_max_height() -> stateright::Property<IBC> {
    use ibc::ics03_connection::context::ConnectionReader;
    stateright::Property::sometimes(
        "eventually all chains reach max height",
        |_, state: &IBCTestExecutor| {
            // \A chainId \in ChainIds
            CHAIN_IDS.into_iter().all(|chain_id| {
                let ctx = state.chain_context(chain_id.to_string());
                // chains[chainsId].height == MAX_CHAIN_HEIGHT
                ctx.host_current_height().revision_height == MAX_CHAIN_HEIGHT
            })
        },
    )
}

fn eventually_some_connection_reaches_is_in_try_open_state() -> stateright::Property<IBC> {
    use ibc::ics03_connection::context::ConnectionReader;
    stateright::Property::sometimes(
        "eventually some connections is in try open state",
        |_, state: &IBCTestExecutor| {
            // \E chainId \in ChainIds
            CHAIN_IDS.into_iter().any(|chain_id| {
                let ctx = state.chain_context(chain_id.to_string());
                // \E connectionId \in ConnectionIds
                IBC::connection_ids().any(|connection_id| {
                    let connection =
                        ctx.connection_end(&IBCTestExecutor::connection_id(connection_id));
                    // connections[connectionId].state == TRY_OPEN
                    connection.iter().any(|connection| {
                        connection.state() == &ibc::ics03_connection::connection::State::TryOpen
                    })
                })
            })
        },
    )
}

#[test]
fn stateright() {
    use stateright::Checker;
    use stateright::Model;
    // requires: IBCTestExecutor implements Hash
    IBC.checker().spawn_bfs().join().assert_properties()
}
