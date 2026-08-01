use std::collections::{BTreeMap, BTreeSet};

use ferrite_foundation::coordinate::{BlockPos, ChunkPos, LocalBlockPos};
use ferrite_foundation::identity::DimensionId;
use ferrite_foundation::region::RegionMapping;
use ferrite_foundation::resource::ResourceId;
use ferrite_gameplay::player::state::{PlayerPose, Rotation as PlayerRotation, Vec3 as PlayerVec3};
use ferrite_world::generation::border::state::WorldBorder;
use ferrite_world::generation::dimension::{DimensionType, LockedDimension, Position};
use ferrite_world::generation::portal::HorizontalAxis;
use ferrite_world::generation::portal::end_portal::{
    EndPortalDesiredBlock, SavedRespawn, enter_end, entering_end_platform, leave_end,
};
use ferrite_world::generation::portal::nether::{
    NetherExitInput, PortalBlock, PortalBorder, PortalCreationWorld, PortalPoi, create_portal,
    largest_matching_rectangle, nether_exit, scaled_search_block, select_portal_poi,
};
use ferrite_world::generation::portal::processor::{
    PortalContactState, PortalTickResult, PortalWaitInput, entity_portal_cooldown,
    nether_portal_wait,
};
use ferrite_world::id::{
    AIR, BlockStateId, END_PORTAL, FIRE, NETHER_PORTAL_X, NETHER_PORTAL_Z, OBSIDIAN,
    has_empty_collision,
};
use ferrite_world::projection::{ChunkSnapshot, ClientHeightmap};
use thiserror::Error;

use crate::chunk::ticket::{ACCESSIBLE_LEVEL, ChunkTicket, TicketLevel, TicketSource};
use crate::composite::gateway::CompositeRegionRouter;
use crate::player::connection::JavaPlayerConnection;
use crate::world_service::runtime::WorldBlockWrite;
use ferrite_simulation::tick::GameTick;

