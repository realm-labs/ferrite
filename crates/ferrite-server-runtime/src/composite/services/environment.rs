use std::collections::BTreeMap;

use ferrite_foundation::coordinate::{BlockPos, ChunkPos};
use ferrite_foundation::resource::ResourceId;
use ferrite_simulation::scheduled_tick::record::{ScheduledTick, TickPriority};
use ferrite_simulation::tick::GameTick;
use ferrite_world::id::BlockStateId;

use super::{
    AuthoritativeBlockUpdate, ChunkActivity, ChunkLifecycle, CompositeProductionRegionRuntime,
    CompositeServiceRuntimeError, ScheduledQueueKind, WorldServiceRuntimeError,
    encode_block_projection,
};

impl CompositeProductionRegionRuntime {
    pub(super) fn execute_environment_work(
        &mut self,
        tick: GameTick,
    ) -> Result<(), CompositeServiceRuntimeError> {
        const MAXIMUM_SCHEDULED_PER_QUEUE: usize = 64;
        let activities = self.world.chunks().collect::<BTreeMap<_, _>>();
        let in_range = |chunk: ChunkPos| {
            activities
                .get(&chunk)
                .is_some_and(|lifecycle| lifecycle.activity >= ChunkActivity::BlockTicking)
        };
        let mut block_ticks = Vec::new();
        self.simulation.tick_scheduled(
            ScheduledQueueKind::Block,
            MAXIMUM_SCHEDULED_PER_QUEUE,
            in_range,
            |scheduled| block_ticks.push(scheduled),
        );
        let mut fluid_ticks = Vec::new();
        self.simulation.tick_scheduled(
            ScheduledQueueKind::Fluid,
            MAXIMUM_SCHEDULED_PER_QUEUE,
            in_range,
            |scheduled| fluid_ticks.push(scheduled),
        );

        let mut projection_index = 0_u64;
        for scheduled in block_ticks {
            self.execute_scheduled_block(tick, scheduled, &mut projection_index)?;
        }
        for scheduled in fluid_ticks {
            self.execute_scheduled_fluid(tick, scheduled, &mut projection_index)?;
        }
        self.schedule_random_environment_work(&activities)?;
        Ok(())
    }

    fn execute_scheduled_block(
        &mut self,
        tick: GameTick,
        scheduled: ScheduledTick<ResourceId>,
        projection_index: &mut u64,
    ) -> Result<(), CompositeServiceRuntimeError> {
        if scheduled.type_identity != ResourceId::minecraft("fire").expect("static identity")
            || self.world_block(scheduled.position) != Some(ferrite_world::id::FIRE)
        {
            return Ok(());
        }
        let extinguish = self
            .simulation
            .gameplay_random_mut()
            .next_u64()
            .is_multiple_of(3);
        if extinguish {
            self.set_environment_block(
                tick,
                projection_index,
                scheduled.position,
                ferrite_world::id::AIR,
            )?;
        } else {
            let delay = i32::from(ferrite_gameplay::environment::fire::FIRE_SCHEDULE_BASE)
                + (self.simulation.gameplay_random_mut().next_u64()
                    % u64::from(ferrite_gameplay::environment::fire::FIRE_SCHEDULE_SPREAD))
                    as i32;
            self.simulation.schedule_local(
                ScheduledQueueKind::Block,
                scheduled.type_identity,
                scheduled.position,
                delay,
                TickPriority::Normal,
            )?;
        }
        Ok(())
    }

    fn execute_scheduled_fluid(
        &mut self,
        tick: GameTick,
        scheduled: ScheduledTick<ResourceId>,
        projection_index: &mut u64,
    ) -> Result<(), CompositeServiceRuntimeError> {
        use ferrite_gameplay::environment::fluid::{FluidFamily, fluid_parameters};
        let (state, family) = if scheduled.type_identity
            == ResourceId::minecraft("water").expect("static identity")
        {
            (ferrite_world::id::WATER, FluidFamily::Water)
        } else if scheduled.type_identity == ResourceId::minecraft("lava").expect("static identity")
        {
            (ferrite_world::id::LAVA, FluidFamily::Lava)
        } else {
            return Ok(());
        };
        if self.world_block(scheduled.position) != Some(state) {
            return Ok(());
        }
        let candidates = [
            BlockPos::new(
                scheduled.position.x,
                scheduled.position.y - 1,
                scheduled.position.z,
            ),
            BlockPos::new(
                scheduled.position.x + 1,
                scheduled.position.y,
                scheduled.position.z,
            ),
            BlockPos::new(
                scheduled.position.x - 1,
                scheduled.position.y,
                scheduled.position.z,
            ),
            BlockPos::new(
                scheduled.position.x,
                scheduled.position.y,
                scheduled.position.z + 1,
            ),
            BlockPos::new(
                scheduled.position.x,
                scheduled.position.y,
                scheduled.position.z - 1,
            ),
        ];
        let destination = candidates.into_iter().find(|position| {
            position.chunk() == scheduled.position.chunk()
                && self.world_block(*position) == Some(ferrite_world::id::AIR)
        });
        let Some(destination) = destination else {
            return Ok(());
        };
        self.set_environment_block(tick, projection_index, destination, state)?;
        let delay = i32::from(fluid_parameters(family, false).tick_delay);
        self.simulation.schedule_local(
            ScheduledQueueKind::Fluid,
            scheduled.type_identity,
            destination,
            delay,
            TickPriority::Normal,
        )?;
        Ok(())
    }

