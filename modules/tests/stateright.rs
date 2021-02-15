mod executor;

use executor::{step, IBCTestExecutor};

use ibc::ics02_client::context::ClientReader;
use ibc::ics03_connection::connection::State as ConnectionState;
use ibc::ics03_connection::context::ConnectionReader;
use ibc::mock::context::MockContext;

const CHAIN_IDS: &[&'static str] = &["chainA", "chainB", "chainC"];
const MAX_CHAIN_HEIGHT: u64 = 5;
const MAX_CLIENTS_PER_CHAIN: u64 = 1;
const MAX_CONNECTIONS_PER_CHAIN: u64 = 1;

#[derive(Debug, Clone, Hash)]
struct IBCModelState {
    executor: IBCTestExecutor,
    action: step::Action,
}

struct IBC;

impl IBC {
    // Heights == 1..MaxChainHeight
    fn heights() -> impl Iterator<Item = u64> {
        1..=MAX_CHAIN_HEIGHT
    }

    // ClientIds == 0..(MaxClientsPerChain - 1)
    fn client_ids() -> impl Iterator<Item = u64> {
        0..MAX_CLIENTS_PER_CHAIN
    }

    // ConnectionIds == 0..(MaxConnectionsPerChain- 1)
    fn connection_ids() -> impl Iterator<Item = u64> {
        0..MAX_CONNECTIONS_PER_CHAIN
    }

    fn create_client_actions<'a>(
        chain_id: &'static str,
        ctx: &'a MockContext,
    ) -> impl Iterator<Item = step::Action> + 'a {
        // \E height \in \Heights
        Self::heights().filter_map(move |height| {
            // only create client if the model constant `MAX_CLIENTS_PER_CHAIN`
            // allows it
            let allowed = ctx.client_counter() < MAX_CLIENTS_PER_CHAIN;
            allowed.then(|| step::Action::ICS02CreateClient {
                chain_id: chain_id.to_string(),
                client_state: height,
                consensus_state: height,
            })
        })
    }

    fn update_client_actions(chain_id: &'static str) -> impl Iterator<Item = step::Action> + '_ {
        // \E clientId \in \ClientIds
        Self::client_ids().flat_map(move |client_id| {
            // \E height \in \Heights
            Self::heights().map(move |height| step::Action::ICS02UpdateClient {
                chain_id: chain_id.to_string(),
                client_id,
                header: height,
            })
        })
    }

    fn connection_open_init_actions<'a>(
        chain_id: &'static str,
        ctx: &'a MockContext,
    ) -> impl Iterator<Item = step::Action> + 'a {
        // \E clientId \in \ClientIds
        Self::client_ids().flat_map(move |client_id| {
            // \E counterpartyClientId \in \ClientIds
            Self::client_ids().filter_map(move |counterparty_client_id| {
                // only create connection if the model constant
                // `MaxConnectionsPerChain` allows it
                let allowed = ctx.connection_counter() < MAX_CONNECTIONS_PER_CHAIN;
                allowed.then(|| step::Action::ICS03ConnectionOpenInit {
                    chain_id: chain_id.to_string(),
                    client_id,
                    counterparty_client_id,
                })
            })
        })
    }

    fn connection_open_try_actions<'a>(
        chain_id: &'static str,
        ctx: &'a MockContext,
    ) -> impl Iterator<Item = step::Action> + 'a {
        // \E previousConnectionId \in ConnectionIds \union {ConnectionIdNone}
        Self::connection_ids()
            .map(Some)
            .chain(None)
            .flat_map(move |previous_connection_id| {
                // \E clientId \in ClientIds:
                Self::client_ids().flat_map(move |client_id| {
                    // \E height \in Heights:
                    Self::heights().flat_map(move |height| {
                        // \E counterpartyClientId \in ClientIds:
                        Self::client_ids().flat_map(move |counterparty_client_id| {
                            // \E counterpartyConnectionId \in ConnectionIds:
                            Self::connection_ids().filter_map(move |counterparty_connection_id| {
                                // only perform action if there was a previous
                                // connection or if the model constant
                                // `MAX_CONNECTIONS_PER_CHAIN` allows it
                                let allowed = previous_connection_id.is_some()
                                    && ctx.connection_counter() < MAX_CONNECTIONS_PER_CHAIN;
                                allowed.then(|| step::Action::ICS03ConnectionOpenTry {
                                    chain_id: chain_id.to_string(),
                                    previous_connection_id,
                                    client_id,
                                    client_state: height,
                                    counterparty_client_id,
                                    counterparty_connection_id,
                                })
                            })
                        })
                    })
                })
            })
    }

    fn next_actions(state: &IBCTestExecutor) -> impl Iterator<Item = step::Action> + '_ {
        // \E chainId \in ChainIds:
        CHAIN_IDS
            .into_iter()
            .filter_map(move |chain_id| {
                let ctx = state.chain_context(chain_id.to_string());
                // perform action on chain if the model constant
                // `MAX_CHAIN_HEIGHT` allows it
                let allowed = ctx.host_current_height() < IBCTestExecutor::height(MAX_CHAIN_HEIGHT);
                allowed.then(|| {
                    Self::create_client_actions(chain_id, ctx)
                        .chain(Self::update_client_actions(chain_id))
                        .chain(Self::connection_open_init_actions(chain_id, ctx))
                        .chain(Self::connection_open_try_actions(chain_id, ctx))
                })
            })
            .flatten()
    }
}

