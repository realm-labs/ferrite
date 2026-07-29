#![forbid(unsafe_code)]

//! Versioned Region snapshots, journals, migrations, and recovery.

mod codec;

pub mod dirty;
pub mod recovery;
pub mod snapshot;
pub mod store;
