# ADR 010: Unified approach for CLI arguments for Hermes v1.0.0

## Changelog
* 15.06.2022: Proposed.
* 28.06.2022: Accepted.

## Context

In this ADR we provide recommendations and intuitions to using flags for all the arguments of the Hermes commands.

The problem we are trying to solve is a unified approach to CLI arguments for Hermes v1.0.0.

## Decision

To avoid confusion, all the parameters should take long flags. The following approach should be applied:

* Only long flags are used in order to avoid having nonintuitive flags or conflicting flags.
* Any parameter ending with `_id` should have the `_id` removed from the flag to shorten it. For example the flag for `chain_id` should only be `chain`.
* Flags which can be shortened and still be meaningful should have a shortened alias. This is done for `connection`, `channel` and `sequence`, which have respectively `conn`, `chan` and `seq` aliases.
* For the channel and connection creation CLIs, the objects at the two ends are prefixed by `--a-` and `--b-`. Example `--a-chain` and `--b-chain`.
* Whenever `chain`, `conn`, `chan` and `port` flags have to be disambiguated with a specifier, the specifier will be a prefix. Example of specifiers we currently use are `host`, `reference`, `a`, `b` and `counterparty`.

The following commands are implemented, with the binary name `hermes` often omitted:

### Hermes global flags

* `hermes --config <CONFIG> <COMMAND>`

* `hermes --json <COMMAND>`

### Commands for clients

* `create client --host-chain <HOST_CHAIN_ID> --reference-chain <REFERENCE_CHAIN_ID>`
    * Optional: `[--clock-drift <CLOCK_DRIFT>] [--trust-threshold <TRUST_THRESHOLD>] [--trusting-period <TRUSTING_PERIOD>]`

* `update client --host-chain <HOST_CHAIN_ID> --client <CLIENT_ID>`
    * Optional: `[--height <REFERENCE_HEIGHT>] [--trusted-height <REFERENCE_TRUSTED_HEIGHT>]`

* `upgrade client --host-chain <HOST_CHAIN_ID> --client <CLIENT_ID> --upgrade-height <REFERENCE_UPGRADE_HEIGHT>`

* `upgrade clients --reference-chain <REFERENCE_CHAIN_ID> --upgrade-height <REFERENCE_UPGRADE_HEIGHT>`
    * Optional: `[--host-chain <HOST_CHAIN_ID>]`

### Create a connection

* `create connection --a-chain <A_CHAIN_ID> --b-chain <B_CHAIN_ID>`
    * Optional: `[--delay <DELAY>]`

* `create connection --a-chain <A_CHAIN_ID> --a-client <A_CLIENT_ID> --b-client <B_CLIENT_ID>`
    * Optional: `[--delay <DELAY>]`

### Create a channel

* `create channel --a-chain <A_CHAIN_ID> --a-connection <A_CONNECTION_ID> --a-port <A_PORT_ID> --b-port <B_PORT_ID>`
    * Optional: `[--channel-version <VERSION>] [--order <ORDER>]`

* `create channel --a-chain <A_CHAIN_ID> --b-chain <B_CHAIN_ID> --a-port <A_PORT_ID> --b-port <B_PORT_ID> --new-client-connection`
    * Optional: `[--channel-version <VERSION>] [--order <ORDER>] [--yes]`

### Commands for keys

* `keys add --chain <CHAIN_ID> --key-file <KEY_FILE>`
    * Optional: `[--hd-path <HD_PATH>] [--key-name <KEY_NAME>]`

* `keys add --chain <CHAIN_ID> --mnemonic-file <MNEMONIC_FILE>`
    * Optional: `[--hd-path <HD_PATH>] [--key-name <KEY_NAME>]`

* `keys balance --chain <CHAIN_ID>`
    * Optional: `[--key-name <KEY_NAME>]`

* `keys delete --chain <CHAIN_ID> --all`

* `keys delete --chain <CHAIN_ID> --key-name <KEY_NAME>`

* `keys list --chain <CHAIN_ID>`

