//! Durable world generation, lifecycle, recovery, handoff, and inspection integration.

pub mod continuity;
pub(crate) mod dimension;
pub mod environment;
pub(crate) mod formal_lifecycle;
pub(crate) mod formal_persistence;
pub mod inspection;
pub mod lifecycle;
pub(crate) mod metadata;
pub mod model;
pub mod runtime;
pub(crate) mod spawn;
