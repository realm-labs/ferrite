use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerError};
use ferrite_simulation::command::RegionCommand;
use thiserror::Error;

pub trait RegionCommandRouter {
    fn route(&mut self, command: RegionCommand) -> Result<(), RegionRouteError>;
}

impl RegionCommandRouter for LocalRegionRunner {
    fn route(&mut self, command: RegionCommand) -> Result<(), RegionRouteError> {
        self.admit_command(command)?;
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum RegionRouteError {
    #[error(transparent)]
    Local(#[from] LocalRunnerError),
    #[error("distributed Region route is unavailable")]
    Unavailable,
    #[error("distributed Region route reached its bounded mailbox")]
    Full,
    #[error("distributed Region route rejected a stale activation generation")]
    StaleGeneration,
}
