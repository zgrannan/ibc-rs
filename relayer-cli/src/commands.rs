//! Cli Subcommands
//!
//! This is where you specify the subcommands of your application.
//!
//! See the `impl Configurable` below for how to specify the path to the
//! application's configuration file.

use std::path::PathBuf;

use abscissa_core::{Command, Configurable, FrameworkError, Help, Options, Runnable};

use crate::config::Config;

use self::{
    create::CreateCmds, keys::KeysCmd, light::LightCmd, listen::ListenCmd, query::QueryCmd,
    start::StartCmd, start_multi::StartMultiCmd, tx::TxCmd, update::UpdateCmds,
    version::VersionCmd,
};
use crate::commands::misbehaviour::MisbehaviourCmd;

mod cli_utils;
mod config;
mod create;
mod keys;
mod light;
mod listen;
mod misbehaviour;
mod query;
mod start;
mod start_multi;
mod tx;
mod update;
mod version;

/// Default configuration file path
pub fn default_config_file() -> Option<PathBuf> {
    dirs_next::home_dir().map(|home| home.join(".hermes/config.toml"))
}

// TODO: Re-add the `config` subcommand
// /// The `config` subcommand
// #[options(help = "manipulate the relayer configuration")]
// Config(ConfigCmd),

/// Cli Subcommands
#[derive(Command, Debug, Options, Runnable)]
pub enum CliCmd {
    /// The `help` subcommand
    #[options(help = "Get usage information")]
    Help(Help<Self>),

    /// The `keys` subcommand
    #[options(help = "Manage keys in the relayer for each chain")]
    Keys(KeysCmd),

    /// The `light` subcommand
    #[options(help = "Basic functionality for managing the light clients")]
    Light(LightCmd),

    /// The `create` subcommand
    #[options(help = "Create objects (client, connection, or channel) on chains")]
    Create(CreateCmds),

    /// The `update` subcommand
    #[options(
        help = "Update objects on chains. Currently this sub-commands serves only to update clients"
    )]
    Update(UpdateCmds),

    /// The `start` subcommand
    #[options(help = "Start the relayer")]
    Start(StartCmd),

    /// The `start-multi` subcommand
    #[options(help = "Start the relayer in multi-channel mode")]
    StartMulti(StartMultiCmd),

    /// The `query` subcommand
    #[options(help = "Query objects from the chain")]
    Query(QueryCmd),

    /// The `tx` subcommand
    #[options(help = "Create and send IBC transactions")]
    Tx(TxCmd),

    /// The `listen` subcommand
    #[options(help = "Listen to and display IBC events emitted by a chain")]
    Listen(ListenCmd),

    /// The `misbehaviour` subcommand
    #[options(help = "Listen to client update IBC events and handles misbehaviour")]
    Misbehaviour(MisbehaviourCmd),

    /// The `version` subcommand
    #[options(help = "Display version information")]
    Version(VersionCmd),
}

/// This trait allows you to define how application configuration is loaded.
impl Configurable<Config> for CliCmd {
    /// Location of the configuration file
    fn config_path(&self) -> Option<PathBuf> {
        let filename = default_config_file();
        filename.filter(|f| f.exists())
    }

    /// Apply changes to the config after it's been loaded, e.g. overriding
    /// values in a config file using command-line options.
    ///
    /// This can be safely deleted if you don't want to override config
    /// settings from command-line options.
    fn process_config(&self, config: Config) -> Result<Config, FrameworkError> {
        Ok(config)
    }
}