impl stateright::Model for IBC {
    type State = IBCModelState;
    type Action = step::Action;

    fn init_states(&self) -> Vec<Self::State> {
        let mut executor = executor::IBCTestExecutor::new();
        // initialize all chains with height 1
        CHAIN_IDS.into_iter().for_each(|chain_id| {
            let initial_height = 1;
            executor.init_chain_context(chain_id.to_string(), initial_height);
        });
        let state = IBCModelState {
            executor,
            action: step::Action::None,
        };
        vec![state]
    }

    fn actions(&self, state: &Self::State, actions: &mut Vec<Self::Action>) {
        // compute the set of possible actions
        actions.extend(Self::next_actions(&state.executor))
    }

    fn next_state(
        &self,
        previous_state: &Self::State,
        action: Self::Action,
    ) -> Option<Self::State> {
        let mut next_state = previous_state.clone();
        // apply the action and ignore its result
        let _ = next_state.executor.apply(action.clone());
        // save action performed
        next_state.action = action;
        Some(next_state)
    }

    fn properties(&self) -> Vec<stateright::Property<Self>> {
        vec![
            all_chains_can_reach_max_height(),
            some_connection_can_reaches_a_try_open_state(),
        ]
    }
}

fn all_chains_can_reach_max_height() -> stateright::Property<IBC> {
    stateright::Property::sometimes("all chains reach max height", |_, state: &IBCModelState| {
        // \A chainId \in ChainIds
        CHAIN_IDS.into_iter().all(|chain_id| {
            let ctx = state.executor.chain_context(chain_id.to_string());
            // chains[chainsId].height == MAX_CHAIN_HEIGHT
            ctx.host_current_height().revision_height == MAX_CHAIN_HEIGHT
        })
    })
}

fn some_connection_can_reaches_a_try_open_state() -> stateright::Property<IBC> {
    stateright::Property::sometimes(
        "some connection reaches a try open state",
        |_, state: &IBCModelState| {
            // \E chainId \in ChainIds
            CHAIN_IDS.into_iter().any(|chain_id| {
                let ctx = state.executor.chain_context(chain_id.to_string());
                // \E connectionId \in ConnectionIds
                IBC::connection_ids().any(|connection_id| {
                    let connection =
                        ctx.connection_end(&IBCTestExecutor::connection_id(connection_id));
                    // connections[connectionId].state == TRY_OPEN
                    connection
                        .iter()
                        .any(|connection| connection.state() == &ConnectionState::TryOpen)
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
    IBC.checker()
        .threads(dbg!(num_cpus::get()))
        // .serve("localhost:3000");
        .spawn_dfs()
        .report(&mut std::io::stdout())
        .join()
        .assert_properties()
}
