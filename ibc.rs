extern crate prusti_contracts;
use prusti_contracts::*;

type AnyConsensusState = u32;
type AnyClientState = u32;
type ClientId = u32;
type MockClientRecord = u32;
type Clients = u32;
type Height = u32;
type Ics02Error = u32;
type ClientType = u32;

#[derive(Clone, Copy)]
pub struct CreateResult {
    pub client_id: ClientId,
    pub client_type: ClientType,
    pub client_state: AnyClientState,
    pub consensus_state: AnyConsensusState,
}

#[derive(Clone, Copy)]
pub struct UpdateResult {
    pub client_id: ClientId,
    pub client_state: AnyClientState,
    pub consensus_state: AnyConsensusState,
}

#[derive(Clone, Copy)]
pub struct UpgradeResult {
    pub client_id: ClientId,
    pub client_state: AnyClientState,
    pub consensus_state: AnyConsensusState,
}

#[derive(Clone, Copy)]
pub enum ClientResult {
    Create(CreateResult),
    Update(UpdateResult),
    Upgrade(UpgradeResult)
}


#[macro_export]
macro_rules! handle_result {
    ($e: expr) => {
        match $e {
            Ok(data) => data,
            Err(err) => return Err(err)
        }
    };
}


#[pure]
#[trusted]
fn get_client_state(client: MockClientRecord) -> Option<AnyClientState> {
   unreachable!()
}

#[pure]
#[trusted]
fn get_client_type(client: MockClientRecord) -> ClientType {
   unreachable!()
}

#[extern_spec]
impl std::option::Option<u32> {
    #[pure]
    #[ensures(matches!(*self, Some(_)) == result)]
    pub fn is_some(&self) -> bool;

    #[pure]
    #[ensures(self.is_some() == !result)]
    pub fn is_none(&self) -> bool;

    #[pure]
    #[requires(self.is_some())]
    pub fn unwrap(self) -> u32;
}

#[pure]
fn client_state_equals(
    client: MockClientRecord,
    state: AnyClientState
) -> bool {
    match get_client_state(client) {
        Some(cs) => cs == state,
        None     => false
    }
}

#[pure]
#[trusted]
fn get_client_consensus_state(
    client_id: ClientId,
    height: Height,
    context: &MockContext) -> AnyConsensusState {
    unreachable!()
}

#[pure]
#[trusted]
fn get_latest_height(cs: AnyClientState) -> Height {
   unreachable!()
}

#[pure]
#[trusted]
fn consensus_states_is_empty(client: MockClientRecord) -> bool {
   unreachable!()
}

#[pure]
#[trusted]
fn get_max_consensus_state_height(client: MockClientRecord) -> Option<Height> {
   unreachable!()
}


#[pure]
pub fn client_invariant(client: MockClientRecord) -> bool {
    match get_client_state(client) {
        Some(cs) => {
            let hcs = get_max_consensus_state_height(client);
            hcs.is_some() && hcs.unwrap() == get_latest_height(cs)
        },
        None => consensus_states_is_empty(client)
    }
}

#[pure]
#[trusted]
fn contains_key(clients: Clients, client_id: ClientId) -> bool {
   unreachable!()
}

#[pure]
#[trusted]
#[requires(contains_key(clients, client_id))]
fn get_client(clients: Clients, client_id: ClientId) -> MockClientRecord {
   unreachable!()
}

predicate! {
    fn mock_context_invariant(context: &MockContext) -> bool {
        clients_invariant(context.clients)
    }
}
predicate! {
    fn clients_invariant(clients: Clients) -> bool {
        forall(|client_id: ClientId| contains_key(clients, client_id) ==> client_invariant(get_client(clients, client_id)))
    }
}

struct MockContext {
    clients: Clients,
    client_ids_counter: u64
}

#[pure]
fn get_cid(res: ClientResult) -> ClientId {
   match res {
        ClientResult::Create(res) => res.client_id,
        ClientResult::Update(res) => res.client_id,
        ClientResult::Upgrade(res) => res.client_id
   }
}

impl MockContext {

