#![forbid(unsafe_code)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    rust_2018_idioms
)]
#![feature(type_ascription)]
#![feature(allocator_api)]
// TODO: disable unwraps:
//  https://github.com/informalsystems/ibc-rs/issues/987
// #![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! IBC Relayer implementation as a library.
//!
//! For the IBC relayer binary, please see [Hermes] (`ibc-relayer-cli` crate).
//!
//! [Hermes]: https://docs.rs/ibc-relayer-cli/0.2.0/

#[cfg(feature="prusti")]
use prusti_contracts::*;

pub mod chain;

pub mod channel;
pub mod config;
pub mod connection;
pub mod error;
pub mod event;
pub mod foreign_client;
pub mod keyring;
pub mod light_client;
pub mod link;
pub mod macros;
pub mod object;
pub mod registry;
pub mod sdk_error;
pub mod supervisor;
pub mod telemetry;
pub mod transfer;
pub mod upgrade_chain;
pub mod util;
pub mod worker;

#[cfg(feature = "prusti")]
#[extern_spec]
impl <T> std::option::Option<T> {
    #[pure]
    #[ensures(matches!(*self, Some(_)) == result)]
    pub fn is_some(&self) -> bool;

    #[pure]
    #[requires(self.is_some())]
    pub fn unwrap(self) -> T;

}

#[cfg(feature="prusti")]
#[extern_spec]
impl<T : Copy, E: Copy + std::fmt::Debug> Result<T, E> {
    #[pure]
    #[ensures(matches!(*self, Ok(_)) == result)]
    pub fn is_ok(&self) -> bool {
        match self {
            Ok(_) => true,
            Err(_) => false
        }
    }

}
