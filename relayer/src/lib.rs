#![forbid(unsafe_code)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_qualifications,
    rust_2018_idioms
)]
#![allow(clippy::too_many_arguments)]
// TODO: disable unwraps:
//  https://github.com/informalsystems/ibc-rs/issues/987
// #![cfg_attr(not(test), deny(clippy::unwrap_used))]

//! IBC Relayer implementation as a library.
//!
//! For the IBC relayer binary, please see [Hermes] (`ibc-relayer-cli` crate).
//!
//! [Hermes]: https://docs.rs/ibc-relayer-cli/0.2.0/

extern crate alloc;

pub mod account;
pub mod chain;
pub mod config;
pub mod connection;
pub mod error;
pub mod event;
pub mod keyring;
pub mod light_client;
pub mod macros;
pub mod object;
pub mod path;
pub mod telemetry;
pub mod util;