### Listen

* `listen --chain <CHAIN_ID>`
    * Optional: `[--events <EVENT>...]`

### Misbehaviour

* `misbehaviour --chain <CHAIN_ID> --client <CLIENT_ID>`

### Start the relayer in multi-chain mode

* `start`
    * Optional: `[--full-scan]`

### Clear packets

* `clear packets [OPTIONS] --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`
    * Optional: `[--key-name <KEY>] [--counterparty-key-name <KEY>]`

### Queries

__Client__

* `query client connections --chain <CHAIN_ID> --client <CLIENT_ID>`
    * Optional: `[--height <HEIGHT>]`

* `query client consensus --chain <CHAIN_ID> --client <CLIENT_ID>`
    * Optional: `[--consensus-height <CONSENSUS_HEIGHT>] [--height <HEIGHT>] [--heights-only]`

* `query client header --chain <CHAIN_ID> --client <CLIENT_ID> --consensus-height <CONSENSUS_HEIGHT>`
    * Optional: `[--height <HEIGHT>]`

* `query client state --chain <CHAIN_ID> --client <CLIENT_ID>`
    * Optional: `[--height <HEIGHT>]`

* `query clients --host-chain <HOST_CHAIN_ID>`
    * Optional: `[--omit-chain-ids] [--reference-chain <REFERENCE_CHAIN_ID>]`

__Connection__

* `query connection channels --chain <CHAIN_ID> --connection <CONNECTION_ID>`

* `query connection end --chain <CHAIN_ID> --connection <CONNECTION_ID>`
    * Optional: `[--height <HEIGHT>]`

* `query connections --chain <CHAIN_ID>`
    * Optional: `[--counterparty-chain <COUNTERPARTY_CHAIN_ID>] [--verbose]`

__Channel__

* `query channel client --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

* `query channel end --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`
    * Optional: `[--height <HEIGHT>]`

* `query channel full --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`
    * Optional: `[--height <HEIGHT>] [--verbose]`

* `query channels --chain <CHAIN_ID>`
    * Optional: `[--counterparty-chain <COUNTERPARTY_CHAIN_ID>] [--verbose]`

__Packet__

* `query packet ack --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID> --sequence <SEQUENCE>`
    * Optional: `[--height <HEIGHT>]`

* `query packet acks --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

* `query packet commitment --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID> --sequence <SEQUENCE>`
    * Optional: `[--height <HEIGHT>]`

* `query packet commitments --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

* `query packet pending --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