    fn schedule_random_environment_work(
        &mut self,
        activities: &BTreeMap<ChunkPos, ChunkLifecycle>,
    ) -> Result<(), CompositeServiceRuntimeError> {
        const RANDOM_TICK_SPEED: usize = 3;
        let mut scheduled = Vec::new();
        for (position, lifecycle) in activities {
            if lifecycle.activity < ChunkActivity::BlockTicking {
                continue;
            }
            let Some(sections) = self
                .world
                .chunk(*position)
                .map(|chunk| chunk.layout().sections())
            else {
                continue;
            };
            let base_x = position.checked_min_block_x()?;
            let base_z = position.checked_min_block_z()?;
            for section_y in sections.minimum()..sections.maximum_exclusive() {
                for _ in 0..RANDOM_TICK_SPEED {
                    let sample = self
                        .simulation
                        .next_random_position(BlockPos::new(base_x, section_y * 16, base_z), 15);
                    if let Some(state) = self.world_block(sample) {
                        scheduled.push((sample, state));
                    }
                }
            }
        }
        for (position, state) in scheduled {
            if let Some((kind, identity, delay)) = random_schedule(state) {
                self.simulation.schedule_local(
                    kind,
                    identity,
                    position,
                    delay,
                    TickPriority::Normal,
                )?;
            }
        }
        Ok(())
    }

    fn world_block(&self, position: BlockPos) -> Option<BlockStateId> {
        self.world
            .chunk(position.chunk())?
            .block_state(position)
            .ok()
    }

    fn set_environment_block(
        &mut self,
        tick: GameTick,
        projection_index: &mut u64,
        position: BlockPos,
        state: BlockStateId,
    ) -> Result<(), CompositeServiceRuntimeError> {
        self.require_projection_capacity(1)?;
        let expected_revision = self
            .world
            .chunk(position.chunk())
            .ok_or(WorldServiceRuntimeError::ChunkNotLoaded(position.chunk()))?
            .revision();
        self.world.set_block(
            self.coordinator.key(),
            self.coordinator.generation(),
            expected_revision,
            position,
            state,
        )?;
        *projection_index += 1;
        let sequence = tick
            .get()
            .saturating_mul(128)
            .saturating_add(*projection_index)
            .max(1);
        self.coordinator.queue_projection(encode_block_projection(
            sequence,
            AuthoritativeBlockUpdate { position, state },
        ))?;
        Ok(())
    }
}

fn random_schedule(state: BlockStateId) -> Option<(ScheduledQueueKind, ResourceId, i32)> {
    let schedule = match state {
        ferrite_world::id::FIRE => (
            ScheduledQueueKind::Block,
            ResourceId::minecraft("fire").expect("static identity"),
            i32::from(ferrite_gameplay::environment::fire::FIRE_SCHEDULE_BASE),
        ),
        ferrite_world::id::WATER => fluid_schedule(
            "water",
            ferrite_gameplay::environment::fluid::FluidFamily::Water,
        ),
        ferrite_world::id::LAVA => fluid_schedule(
            "lava",
            ferrite_gameplay::environment::fluid::FluidFamily::Lava,
        ),
        _ => return None,
    };
    Some(schedule)
}