const OVERWORLD: &str = "minecraft:overworld";
const NETHER: &str = "minecraft:the_nether";
const END: &str = "minecraft:the_end";
const PORTAL_TICKET_CHUNK_RADIUS: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormalPortalKind {
    Nether(HorizontalAxis),
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PortalJourney {
    source_dimension: DimensionId,
    destination_dimension: DimensionId,
    source_entry: BlockPos,
    source_pose: PlayerPose,
    target: BlockPos,
    kind: FormalPortalKind,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PortalResolution {
    pub(super) destination_dimension: DimensionId,
    pub(super) pose: PlayerPose,
    pub(super) writes: Vec<WorldBlockWrite>,
    pub(super) player_level_event: Option<u16>,
}

#[derive(Debug, Default)]
pub(super) struct PortalSessionState {
    contact: PortalContactState,
    journey: Option<PortalJourney>,
    transition_pending: bool,
    level_event: Option<u16>,
}

impl PortalSessionState {
    pub(super) fn observe_contact(
        &mut self,
        player: Option<&JavaPlayerConnection>,
        snapshots: &BTreeMap<DimensionId, BTreeMap<ChunkPos, ChunkSnapshot>>,
        dimensions: &[DimensionId],
        borders: &BTreeMap<DimensionId, WorldBorder>,
        respawn: BlockPos,
    ) -> Result<(), PortalDynError> {
        let Some(player) = player else {
            return Ok(());
        };
        if self.journey.is_some() || self.transition_pending {
            self.contact.tick(false, 0);
            return Ok(());
        }
        let dimension = player.player().region().dimension();
        let pose = player.player().committed_state().pose();
        let contact = snapshots
            .get(dimension)
            .and_then(|snapshots| portal_contact(snapshots, pose));
        let wait = match contact {
            Some((FormalPortalKind::Nether(_), _)) => nether_portal_wait(PortalWaitInput {
                is_player: true,
                invulnerable_ability: false,
                creative_delay: 0,
                default_delay: 80,
            }),
            Some((FormalPortalKind::End, _)) | None => 0,
        };
        if let Some((kind, entry)) = contact {
            self.contact.contact(
                portal_object(kind),
                entry,
                entity_portal_cooldown(true, false),
            );
        }
        let PortalTickResult::Ready {
            portal_object,
            entry_block,
        } = self.contact.tick(true, wait)
        else {
            return Ok(());
        };
        let journey = PortalJourney::begin(
            dimension,
            entry_block,
            pose,
            portal_kind(portal_object)?,
            dimensions,
            borders,
            respawn,
        )?;
        self.journey = self
            .contact
            .attempt_ready(entity_portal_cooldown(true, false), || journey);
        Ok(())
    }

    pub(super) fn tickets(
        &self,
        player: Option<&JavaPlayerConnection>,
        router: &CompositeRegionRouter,
    ) -> Vec<(DimensionId, ChunkTicket)> {
        let (Some(player), Some(journey)) = (player, &self.journey) else {
            return Vec::new();
        };
        journey
            .ticket_chunks()
            .into_iter()
            .filter(|position| {
                let region = RegionMapping::V1.region_for_chunk(
                    player.player().region().world(),
                    journey.destination_dimension().clone(),
                    *position,
                );
                router.activation_generation(&region).is_some()
            })
            .map(|position| {
                (
                    journey.destination_dimension().clone(),
                    ChunkTicket {
                        source: TicketSource::Portal(portal_ticket_source()),
                        position,
                        level: TicketLevel::new(ACCESSIBLE_LEVEL),
                        expires_at: None,
                    },
                )
            })
            .collect()
    }

    pub(super) fn stage_ready(
        &mut self,
        player: Option<&mut JavaPlayerConnection>,
        tick: GameTick,
        snapshots: &BTreeMap<DimensionId, BTreeMap<ChunkPos, ChunkSnapshot>>,
        borders: &BTreeMap<DimensionId, WorldBorder>,
        respawn: BlockPos,
        router: &mut CompositeRegionRouter,
    ) -> Result<(), PortalDynError> {
        let (Some(player), Some(journey)) = (player, self.journey.as_ref()) else {
            return Ok(());
        };
        let source = snapshots
            .get(player.player().region().dimension())
            .ok_or("portal source dimension has no projectable chunks")?;
        let destination = snapshots
            .get(journey.destination_dimension())
            .ok_or("portal destination dimension has no projectable chunks")?;
        let border = borders
            .get(journey.destination_dimension())
            .ok_or("portal destination dimension has no border")?;
        let Some(resolution) = journey.resolve(source, destination, border, respawn)? else {
            return Ok(());
        };
        let world = player.player().region().world();
        let mut writes_by_region = BTreeMap::new();
        for write in resolution.writes {
            let region = RegionMapping::V1.region_for_chunk(
                world,
                resolution.destination_dimension.clone(),
                write.position.chunk(),
            );
            if router.activation_generation(&region).is_none()
                || router
                    .world_block_state(&resolution.destination_dimension, write.position)?
                    .is_none()
            {
                return Ok(());
            }
            writes_by_region
                .entry(region)
                .or_insert_with(Vec::new)
                .push(write);
        }
        let target_region = RegionMapping::V1.region_for_chunk(
            world,
            resolution.destination_dimension.clone(),
            BlockPos::new(
                resolution.pose.position.x.floor() as i32,
                resolution.pose.position.y.floor() as i32,
                resolution.pose.position.z.floor() as i32,
            )
            .chunk(),
        );
        if router.activation_generation(&target_region).is_none() {
            return Ok(());
        }
        player.stage_dimension_transfer(
            resolution.destination_dimension,
            resolution.pose,
            tick,
            router,
        )?;
        for (region, writes) in writes_by_region {
            router.admit_world_blocks(&region, tick, writes)?;
        }
        self.level_event = resolution.player_level_event;
        self.transition_pending = true;
        self.journey = None;
        Ok(())
    }

    pub(super) fn complete_transition(&mut self) -> Option<u16> {
        self.transition_pending = false;
        self.level_event.take()
    }
}

const fn portal_object(kind: FormalPortalKind) -> u32 {
    match kind {
        FormalPortalKind::Nether(HorizontalAxis::X) => 1,
        FormalPortalKind::Nether(HorizontalAxis::Z) => 2,
        FormalPortalKind::End => 3,
    }
}

fn portal_kind(value: u32) -> Result<FormalPortalKind, PortalIntegrationError> {
    match value {
        1 => Ok(FormalPortalKind::Nether(HorizontalAxis::X)),
        2 => Ok(FormalPortalKind::Nether(HorizontalAxis::Z)),
        3 => Ok(FormalPortalKind::End),
        _ => Err(PortalIntegrationError::UnknownPortalObject(value)),
    }
}

type PortalDynError = Box<dyn std::error::Error + Send + Sync>;

impl PortalJourney {
    pub(super) fn begin(
        source_dimension: &DimensionId,
        source_entry: BlockPos,
        source_pose: PlayerPose,
        kind: FormalPortalKind,
        enabled_dimensions: &[DimensionId],
        borders: &BTreeMap<DimensionId, WorldBorder>,
        respawn: BlockPos,
    ) -> Result<Option<Self>, PortalIntegrationError> {
        let destination = match (source_dimension.to_string().as_str(), kind) {
            (OVERWORLD, FormalPortalKind::Nether(_)) => NETHER,
            (NETHER, FormalPortalKind::Nether(_)) => OVERWORLD,
            (OVERWORLD, FormalPortalKind::End) => END,
            (END, FormalPortalKind::End) => OVERWORLD,
            _ => return Ok(None),
        };
        let destination_dimension = enabled_dimensions
            .iter()
            .find(|dimension| dimension.to_string() == destination)
            .cloned();
        let Some(destination_dimension) = destination_dimension else {
            return Ok(None);
        };
        let destination_border = borders
            .get(&destination_dimension)
            .ok_or_else(|| PortalIntegrationError::MissingBorder(destination_dimension.clone()))?;
        let target = match kind {
            FormalPortalKind::Nether(_) => scaled_search_block(
                Position {
                    x: source_pose.position.x,
                    y: source_pose.position.y,
                    z: source_pose.position.z,
                },
                &dimension_type(source_dimension)?,
                &dimension_type(&destination_dimension)?,
                portal_border(destination_border),
            ),
            FormalPortalKind::End if destination == END => BlockPos::new(100, 49, 0),
            FormalPortalKind::End => respawn,
        };
        Ok(Some(Self {
            source_dimension: source_dimension.clone(),
            destination_dimension,
            source_entry,
            source_pose,
            target,
            kind,
        }))
    }

    pub(super) fn destination_dimension(&self) -> &DimensionId {
        &self.destination_dimension
    }

    pub(super) fn ticket_chunks(&self) -> BTreeSet<ChunkPos> {
        let center = self.target.chunk();
        (-PORTAL_TICKET_CHUNK_RADIUS..=PORTAL_TICKET_CHUNK_RADIUS)
            .flat_map(|x| {
                (-PORTAL_TICKET_CHUNK_RADIUS..=PORTAL_TICKET_CHUNK_RADIUS).map(move |z| {
                    ChunkPos::new(center.x.saturating_add(x), center.z.saturating_add(z))
                })
            })
            .collect()
    }

    pub(super) fn resolve(
        &self,
        source: &BTreeMap<ChunkPos, ChunkSnapshot>,
        destination: &BTreeMap<ChunkPos, ChunkSnapshot>,
        destination_border: &WorldBorder,
        respawn: BlockPos,
    ) -> Result<Option<PortalResolution>, PortalIntegrationError> {
        if !destination.contains_key(&self.target.chunk()) {
            return Ok(None);
        }
        match self.kind {
            FormalPortalKind::Nether(axis) => {
                self.resolve_nether(axis, source, destination, destination_border)
            }
            FormalPortalKind::End => self.resolve_end(destination, respawn),
        }
    }

    fn resolve_nether(
        &self,
        source_axis: HorizontalAxis,
        source: &BTreeMap<ChunkPos, ChunkSnapshot>,
        destination: &BTreeMap<ChunkPos, ChunkSnapshot>,
        destination_border: &WorldBorder,
    ) -> Result<Option<PortalResolution>, PortalIntegrationError> {
        let source_view = PortalSnapshotView::new(source, portal_border(destination_border));
        let source_rectangle =
            largest_matching_rectangle(self.source_entry, source_axis, |position| {
                source_view.block(position) == Some(portal_state(source_axis))
            });
        let (source_axis, relative) =
            ferrite_world::generation::portal::nether::relative_entry_position(
                Some(source_rectangle),
                portal_vec(self.source_pose.position),
                0.6,
                1.8,
            );
        let destination_view =
            PortalSnapshotView::new(destination, portal_border(destination_border));
        let mut points = destination_view.portal_points(self.target);
        points.sort_by_key(|point| {
            (
                point.position.chunk(),
                point.position.y,
                point.position.x,
                point.position.z,
            )
        });
        for (order, point) in points.iter_mut().enumerate() {
            point.encounter_order = order as u64;
        }
        let destination_key = self.destination_dimension.to_string();
        let selected = select_portal_poi(
            self.target,
            &destination_key,
            destination_view.border,
            points,
        );
        let (rectangle, writes, existing_poi) = if let Some(poi) = selected {
            let axis = poi.axis.expect("portal POI selection requires an axis");
            (
                largest_matching_rectangle(poi.position, axis, |position| {
                    destination_view.block(position) == Some(portal_state(axis))
                }),
                Vec::new(),
                Some(poi.position),
            )
        } else {
            let Some(creation) = create_portal(
                &destination_view,
                self.target,
                source_axis,
                destination_view.minimum_y(),
                destination_view.maximum_y(),
                destination_view.logical_height(),
            ) else {
                return Ok(None);
            };
            let writes = creation
                .writes
                .into_iter()
                .map(|write| WorldBlockWrite {
                    position: write.position,
                    state: match write.block {
                        PortalBlock::Obsidian => OBSIDIAN,
                        PortalBlock::Air => AIR,
                        PortalBlock::Portal(axis) => portal_state(axis),
                    },
                })
                .collect::<Vec<_>>();
            if writes
                .iter()
                .any(|write| !destination.contains_key(&write.position.chunk()))
            {
                return Ok(None);
            }
            (creation.rectangle, writes, None)
        };
        let exit = nether_exit(
            NetherExitInput {
                destination: rectangle,
                source_axis,
                relative,
                entity_size: [0.6, 1.8],
                velocity: ferrite_world::generation::portal::Vec3::ZERO,
                rotation: ferrite_world::generation::portal::Rotation {
                    yaw: self.source_pose.rotation.yaw,
                    pitch: self.source_pose.rotation.pitch,
                },
                is_server_player: true,
                existing_poi,
            },
            |position, _| Some(position),
        );
        Ok(Some(PortalResolution {
            destination_dimension: self.destination_dimension.clone(),
            pose: PlayerPose::new(
                player_vec(exit.position),
                PlayerRotation {
                    yaw: exit.rotation.yaw,
                    pitch: exit.rotation.pitch,
                },
            ),
            writes,
            player_level_event: exit.player_level_event,
        }))
    }

    fn resolve_end(
        &self,
        destination: &BTreeMap<ChunkPos, ChunkSnapshot>,
        respawn: BlockPos,
    ) -> Result<Option<PortalResolution>, PortalIntegrationError> {
        let entering = self.destination_dimension.to_string() == END;
        let transition = if entering {
            enter_end(
                true,
                true,
                ferrite_world::generation::portal::Vec3::ZERO,
                self.source_pose.rotation.pitch,
            )
        } else {
            leave_end(
                true,
                OVERWORLD,
                SavedRespawn {
                    position: respawn,
                    yaw: self.source_pose.rotation.yaw,
                    pitch: self.source_pose.rotation.pitch,
                },
                true,
                ferrite_world::generation::portal::Vec3::ZERO,
            )
        };
        let Some(transition) = transition else {
            return Ok(None);
        };
        let writes = if transition.build_platform {
            let writes = entering_end_platform()
                .into_iter()
                .map(|write| WorldBlockWrite {
                    position: write.position,
                    state: match write.desired {
                        EndPortalDesiredBlock::Obsidian => OBSIDIAN,
                        EndPortalDesiredBlock::Air => AIR,
                    },
                })
                .collect::<Vec<_>>();
            if writes
                .iter()
                .any(|write| !destination.contains_key(&write.position.chunk()))
            {
                return Ok(None);
            }
            writes
        } else {
            Vec::new()
        };
        Ok(Some(PortalResolution {
            destination_dimension: self.destination_dimension.clone(),
            pose: PlayerPose::new(
                player_vec(transition.position),
                PlayerRotation {
                    yaw: transition.rotation.yaw,
                    pitch: transition.rotation.pitch,
                },
            ),
            writes,
            player_level_event: transition.player_level_event,
        }))
    }
}

pub(super) fn portal_contact(
    snapshots: &BTreeMap<ChunkPos, ChunkSnapshot>,
    pose: PlayerPose,
) -> Option<(FormalPortalKind, BlockPos)> {
    let feet = portal_vec(pose.position).containing();
    [
        feet,
        BlockPos::new(feet.x, feet.y.saturating_add(1), feet.z),
    ]
    .into_iter()
    .find_map(|position| {
        let state = snapshot_block(snapshots, position)?;
        let kind = if state == NETHER_PORTAL_X {
            FormalPortalKind::Nether(HorizontalAxis::X)
        } else if state == NETHER_PORTAL_Z {
            FormalPortalKind::Nether(HorizontalAxis::Z)
        } else if state == END_PORTAL {
            FormalPortalKind::End
        } else {
            return None;
        };
        Some((kind, position))
    })
}

fn dimension_type(dimension: &DimensionId) -> Result<DimensionType, PortalIntegrationError> {
    match dimension.to_string().as_str() {
        OVERWORLD => Ok(DimensionType::locked(LockedDimension::Overworld)),
        NETHER => Ok(DimensionType::locked(LockedDimension::TheNether)),
        END => Ok(DimensionType::locked(LockedDimension::TheEnd)),
        _ => Err(PortalIntegrationError::UnsupportedDimension(
            dimension.clone(),
        )),
    }
}

fn portal_border(border: &WorldBorder) -> PortalBorder {
    let half = border.extent.size() * 0.5;
    let absolute = f64::from(border.absolute_max);
    PortalBorder {
        minimum_x: (border.center_x - half).clamp(-absolute, absolute),
        maximum_x: (border.center_x + half).clamp(-absolute, absolute),
        minimum_z: (border.center_z - half).clamp(-absolute, absolute),
        maximum_z: (border.center_z + half).clamp(-absolute, absolute),
    }
}

const fn portal_state(axis: HorizontalAxis) -> BlockStateId {
    match axis {
        HorizontalAxis::X => NETHER_PORTAL_X,
        HorizontalAxis::Z => NETHER_PORTAL_Z,
    }
}

fn snapshot_block(
    snapshots: &BTreeMap<ChunkPos, ChunkSnapshot>,
    position: BlockPos,
) -> Option<BlockStateId> {
    let snapshot = snapshots.get(&position.chunk())?;
    let section_y = position.y.div_euclid(16);
    let sections = snapshot.layout().sections();
    if !sections.contains(section_y) {
        return None;
    }
    let index = usize::try_from(section_y - sections.minimum()).ok()?;
    snapshot
        .sections()
        .get(index)
        .map(|section| section.block(position.local()))
}

struct PortalSnapshotView<'a> {
    snapshots: &'a BTreeMap<ChunkPos, ChunkSnapshot>,
    border: PortalBorder,
}

impl<'a> PortalSnapshotView<'a> {
    const fn new(snapshots: &'a BTreeMap<ChunkPos, ChunkSnapshot>, border: PortalBorder) -> Self {
        Self { snapshots, border }
    }

    fn block(&self, position: BlockPos) -> Option<BlockStateId> {
        snapshot_block(self.snapshots, position)
    }

    fn minimum_y(&self) -> i32 {
        self.snapshots
            .values()
            .next()
            .map_or(0, |snapshot| snapshot.layout().sections().minimum() * 16)
    }

    fn maximum_y(&self) -> i32 {
        self.snapshots.values().next().map_or(255, |snapshot| {
            snapshot.layout().sections().maximum_exclusive() * 16 - 1
        })
    }

    fn logical_height(&self) -> u32 {
        u32::try_from(
            self.maximum_y()
                .saturating_sub(self.minimum_y())
                .saturating_add(1),
        )
        .unwrap_or(u32::MAX)
    }

    fn portal_points(&self, target: BlockPos) -> Vec<PortalPoi> {
        let mut points = Vec::new();
        for snapshot in self.snapshots.values() {
            let layout = snapshot.layout();
            for (section_index, section) in snapshot.sections().iter().enumerate() {
                let section_y = layout.sections().minimum() + section_index as i32;
                for local_y in 0..16_u8 {
                    for local_z in 0..16_u8 {
                        for local_x in 0..16_u8 {
                            let local = LocalBlockPos::new(local_x, local_y, local_z)
                                .expect("bounded portal scan uses local coordinates");
                            let state = section.block(local);
                            let axis = if state == NETHER_PORTAL_X {
                                Some(HorizontalAxis::X)
                            } else if state == NETHER_PORTAL_Z {
                                Some(HorizontalAxis::Z)
                            } else {
                                None
                            };
                            let Some(axis) = axis else {
                                continue;
                            };
                            let position = BlockPos::new(
                                snapshot.position().x * 16 + i32::from(local_x),
                                section_y * 16 + i32::from(local_y),
                                snapshot.position().z * 16 + i32::from(local_z),
                            );
                            if (position.x - target.x).abs() <= 128
                                && (position.z - target.z).abs() <= 128
                            {
                                points.push(PortalPoi {
                                    position,
                                    axis: Some(axis),
                                    encounter_order: 0,
                                });
                            }
                        }
                    }
                }
            }
        }
        points
    }
}

impl PortalCreationWorld for PortalSnapshotView<'_> {
    fn border(&self) -> PortalBorder {
        self.border
    }

    fn motion_blocking_height(&self, x: i32, z: i32) -> i32 {
        let Some(snapshot) = self.snapshots.get(&BlockPos::new(x, 0, z).chunk()) else {
            return self.minimum_y();
        };
        let local_x = x.rem_euclid(16) as usize;
        let local_z = z.rem_euclid(16) as usize;
        snapshot
            .heightmaps()
            .get(&ClientHeightmap::MotionBlocking)
            .map_or(self.minimum_y(), |heights| heights[local_z * 16 + local_x])
    }

    fn is_dry_replaceable(&self, position: BlockPos) -> bool {
        self.block(position)
            .is_some_and(|state| state == AIR || state == FIRE)
    }

    fn is_solid(&self, position: BlockPos) -> bool {
        self.block(position)
            .is_some_and(|state| !has_empty_collision(state))
    }
}

