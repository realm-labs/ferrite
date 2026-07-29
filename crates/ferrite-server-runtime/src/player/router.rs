use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_region_runtime::local::{LocalRegionRunner, LocalRunnerError};
use ferrite_region_runtime::transfer::EntityTransfer;
use ferrite_simulation::command::RegionCommand;
use thiserror::Error;

pub trait PlayerRegionRouter {
    fn route_player_command(
        &mut self,
        command: RegionCommand,
    ) -> Result<(), PlayerRegionRouteError>;

    fn route_player_transfer(
        &mut self,
        transfer: EntityTransfer,
    ) -> Result<(), PlayerRegionRouteError>;

    fn activation_generation(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, PlayerRegionRouteError>;
}

impl PlayerRegionRouter for LocalRegionRunner {
    fn route_player_command(
        &mut self,
        command: RegionCommand,
    ) -> Result<(), PlayerRegionRouteError> {
        self.admit_command(command)?;
        Ok(())
    }

    fn route_player_transfer(
        &mut self,
        transfer: EntityTransfer,
    ) -> Result<(), PlayerRegionRouteError> {
        self.admit_transfer(transfer)?;
        Ok(())
    }

    fn activation_generation(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, PlayerRegionRouteError> {
        Ok(LocalRegionRunner::activation_generation(self, region)?)
    }
}

#[derive(Debug, Error)]
pub enum PlayerRegionRouteError {
    #[error(transparent)]
    Local(#[from] LocalRunnerError),
    #[error("distributed player Region route is unavailable")]
    Unavailable,
    #[error("distributed player Region route reached its bounded mailbox")]
    Full,
    #[error("distributed player Region route rejected a stale activation generation")]
    StaleGeneration,
}
