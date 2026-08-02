#![forbid(unsafe_code)]

//! Deterministic fixtures and bounded behavior harnesses for Ferrite tests.

pub mod clock;
pub mod entity_service;
pub mod malformed;
pub mod player_service;
pub mod recording;
pub mod scenario;
pub mod seed;
pub mod service_conformance;
pub mod simulation;
pub mod snapshot;
pub mod world_service;
pub mod worldgen_oracle;