const fn portal_vec(value: PlayerVec3) -> ferrite_world::generation::portal::Vec3 {
    ferrite_world::generation::portal::Vec3 {
        x: value.x,
        y: value.y,
        z: value.z,
    }
}

const fn player_vec(value: ferrite_world::generation::portal::Vec3) -> PlayerVec3 {
    PlayerVec3::new(value.x, value.y, value.z)
}

#[derive(Debug, Error)]
pub(super) enum PortalIntegrationError {
    #[error("portal integration does not support dimension {0}")]
    UnsupportedDimension(DimensionId),
    #[error("portal integration has no authoritative border for dimension {0}")]
    MissingBorder(DimensionId),
    #[error("formal portal object {0} is unknown")]
    UnknownPortalObject(u32),
    #[error(transparent)]
    Resource(#[from] ferrite_foundation::resource::ResourceIdError),
}

pub(super) fn portal_ticket_source() -> ResourceId {
    ResourceId::new("ferrite", "portal").expect("static portal ticket identity is valid")
}

#[cfg(test)]
mod tests {
    use ferrite_foundation::identity::StableEntityId;
    use ferrite_foundation::region::RegionMapping;
    use ferrite_foundation::resource::ResourceId;
    use ferrite_gameplay::player::state::{Rotation, Vec3};
    use ferrite_protocol::semantic::{
        ChatVisibility, ClientSettings, MainHand, ParticleStatus, PlayAdmission, PlayerSpawn,
        SessionId, SessionIdentity, VirtualHost,
    };
    use ferrite_world::chunk::{ChunkColumn, ChunkLayout, VerticalSectionRange};
    use ferrite_world::id::{BiomeId, STONE};
    use ferrite_world::light::recompute_chunk_light;
    use tempfile::TempDir;

