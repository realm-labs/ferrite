#![forbid(unsafe_code)]

//! Canonical replay envelopes, state hashes, verification, and divergence reports.

pub mod codec;
pub mod envelope;
pub mod hash;
pub mod log;
pub mod verify;

mod semantic;
