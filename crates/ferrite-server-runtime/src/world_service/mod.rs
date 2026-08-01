//! Durable world generation, lifecycle, recovery, handoff, and inspection integration.

pub mod continuity;
pub(crate) mod formal_persistence;
pub mod inspection;
pub mod lifecycle;
pub(crate) mod metadata;
pub mod model;
pub mod runtime;