    use super::*;
    use crate::chunk::session::ChunkSessionLimits;
    use crate::config::ServerConfig;
    use crate::minecraft::{settings, world};
    use crate::player::session::PlayerSessionAction;
    use crate::session::command::SessionJoinPayload;
    use crate::session::router::RegionCommandRouter;

    fn dimension(path: &str) -> DimensionId {
        DimensionId::new(ResourceId::minecraft(path).unwrap())
    }

    fn layout(minimum: i32, count: u16) -> ChunkLayout {
        ChunkLayout::new(
            VerticalSectionRange::new(minimum, count).unwrap(),
            AIR,
            BiomeId::new(0),
        )
    }

    fn snapshot(
        position: ChunkPos,
        layout: ChunkLayout,
        floor: Option<i32>,
        writes: &[(BlockPos, BlockStateId)],
    ) -> ChunkSnapshot {
        let mut chunk = ChunkColumn::new(position, layout);
        if let Some(y) = floor {
            for x in position.x * 16..position.x * 16 + 16 {
                for z in position.z * 16..position.z * 16 + 16 {
                    chunk.set_block(BlockPos::new(x, y, z), STONE).unwrap();
                }
            }
        }
        for (position, state) in writes {
            if position.chunk() == chunk.position() {
                chunk.set_block(*position, *state).unwrap();
            }
        }
        recompute_chunk_light(&mut chunk).unwrap();
        let light = chunk
            .light()
            .unwrap()
            .snapshot(layout.sections().count())
            .unwrap();
        chunk
            .snapshot(light, |_, state| !has_empty_collision(state))
            .unwrap()
    }

