use ferrite_foundation::identity::ActivationGeneration;
use ferrite_foundation::region::SimulationRegionKey;
use ferrite_region_runtime::lattice::remoting::{LatticeRemotingAdapter, RemotingAdapterError};
use ferrite_region_runtime::lattice::semantic::{
    SemanticRemotingError, decode_entity_transfer, decode_region_command, encode_entity_transfer,
    encode_region_command,
};
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

pub struct LatticePlayerRegionRouter<'a> {
    runner: &'a mut LocalRegionRunner,
    ingress: SimulationRegionKey,
    ingress_generation: ActivationGeneration,
    adapter: LatticeRemotingAdapter,
}

impl<'a> LatticePlayerRegionRouter<'a> {
    pub fn new(
        runner: &'a mut LocalRegionRunner,
        ingress: SimulationRegionKey,
        ingress_generation: ActivationGeneration,
        maximum_frame_bytes: usize,
    ) -> Result<Self, PlayerRegionRouteError> {
        Ok(Self {
            runner,
            ingress,
            ingress_generation,
            adapter: LatticeRemotingAdapter::new(maximum_frame_bytes)?,
        })
    }

    fn transport_command(
        &self,
        command: &RegionCommand,
    ) -> Result<RegionCommand, PlayerRegionRouteError> {
        let target_generation = self.runner.activation_generation(command.target())?;
        let envelope = encode_region_command(
            self.ingress.clone(),
            self.ingress_generation,
            target_generation,
            command,
        )?;
        let frame = self.adapter.encode(&envelope)?;
        let received = self.adapter.decode(&frame)?;
        if received.target_generation() != target_generation {
            return Err(PlayerRegionRouteError::StaleGeneration);
        }
        Ok(decode_region_command(&received)?)
    }

    fn transport_transfer(
        &self,
        transfer: &EntityTransfer,
    ) -> Result<EntityTransfer, PlayerRegionRouteError> {
        let envelope = encode_entity_transfer(transfer)?;
        let frame = self.adapter.encode(&envelope)?;
        let received = self.adapter.decode(&frame)?;
        Ok(decode_entity_transfer(&received)?)
    }
}

impl PlayerRegionRouter for LatticePlayerRegionRouter<'_> {
    fn route_player_command(
        &mut self,
        command: RegionCommand,
    ) -> Result<(), PlayerRegionRouteError> {
        let command = self.transport_command(&command)?;
        self.runner.admit_command(command)?;
        Ok(())
    }

    fn route_player_transfer(
        &mut self,
        transfer: EntityTransfer,
    ) -> Result<(), PlayerRegionRouteError> {
        let transfer = self.transport_transfer(&transfer)?;
        self.runner.admit_transfer(transfer)?;
        Ok(())
    }

    fn activation_generation(
        &self,
        region: &SimulationRegionKey,
    ) -> Result<ActivationGeneration, PlayerRegionRouteError> {
        Ok(self.runner.activation_generation(region)?)
    }
}

#[derive(Debug, Error)]
pub enum PlayerRegionRouteError {
    #[error(transparent)]
    Local(#[from] LocalRunnerError),
    #[error(transparent)]
    Remoting(#[from] RemotingAdapterError),
    #[error(transparent)]
    Semantic(#[from] SemanticRemotingError),
    #[error("distributed player Region route is unavailable")]
    Unavailable,
    #[error("distributed player Region route reached its bounded mailbox")]
    Full,
    #[error("distributed player Region route rejected a stale activation generation")]
    StaleGeneration,
}
