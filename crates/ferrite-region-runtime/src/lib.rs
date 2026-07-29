#![forbid(unsafe_code)]

//! Local and Lattice-backed Region placement, routing, fencing, and handoff.

pub mod immediate;
pub mod lattice;
pub mod local;
pub mod logic;
pub mod transfer;