    fn square_snapshots(
        center: ChunkPos,
        radius: i32,
        layout: ChunkLayout,
        floor: Option<i32>,
    ) -> BTreeMap<ChunkPos, ChunkSnapshot> {
        (-radius..=radius)
            .flat_map(|x| {
                (-radius..=radius).map(move |z| {
                    let position = ChunkPos::new(center.x + x, center.z + z);
                    (position, snapshot(position, layout, floor, &[]))
                })
            })
            .collect()
    }

    fn client_settings() -> ClientSettings {
        ClientSettings {
            language: "en_us".to_owned(),
            view_distance: 2,
            chat_visibility: ChatVisibility::Full,
            chat_colors: true,
            model_customization: u8::MAX,
            main_hand: MainHand::Right,
            text_filtering: false,
            allows_listing: true,
            particle_status: ParticleStatus::All,
        }
    }

    fn projectable_snapshots(
        router: &CompositeRegionRouter,
        dimensions: &[DimensionId],
    ) -> BTreeMap<DimensionId, BTreeMap<ChunkPos, ChunkSnapshot>> {
        dimensions
            .iter()
            .map(|dimension| {
                let positions = router.projectable_world_positions(dimension).unwrap();
                let snapshots = router
                    .projectable_world_snapshots(dimension, positions)
                    .unwrap();
                (dimension.clone(), snapshots)
            })
            .collect()
    }