fn fluid_schedule(
    identity: &'static str,
    family: ferrite_gameplay::environment::fluid::FluidFamily,
) -> (ScheduledQueueKind, ResourceId, i32) {
    (
        ScheduledQueueKind::Fluid,
        ResourceId::minecraft(identity).expect("static identity"),
        i32::from(ferrite_gameplay::environment::fluid::fluid_parameters(family, false).tick_delay),
    )
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::{ActivationGeneration, DimensionId, WorldId};
    use ferrite_foundation::region::{
        RegionCoord, RegionMapping, RegionMappingVersion, SimulationRegionKey,
    };
    use ferrite_simulation::tick::GameTick;
    use ferrite_world::chunk::{ChunkLayout, VerticalSectionRange};
    use ferrite_world::generation::status::ChunkStatus;
    use ferrite_world::id::{AIR, BiomeId, WATER};

    use crate::composite::runtime::CompositeRuntimeConfig;
    use crate::composite::services::CompositeProductionRuntimeConfig;
    use crate::entity_service::runtime::EntityServiceRuntimeLimits;
    use crate::simulation::budget::{SimulationQueueBudget, SimulationQueueKind};
    use crate::simulation::runtime::SimulationRuntimeConfig;
    use crate::world_service::model::{ChunkActivity, WorldServiceRuntimeConfig};

    use super::*;

    fn key() -> SimulationRegionKey {
        SimulationRegionKey::new(
            WorldId::new(1).unwrap(),
            DimensionId::new(ResourceId::minecraft("overworld").unwrap()),
            RegionCoord::new(0, 0),
            RegionMappingVersion::V1,
        )
    }

    fn config() -> CompositeProductionRuntimeConfig {
        let budget = SimulationQueueBudget::new([
            (SimulationQueueKind::ScheduledBlocks, 32),
            (SimulationQueueKind::ScheduledFluids, 32),
            (SimulationQueueKind::BoundaryTransactions, 32),
            (SimulationQueueKind::ImmediateNeighbors, 32),
            (SimulationQueueKind::Fluids, 32),
            (SimulationQueueKind::Redstone, 32),
            (SimulationQueueKind::Lighting, 32),
            (SimulationQueueKind::ProjectionPositions, 32),
        ])
        .unwrap();
        let layout = ChunkLayout::new(
            VerticalSectionRange::new(0, 8).unwrap(),
            AIR,
            BiomeId::new(0),
        );
        CompositeProductionRuntimeConfig {
            coordinator: CompositeRuntimeConfig::testing(),
            simulation: SimulationRuntimeConfig {
                mapping: RegionMapping::V1,
                budget,
                projection_capacity: 32,
                receipt_capacity: 32,
                gameplay_random_seed: 9,
            },
            entities: EntityServiceRuntimeLimits::new(8, 8, 8, 8),
            world: WorldServiceRuntimeConfig {
                mapping: RegionMapping::V1,
                layout,
                region_side_chunks: 8,
                chunk_capacity: 8,
                event_capacity: 64,
                content_manifest: [3; 32],
            },
            player_capacity: 8,
            projection_capacity_per_player: 8,
        }
    }

    #[test]
    fn due_fluid_work_mutates_authority_relights_and_projects() {
        let position = ChunkPos::new(0, 0);
        let mut runtime = CompositeProductionRegionRuntime::new(
            key(),
            ActivationGeneration::INITIAL,
            GameTick::ZERO,
            0,
            [position],
            config(),
        )
        .unwrap();
        for status in ChunkStatus::ALL.into_iter().skip(1) {
            let request = runtime.world.begin_generation(position, status).unwrap();
            let mut generated = request.source.clone();
            if status == ChunkStatus::InitializeLight {
                ferrite_world::light::recompute_chunk_light(&mut generated).unwrap();
            }
            runtime
                .world
                .apply_generated(request.complete(generated))
                .unwrap();
        }
        for activity in [ChunkActivity::Accessible, ChunkActivity::BlockTicking] {
            runtime.world.promote(position, activity).unwrap();
        }
        let source = BlockPos::new(4, 20, 4);
        let revision = runtime.world.chunk(position).unwrap().revision();
        runtime
            .world
            .set_block(
                &key(),
                ActivationGeneration::INITIAL,
                revision,
                source,
                WATER,
            )
            .unwrap();
        runtime
            .simulation
            .schedule_local(
                ScheduledQueueKind::Fluid,
                ResourceId::minecraft("water").unwrap(),
                source,
                0,
                TickPriority::Normal,
            )
            .unwrap();

        let report = runtime.run_tick(GameTick::new(1), 1, usize::MAX).unwrap();
        let below = BlockPos::new(4, 19, 4);
        assert_eq!(runtime.world_block(below), Some(WATER));
        assert!(runtime.world.chunk(position).unwrap().light().is_some());
        assert_eq!(report.projections.len(), 1);
    }
}
