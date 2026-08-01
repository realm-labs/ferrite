#![forbid(unsafe_code)]

//! Server composition, session lifecycle, admission, and semantic projection.

pub mod chunk;
pub mod composite;
pub mod config;
pub mod conformance;
pub mod continuity;
pub mod entity_service;
pub mod lifecycle;
pub mod management;
mod minecraft;
pub mod player;
pub mod player_service;
pub mod process;
mod runtime_status;
pub mod session;
pub mod simulation;
pub mod world_config;
pub mod world_service;