    #[test]
    fn contact_recognizes_both_nether_axes_and_the_end_surface() {
        let layout = layout(-4, 24);
        let position = ChunkPos::new(0, 0);
        for (state, expected) in [
            (NETHER_PORTAL_X, FormalPortalKind::Nether(HorizontalAxis::X)),
            (NETHER_PORTAL_Z, FormalPortalKind::Nether(HorizontalAxis::Z)),
            (END_PORTAL, FormalPortalKind::End),
        ] {
            let block = BlockPos::new(2, 70, 3);
            let snapshots = BTreeMap::from([(
                position,
                snapshot(position, layout, None, &[(block, state)]),
            )]);
            let contact = portal_contact(
                &snapshots,
                PlayerPose::new(PlayerVec3::new(2.5, 70.0, 3.5), PlayerRotation::default()),
            );
            assert_eq!(contact, Some((expected, block)));
        }
    }

    #[test]
    fn nether_journey_scales_coordinates_and_creates_a_safe_durable_exit() {
        let overworld = dimension("overworld");
        let nether = dimension("the_nether");
        let source_entry = BlockPos::new(80, 70, 0);
        let source_layout = layout(-4, 24);
        let source = BTreeMap::from([(
            source_entry.chunk(),
            snapshot(
                source_entry.chunk(),
                source_layout,
                Some(69),
                &[
                    (source_entry, NETHER_PORTAL_X),
                    (BlockPos::new(81, 70, 0), NETHER_PORTAL_X),
                    (BlockPos::new(80, 71, 0), NETHER_PORTAL_X),
                    (BlockPos::new(81, 71, 0), NETHER_PORTAL_X),
                    (BlockPos::new(80, 72, 0), NETHER_PORTAL_X),
                    (BlockPos::new(81, 72, 0), NETHER_PORTAL_X),
                ],
            ),
        )]);
        let destination = square_snapshots(ChunkPos::new(0, 0), 1, layout(0, 16), Some(69));
        let border = WorldBorder::default();
        let borders = BTreeMap::from([
            (overworld.clone(), border.clone()),
            (nether.clone(), border.clone()),
        ]);
        let journey = PortalJourney::begin(
            &overworld,
            source_entry,
            PlayerPose::new(PlayerVec3::new(80.5, 70.0, 0.5), PlayerRotation::default()),
            FormalPortalKind::Nether(HorizontalAxis::X),
            &[overworld.clone(), nether.clone()],
            &borders,
            BlockPos::new(0, 70, 0),
        )
        .unwrap()
        .unwrap();
        assert_eq!(journey.target, BlockPos::new(10, 70, 0));
        let resolution = journey
            .resolve(&source, &destination, &border, BlockPos::new(0, 70, 0))
            .unwrap()
            .unwrap();
        assert_eq!(resolution.destination_dimension, nether);
        assert!(
            resolution
                .writes
                .iter()
                .any(|write| write.state == OBSIDIAN)
        );
        assert!(
            resolution
                .writes
                .iter()
                .any(|write| write.state == NETHER_PORTAL_X)
        );
        assert!(resolution.pose.position.y >= 70.0);
    }

