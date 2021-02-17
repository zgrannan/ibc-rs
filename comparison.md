## Model Checking TLA+ vs Model Checking Rust

***

# WARNING

- what follows reflects my __limited experience__ with both
- I'll make __claims__, probably __wrong__ ones
    - (I might even say we don't need `TLA+`)
- nevertheless, my hope is that it can __bootstrap an interesting conversation__
- I didn't prepare well enough the most important part of this presentation; so please __interrupt me and ask questions__ when I start not making sense 

***

By `"Model Checking X"` I mean that the model of our system is written in the `X` language.

#### What's a model?

A __model__ specifies:
- a set of model __constants__
    - e.g. the number of chains and the maximum height they can reach
- a set of __initial states__
    - e.g. chains start at height 1
- a set of allowed __transitions__ at each state
    - e.g. assume that increasing the height of a chain is a transition; this transition is enabled if the chain has not reached the maximum height allowed by the model constant
- how to __apply a transition to a state__
    - e.g. effectively increasing the height of a chain (`height++`)

_Question for later:_ can the model of our system be the system itself?

#### Now what?

Now we need a __model checker__ (MC). A model checker:
- knows how to __execute code__ written in the `X` language
- generates the initial states, and __consecutively applies the allowed transitions__ _until some termination property_
- proves __safety and liveness properties__ about the system
    - e.g. height never decreases (safety)
- (and probably other fancy things I don't know about)

#### Examples of model checkers

|        |  Explicit MC   |            Symbolic MC           |
|:------:|:--------------:|:--------------------------------:|
| `TLA+` |    [`TLC`]     |           [`Apalache`]           |
| `Rust` | [`stateright`] |    (something using [`KLEE`]?)   |

[`TLC`]: https://github.com/tlaplus/tlaplus/
[`Apalache`]: https://apalache.informal.systems/
[`stateright`]: https://www.stateright.rs/
[`KLEE`]: https://klee.github.io/

The typical termination property in __Explict MC__ is _"all states have been explored"_.

`Apalache` cannot do this, and instead terminates once _"all paths with length up to `N` have been explored"_. This is known as bounded MC.

_Observation:_ in `Apalache`, a user has to bound the explored state in two ways, with the above `N` and with the model constants<sup>[1](#redundant-footnote)</sup>.

#### Code examples
##### _a set of model constants_

```tla
CONSTANTS
    ChainIds = {"chainA", "chainB", "chainC"}
    MaxChainHeight = 5
    MaxClientsPerChain = 1
```

```rust
const CHAIN_IDS: &[&'static str] = &["chainA", "chainB", "chainC"];
const MAX_CHAIN_HEIGHT: u64 = 5;
const MAX_CLIENTS_PER_CHAIN: u64 = 1;
```

##### _custom "types" based on the model constants_

```tla
Heights == 1..MaxChainHeight
ClientIds == 0..(MaxClientsPerChain - 1)
```

```rust
fn heights() -> impl Iterator<Item = u64> {
    1..=MAX_CHAIN_HEIGHT
}

fn client_ids() -> impl Iterator<Item = u64> {
    0..MAX_CLIENTS_PER_CHAIN
}
```

- transitions (aka `Actions`)

```tla
NoneActions == [
    type: {"None"}
]
CreateClientActions == [
    type: {"ICS02CreateClient"},
    chainId: ChainIds,
    clientState: Heights,
    consensusState: Heights
]
UpdateClientActions == [
    type: {"ICS02UpdateClient"},
    chainId: ChainIds,
    clientId: ClientIds,
    header: Heights
]
Actions ==
    NoneActions \union
    CreateClientActions \union
    UpdateClientActions
```

```rust
pub enum Action {
    None,
    ICS02CreateClient {
        chain_id: String,
        client_state: u64,
        consensus_state: u64,
    },
    ICS02UpdateClient {
        chain_id: String,
        client_id: u64,
        header: u64,
    },
}
```

##### _a set of initial states_
```tla
Init ==
    \* create a "none" client
    LET clientNone == [
        heights |-> AsSetInt({})
    ] IN
    \* initialize all chains with height 1
    LET emptyChain == [
        height |-> 1,
        clients |-> [clientId \in ClientIds |-> clientNone],
        clientIdCounter |-> 0
    ] IN
    /\ chains = [chainId \in ChainIds |-> emptyChain]
    /\ action = AsAction([type |-> "None"])
    /\ actionOutcome = "None"
```

```rust
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
```

__Note__: `executor.init_chain_context` is calling the existing Rust implementation, not a model of it. Because of that, in Rust we don't have an `action_outcome`. _This will be important for later_.

##### _a set of allowed transitions at each state_

```tla
Next ==
    \* select a chain id
    \E chainId \in ChainIds:
        \* perform action on chain if the model constant `MaxChainHeight` allows
        \* it
        IF chains[chainId].height < MaxChainHeight THEN
            \/ CreateClientAction(chainId)
            \/ UpdateClientAction(chainId)
            \/ UNCHANGED vars
        ELSE
            \/ UNCHANGED vars

CreateClientAction(chainId) ==
    \* select a height for the client to be created at
    \E height \in Heights:
        \* only create client if the model constant `MaxClientsPerChain` allows
        \* it
        IF chains[chainId].clientIdCounter < MaxClientsPerChain THEN
            CreateClient(chainId, height)
        ELSE
            UNCHANGED vars

UpdateClientAction(chainId) == ...
```

```rust
fn next_actions(
    state: &IBCModelState,
) -> impl Iterator<Item = step::Action> + '_ {
    // select a chain id
    CHAIN_IDS
        .into_iter()
        .filter_map(move |chain_id| {
            let ctx = state.executor.chain_context(chain_id.to_string());
            // perform action on chain if the model constant `MAX_CHAIN_HEIGHT`
            // allows it
            let allowed = ctx.host_current_height() < IBCTestExecutor::height(MAX_CHAIN_HEIGHT);
            allowed.then(|| {
                Self::create_client_actions(chain_id, ctx)
                    .chain(Self::update_client_actions(chain_id))
            })
        })
        .flatten()
}

fn create_client_actions<'a>(
    chain_id: &'static str,
    ctx: &'a MockContext,
) -> impl Iterator<Item = step::Action> + 'a {
    // select a height for the client to be created at
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

fn update_client_actions(
    chain_id: &'static str,
) -> impl Iterator<Item = step::Action> + 'a {
    ...
}
```

##### _how to apply a transition to a state_

_(this is the interesting one...)_

```tla
CreateClient(chainId, height) ==
    LET chain == chains[chainId] IN
    LET clients == chain.clients IN
    LET clientIdCounter == chain.clientIdCounter IN
    LET result == ICS02_CreateClient(clients, clientIdCounter, height) IN
    \* update the chain
    LET updatedChain == [chain EXCEPT
        !.height = UpdateChainHeight(@, result.outcome, "ICS02CreateOK"),
        !.clients = result.clients,
        !.clientIdCounter = result.clientIdCounter
    ] IN
    \* update `chains`, set the `action` and its `actionOutcome`
    /\ chains' = [chains EXCEPT ![chainId] = updatedChain]
    /\ action' = AsAction([
        type |-> "ICS02CreateClient",
        chainId |-> chainId,
        clientState |-> height,
        consensusState |-> height])
    /\ actionOutcome' = result.outcome
```

```rust
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
```

__Note__: Again, note that in Rust we don't have an `action_outcome`. _This will be important for later_.

#### The value of TLA+

A model of our system written in __TLA+ represents another source of truth__:
- if we just have the system, then the system should do what the system does
- if have both, now there needs to be a consensus on what the truth is
    - just this, by itself, can help uncovering bugs

_Going back to a previous question:_ can the model of our system be the system itself?
- yes, but it no longer represents another source of truth
- nonetheless, we can still show interesting things (like safety and liveness, which I didn't show here) 

*__THE QUESTION__:* why does the second source of truth (i.e. the model) have to be `TLA+`? can't the model be written in the same language as the implementation, i.e., in `Rust`?
- For most users, this will be more attractive as they don't have to learn `TLA+`.
- And even if users don't want to write the model, they can still check properties about the `Rust` implementation. 
- The `Rust` implementation can also be used to do MBT of other implementations (e.g., in `Go`, `TypeScript`, ...)

#### Some (very non-scientific) numbers

- `TLC`:

```bash
-----------------------
2 chains [WITHOUT -workers auto]: 179s

Time	    Diameter  Found        Distinct
00:02:59    16	      16210701     368425

TPUT: 1946 distinct states/s
-----------------------
2 chains [WITH]: 25s

Time	    Diameter  Found        Distinct
00:00:25    16        16210701     368425

TPUT: 15K  distinct states/s (7x)
-----------------------
```

> _bad defaults can scare users away_

- `stateright`:

By default, `anomaly`, the crate used for handling errors, comes with a `backtrace` feature enabled. This allows us to have a full stacktrace for errors. If errors occur with high frequency (_and they do when MC_), this creates contention on some lock outside of Rust.

To avoid this, we can disable `backtrace` feature in all crates using `anomaly`:

```diff
-anomaly = "0.2"
+anomaly = { version = "0.2", default-features = false }
```


```bash
-----------------------
3 chains [with backtrace]: 2m22s

Done. generated=175616, sec=142
-----------------------
3 chains [without backtrace]: 3s

Done. generated=175616, sec=3

TPUT: 58K distinct states/s (47x)
-----------------------
4 chains [without backtrace]: 3m4s

Done. generated=9834496, sec=182

TPUT: 54K distinct states/s
-----------------------
```

> _model checkers can be used to detect performance issues; they can even come with an built-in profiler (e.g. that generates flamegraphs if enabled)_

***

<a name="redundant-footnote">[1]</a>: _Question:_ assume that, given the model constants, the model only has paths of length `3`; if an `Apalache` user supplies `N = 30`, will model checking be as fast as if the user had supplied `N = 3`, or will `Apalache` keep doing redundant work?
