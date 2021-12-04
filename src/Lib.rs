#![deny(missing_docs)]//! Vesting Demo.#
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;

pub mod error;
pub mod instruction;
pub mod processer;
pub mod state;