    #[test]
    fn entering_end_uses_the_audited_platform_and_player_target() {
        let overworld = dimension("overworld");
        let end = dimension("the_end");
        let border = WorldBorder::default();
        let borders = BTreeMap::from([
            (overworld.clone(), border.clone()),
            (end.clone(), border.clone()),
        ]);
        let journey = PortalJourney::begin(
            &overworld,
            BlockPos::new(0, 70, 0),
            PlayerPose::new(PlayerVec3::new(0.5, 70.0, 0.5), PlayerRotation::default()),
            FormalPortalKind::End,
            &[overworld.clone(), end.clone()],
            &borders,
            BlockPos::new(0, 70, 0),
        )
        .unwrap()
        .unwrap();
        let destination = square_snapshots(ChunkPos::new(6, 0), 1, layout(0, 16), None);
        let resolution = journey
            .resolve(
                &BTreeMap::new(),
                &destination,
                &border,
                BlockPos::new(0, 70, 0),
            )
            .unwrap()
            .unwrap();
        assert_eq!(resolution.destination_dimension, end);
        assert_eq!(resolution.writes.len(), 100);
        assert_eq!(resolution.pose.position, PlayerVec3::new(100.5, 49.0, 0.5));
        assert_eq!(resolution.player_level_event, Some(1032));
    }