    #[requires(clients_invariant(self.clients))]
    #[ensures(
        forall(|cid: ClientId|
            contains_key(self.clients, cid) && get_cid(handler_res) != cid ==>
                get_client(self.clients, cid) == get_client(old(self.clients), cid)))
    ]
    #[ensures(matches!(result, Ok(_)) ==> client_invariant(get_client(self.clients, get_cid(handler_res))))]
    #[ensures(matches!(result, Ok(_)) ==> clients_invariant(self.clients))]
    fn store_client_result(&mut self, handler_res: ClientResult) -> Result<(), Ics02Error> {
        match handler_res {
            ClientResult::Create(res) => {
                let client_id = res.client_id;
                handle_result!(self.store_client_type(client_id, res.client_type));
                handle_result!(self.store_client_state(client_id, res.client_state));
                handle_result!(self.store_consensus_state(
                    client_id,
                    get_latest_height(res.client_state),
                    res.consensus_state,
                ));
                self.increase_client_counter();
                Ok(())
            }
            ClientResult::Update(res) => {
                handle_result!(self.store_client_state(res.client_id, res.client_state));
                handle_result!(self.store_consensus_state(
                    res.client_id,
                    get_latest_height(res.client_state),
                    res.consensus_state,
                ));
                Ok(())
            }
            ClientResult::Upgrade(res) => {
                handle_result!(self.store_client_state(res.client_id, res.client_state));
                handle_result!(self.store_consensus_state(
                    res.client_id,
                    get_latest_height(res.client_state),
                    res.consensus_state,
                ));
                Ok(())
            }
        }
    }

    #[ensures(self.clients == old(self.clients))]
    fn increase_client_counter(&mut self) {
        self.client_ids_counter += 1
    }

    #[ensures(
        forall(|cid: ClientId|
            (contains_key(old(self.clients), cid) || cid == client_id) ==
                contains_key(self.clients, cid)))
    ]
    #[ensures(
        forall(|cid: ClientId|
            contains_key(self.clients, cid) && client_id != cid ==>
                get_client(self.clients, cid) == get_client(old(self.clients), cid)))
    ]
    #[ensures(
        contains_key(self.clients, client_id) &&
            client_state_equals(get_client(self.clients, client_id), client_state)
    )]
    #[ensures(
        contains_key(old(self.clients), client_id) ==>
            get_max_consensus_state_height(get_client(self.clients, client_id)) ==
                get_max_consensus_state_height(get_client(old(self.clients), client_id))
    )]
    #[ensures(
        !contains_key(old(self.clients), client_id) ==>
            consensus_states_is_empty(get_client(self.clients, client_id))
    )]
    #[trusted]
    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: AnyClientState,
    ) -> Result<(), Ics02Error> {
        unreachable!()
    }

    #[ensures(
        forall(|cid: ClientId|
            (contains_key(old(self.clients), cid) || cid == client_id) ==
                contains_key(self.clients, cid)))
    ]
    #[ensures(
        forall(|cid: ClientId|
            contains_key(self.clients, cid) && client_id != cid ==>
                get_client(self.clients, cid) == get_client(old(self.clients), cid)))
    ]
    #[trusted]
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), Ics02Error> {
        unreachable!()
    }

    #[ensures(
        forall(|cid: ClientId|
            (contains_key(old(self.clients), cid) || cid == client_id) ==
                contains_key(self.clients, cid)))
    ]
    #[ensures(
        forall(|cid: ClientId|
            contains_key(self.clients, cid) && client_id != cid ==>
                get_client(self.clients, cid) == get_client(old(self.clients), cid)))
    ]
    #[ensures(
        contains_key(self.clients, client_id) &&
            get_max_consensus_state_height(get_client(self.clients, client_id)).is_some() &&
            get_max_consensus_state_height(get_client(self.clients, client_id)).unwrap() == height
    )]
    #[ensures(
        contains_key(old(self.clients), client_id) ==>
            get_client_state(get_client(self.clients, client_id)) ==
                get_client_state(get_client(old(self.clients), client_id))
    )]
    #[ensures(!consensus_states_is_empty(get_client(self.clients, client_id)))]
    #[trusted]
    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: AnyConsensusState
    ) -> Result<(), Ics02Error> {
        unreachable!()
    }
}

fn main(){}