* `query packet pending-acks --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

* `query packet pending-sends --chain <CHAIN_ID> --port <PORT_ID> --channel <CHANNEL_ID>`

__Transfer__

* `query transfer denom-trace --chain <CHAIN_ID> --hash <HASH>`

__Tx__

* `query tx events --chain <CHAIN_ID> --hash <HASH>`

### Shell completion

* `completions --shell <SHELL>`

### Validate configuration file

* `config validate`

### Health check

* `health-check`

### Tx

__conn-init__

* `tx conn-init --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-client <B_CLIENT_ID> --a-client <A_CLIENT_ID>`

__conn-ack__

* `tx conn-ack --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-client <B_CLIENT_ID> --a-client <A_CLIENT_ID> --b-connection <B_CONNECTION_ID> --a-connection <A_CONNECTION_ID>`

__conn-confirm__

* `tx conn-confirm --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-client <B_CLIENT_ID> --a-client <A_CLIENT_ID> --b-connection <B_CONNECTION_ID> --a-connection <A_CONNECTION_ID>`

__conn-try__

* `tx conn-try [OPTIONS] --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-client <B_CLIENT_ID> --a-client <A_CLIENT_ID> --a-connection <A_CONNECTION_ID>`
    * Optional: `[--b-connection <B_CONNECTION_ID>]`

__chan-open-init__

* `tx chan-open-init [OPTIONS] --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID>`
    * Optional: `[--order <ORDER>]`

__chan-open-ack__

* `tx chan-open-ack --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID> --b-channel <B_CHANNEL_ID> --a-channel <A_CHANNEL_ID>`

__chan-open-confirm__

* `tx chan-open-confirm --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID> --b-channel <B_CHANNEL_ID> --a-channel <A_CHANNEL_ID>`

__chan-open-try__

* `tx chan-open-try [OPTIONS] --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID> --a-channel <A_CHANNEL_ID>`
    * Optional: `[--b-channel <B_CHANNEL_ID>]`

__chan-close-init__

* `tx chan-close-init --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID> --b-channel <B_CHANNEL_ID> --a-channel <A_CHANNEL_ID>`

__chan-close-confirm__

* `tx chan-close-confirm --b-chain <B_CHAIN_ID> --a-chain <A_CHAIN_ID> --b-connection <B_CONNECTION_ID> --b-port <B_PORT_ID> --a-port <A_PORT_ID> --b-channel <B_CHANNEL_ID> --a-channel <A_CHANNEL_ID>`

__upgrade-chain__

* `tx upgrade-chain [OPTIONS] --receiver-chain <RECEIVER_CHAIN_ID> --sender-chain <SENDER_CHAIN_ID> --sender-client <SENDER_CLIENT_ID> --amount <AMOUNT> --height-offset <HEIGHT_OFFSET>`
    * Optional: `[--denom <DENOM>] [--new-chain <CHAIN_ID>] [--new-unbonding <UNBONDING_PERIOD>] [--upgrade-name <UPGRADE_NAME>]`

__packet-recv__

* `tx raw packet-recv --receiver-chain <RECEIVER_CHAIN_ID> --sender-chain <SENDER_CHAIN_ID> --sender-port <SENDER_PORT_ID> --sender-channel <SENDER_CHANNEL_ID>`

__packet-ack__

* `tx packet-ack --receiver-chain <RECEIVER_CHAIN_ID> --sender-chain <SENDER_CHAIN_ID> --sender-port <SENDER_PORT_ID> --sender-channel <SENDER_CHANNEL_ID>`

__ft-transfer__

* `tx ft-transfer [OPTIONS] --receiver-chain <RECEIVER_CHAIN_ID> --sender-chain <SENDER_CHAIN_ID> --sender-port <SENDER_PORT_ID> --sender-channel <SENDER_CHANNEL_ID> --amount <AMOUNT>`
    * Optional: `[--denom <DENOM>] [--key-name <KEY_NAME>] [--number-msgs <NUMBER_MSGS>] [--receiver <RECEIVER>] [--timeout-height-offset <TIMEOUT_HEIGHT_OFFSET>] [--timeout-seconds <TIMEOUT_SECONDS>]`

## Status

Proposed.

__17.06.22__

The following are not yet implemented:
* Optional flags for `upgrade clients`, issue [#2311](https://github.com/informalsystems/ibc-rs/issues/2311)
* Optional flags for `query connections`, issue [#2310](https://github.com/informalsystems/ibc-rs/issues/2310)
* Updating `query channel ends` to `query channel full`

The PR which updates the flags for all the commands as described in this ADR: [#2275](https://github.com/informalsystems/ibc-rs/pull/2275)

__07.07.22__

Added `tx raw` commands to the ADR

__08.07.22__

* Created a new PR, [#2384](https://github.com/informalsystems/ibc-rs/pull/2384), to add the optional flag for the `upgrade clients` command, issue [#2311](https://github.com/informalsystems/ibc-rs/issues/2311)

__11.07.22__

* Created a new PR, [#2391](https://github.com/informalsystems/ibc-rs/pull/2391), to add the optional flags for the `query connections` command, issue [#2310](https://github.com/informalsystems/ibc-rs/issues/2310)

## Consequences

### Positive

* Clear parameters for Hermes commands

### Negative

* Breaking changes which will require updating anything using Hermes

### Neutral

## References

* Proposal in issue: [#2239](https://github.com/informalsystems/ibc-rs/issues/2239)