    #[test]
    fn formal_portal_contact_generates_a_durable_exit_and_commits_dimension_transfer() {
        let temporary = TempDir::new().unwrap();
        let mut config = ServerConfig::development_node(1, 1, 30_000, temporary.path()).unwrap();
        config.world.view_distance = 2;
        config.world.simulation_distance = 2;
        config.world.dimensions = vec![OVERWORLD.to_owned(), NETHER.to_owned(), END.to_owned()];
        let validated = ServerConfig::from_toml(&config.to_toml().unwrap()).unwrap();
        let mut runtime = world::load(&validated).unwrap();
        let route = runtime
            .routes
            .resolve(&VirtualHost {
                host: "localhost".to_owned(),
                port: 25_565,
            })
            .clone();
        let source = runtime.respawn;
        let source_region =
            route
                .mapping
                .region_for_chunk(route.world, route.dimension.clone(), source.chunk());
        let admission = PlayAdmission {
            session: SessionId::new(1).unwrap(),
            identity: SessionIdentity {
                profile_id: 1,
                name: "PortalWalker".to_owned(),
            },
            player: StableEntityId::new(1).unwrap(),
            region: source_region.clone(),
            region_mapping: RegionMapping::V1,
            spawn_chunk: source.chunk(),
            spawn: PlayerSpawn {
                x: f64::from(source.x) + 0.5,
                y: f64::from(source.y),
                z: f64::from(source.z) + 0.5,
                yaw: 0.0,
                pitch: 0.0,
            },
            requested_view_distance: 2,
            transferred: false,
        };
        let mut tick = runtime.committed_tick.checked_next().unwrap();
        runtime
            .router
            .admit_world_blocks(
                &source_region,
                tick,
                vec![WorldBlockWrite {
                    position: source,
                    state: NETHER_PORTAL_X,
                }],
            )
            .unwrap();
        RegionCommandRouter::route(
            &mut runtime.router,
            SessionJoinPayload {
                session: admission.session,
                player: admission.player,
                identity: admission.identity.clone(),
                settings: client_settings(),
                transferred: false,
                spawn_pose: PlayerPose::new(
                    Vec3::new(admission.spawn.x, admission.spawn.y, admission.spawn.z),
                    Rotation::default(),
                ),
            }
            .into_region_command(source_region, tick, 0)
            .unwrap(),
        )
        .unwrap();
        runtime.router.run_tick(tick).unwrap();

        let protocol = settings::load(None, &runtime.dimensions).unwrap();
        let mut player = JavaPlayerConnection::new(
            admission,
            protocol.registries,
            2,
            2,
            ChunkSessionLimits {
                maximum_tracked_chunks: 25,
                maximum_tickets: 26,
                maximum_chunks_per_batch: 4,
            },
        )
        .unwrap();
        let borders = runtime
            .dimensions
            .iter()
            .map(|dimension| {
                (
                    dimension.clone(),
                    runtime.lifecycle.level(dimension).unwrap().border.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut portal = PortalSessionState::default();
        let source_snapshots = projectable_snapshots(&runtime.router, &runtime.dimensions);
        for _ in 0..=80 {
            portal
                .observe_contact(
                    Some(&player),
                    &source_snapshots,
                    &runtime.dimensions,
                    &borders,
                    runtime.respawn,
                )
                .unwrap();
        }
        let tickets = portal.tickets(Some(&player), &runtime.router);
        assert!(!tickets.is_empty());

        let nether = dimension("the_nether");
        let nether_tickets = tickets
            .into_iter()
            .filter_map(|(dimension, ticket)| (dimension == nether).then_some(ticket))
            .collect::<Vec<_>>();
        let mut committed = false;
        for _ in 0..32 {
            tick = tick.checked_next().unwrap();
            runtime
                .chunk_lifecycles
                .get_mut(&nether)
                .unwrap()
                .drive(tick, nether_tickets.clone(), &mut runtime.router)
                .unwrap();
            let snapshots = projectable_snapshots(&runtime.router, &runtime.dimensions);
            portal
                .stage_ready(
                    Some(&mut player),
                    tick,
                    &snapshots,
                    &borders,
                    runtime.respawn,
                    &mut runtime.router,
                )
                .unwrap();
            let report = runtime.router.run_tick(tick).unwrap();
            if player.player().transfer_pending() {
                let update = player.observe_committed_tick(report.local()).unwrap();
                committed = update.player == PlayerSessionAction::DimensionTransferCommitted;
            }
            if committed {
                break;
            }
        }
        assert!(committed);
        assert_eq!(player.player().region().dimension(), &nether);
        let destination = projectable_snapshots(&runtime.router, &runtime.dimensions);
        assert!(destination[&nether].values().any(|snapshot| {
            snapshot
                .sections()
                .iter()
                .any(|section| (0..4_096).any(|index| section.blocks().get(index) == Ok(OBSIDIAN)))
        }));
    }
}
