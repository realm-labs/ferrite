//! Spectator player-distance admission and independent client chunk projection.

use std::collections::{BTreeMap, BTreeSet};

pub type PlayerId = u64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    #[must_use]
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SectionPos {
    pub chunk: ChunkPos,
    pub section_y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PlayerAdmission {
    section: SectionPos,
    spectator: bool,
    ignored: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionAction {
    PlayerMapAdded {
        player: PlayerId,
        ignored: bool,
    },
    PlayerMapRemoved {
        player: PlayerId,
    },
    ProjectEntities {
        player: PlayerId,
    },
    AddDistanceSource {
        player: PlayerId,
        chunk: ChunkPos,
        first_at_chunk: bool,
    },
    RemoveDistanceSource {
        player: PlayerId,
        chunk: ChunkPos,
        last_at_chunk: bool,
    },
    SetIgnored {
        player: PlayerId,
        ignored: bool,
    },
    ResetClientView {
        player: PlayerId,
    },
    RefreshClientView {
        player: PlayerId,
        chunk: ChunkPos,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SpectatorAdmissions {
    players: BTreeMap<PlayerId, PlayerAdmission>,
    admitted_by_chunk: BTreeMap<ChunkPos, BTreeSet<PlayerId>>,
}

impl SpectatorAdmissions {
    pub fn add_player(
        &mut self,
        player: PlayerId,
        section: SectionPos,
        spectator: bool,
        spectators_generate_chunks: bool,
    ) -> Vec<AdmissionAction> {
        let ignored = skip_player(spectator, spectators_generate_chunks);
        self.players.insert(
            player,
            PlayerAdmission {
                section,
                spectator,
                ignored,
            },
        );
        let mut actions = vec![AdmissionAction::PlayerMapAdded { player, ignored }];
        if !ignored {
            actions.push(self.add_distance(player, section.chunk));
        }
        actions.push(AdmissionAction::ResetClientView { player });
        actions.push(AdmissionAction::RefreshClientView {
            player,
            chunk: section.chunk,
        });
        actions
    }

    pub fn move_player(
        &mut self,
        player: PlayerId,
        section: SectionPos,
        spectator: bool,
        spectators_generate_chunks: bool,
        removed: bool,
    ) -> Vec<AdmissionAction> {
        if removed {
            return Vec::new();
        }
        let mut actions = vec![AdmissionAction::ProjectEntities { player }];
        let Some(previous) = self.players.get(&player).copied() else {
            return actions;
        };
        let ignored = skip_player(spectator, spectators_generate_chunks);
        if previous.section == section && previous.ignored == ignored {
            if let Some(entry) = self.players.get_mut(&player) {
                entry.spectator = spectator;
            }
            return actions;
        }
        if !previous.ignored {
            actions.push(self.remove_distance(player, previous.section.chunk));
        }
        if !ignored {
            actions.push(self.add_distance(player, section.chunk));
        }
        if previous.ignored != ignored {
            actions.push(AdmissionAction::SetIgnored { player, ignored });
        }
        self.players.insert(
            player,
            PlayerAdmission {
                section,
                spectator,
                ignored,
            },
        );
        actions.push(AdmissionAction::RefreshClientView {
            player,
            chunk: section.chunk,
        });
        actions
    }

    pub fn remove_player(&mut self, player: PlayerId) -> Vec<AdmissionAction> {
        let Some(previous) = self.players.remove(&player) else {
            return Vec::new();
        };
        let mut actions = vec![AdmissionAction::PlayerMapRemoved { player }];
        if !previous.ignored {
            actions.push(self.remove_distance(player, previous.section.chunk));
        }
        actions.push(AdmissionAction::ResetClientView { player });
        actions
    }

    #[must_use]
    pub fn admitted_count(&self, chunk: ChunkPos) -> usize {
        self.admitted_by_chunk.get(&chunk).map_or(0, BTreeSet::len)
    }

    #[must_use]
    pub const fn simulation_ticket_level(simulation_distance: u8) -> u8 {
        31_u8.saturating_sub(simulation_distance)
    }

    #[must_use]
    pub fn natural_spawn_cap(base_maximum: u32, spawnable_chunk_count: usize) -> u32 {
        let count = u32::try_from(spawnable_chunk_count).unwrap_or(u32::MAX);
        base_maximum.saturating_mul(count) / 289
    }

    fn add_distance(&mut self, player: PlayerId, chunk: ChunkPos) -> AdmissionAction {
        let occupants = self.admitted_by_chunk.entry(chunk).or_default();
        let first_at_chunk = occupants.is_empty();
        occupants.insert(player);
        AdmissionAction::AddDistanceSource {
            player,
            chunk,
            first_at_chunk,
        }
    }

    fn remove_distance(&mut self, player: PlayerId, chunk: ChunkPos) -> AdmissionAction {
        let last_at_chunk = if let Some(occupants) = self.admitted_by_chunk.get_mut(&chunk) {
            occupants.remove(&player);
            occupants.is_empty()
        } else {
            false
        };
        if last_at_chunk {
            self.admitted_by_chunk.remove(&chunk);
        }
        AdmissionAction::RemoveDistanceSource {
            player,
            chunk,
            last_at_chunk,
        }
    }
}

#[must_use]
pub const fn skip_player(spectator: bool, spectators_generate_chunks: bool) -> bool {
    spectator && !spectators_generate_chunks
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkProjectionAction {
    SetCenter(ChunkPos),
    Pending(ChunkPos),
    Forget(ChunkPos),
    BatchStart,
    SendFullChunk(ChunkPos),
    BatchFinish,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ChunkProjection {
    center: Option<ChunkPos>,
    radius: u8,
    pending: BTreeSet<ChunkPos>,
    sent: BTreeSet<ChunkPos>,
}

impl ChunkProjection {
    pub fn update_view(
        &mut self,
        center: ChunkPos,
        requested_view_distance: u8,
        server_view_distance: u8,
        alive: bool,
        mut is_ready: impl FnMut(ChunkPos) -> bool,
    ) -> Vec<ChunkProjectionAction> {
        let radius = requested_view_distance.clamp(2, server_view_distance.max(2));
        let old_view = self
            .center
            .map_or_else(BTreeSet::new, |old| view_positions(old, self.radius));
        let new_view = view_positions(center, radius);
        let mut actions = Vec::new();
        if self.center != Some(center) {
            actions.push(ChunkProjectionAction::SetCenter(center));
        }
        for position in old_view.difference(&new_view).copied() {
            if !self.pending.remove(&position) && self.sent.remove(&position) && alive {
                actions.push(ChunkProjectionAction::Forget(position));
            }
        }
        for position in new_view.difference(&old_view).copied() {
            if is_ready(position) && self.pending.insert(position) {
                actions.push(ChunkProjectionAction::Pending(position));
            }
        }
        self.center = Some(center);
        self.radius = radius;
        actions
    }

    #[must_use]
    pub fn on_chunk_ready(&mut self, position: ChunkPos) -> Option<ChunkProjectionAction> {
        if self.contains(position)
            && !self.sent.contains(&position)
            && self.pending.insert(position)
        {
            Some(ChunkProjectionAction::Pending(position))
        } else {
            None
        }
    }

    pub fn send_pending(&mut self, quota: usize) -> Vec<ChunkProjectionAction> {
        if quota == 0 || self.pending.is_empty() {
            return Vec::new();
        }
        let selected: Vec<_> = self.pending.iter().copied().take(quota).collect();
        let mut actions = Vec::with_capacity(selected.len() + 2);
        actions.push(ChunkProjectionAction::BatchStart);
        for position in selected {
            self.pending.remove(&position);
            self.sent.insert(position);
            actions.push(ChunkProjectionAction::SendFullChunk(position));
        }
        actions.push(ChunkProjectionAction::BatchFinish);
        actions
    }

    #[must_use]
    pub fn contains(&self, position: ChunkPos) -> bool {
        self.center.is_some_and(|center| {
            center.x.abs_diff(position.x) <= u32::from(self.radius)
                && center.z.abs_diff(position.z) <= u32::from(self.radius)
        })
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

fn view_positions(center: ChunkPos, radius: u8) -> BTreeSet<ChunkPos> {
    let radius = i32::from(radius);
    let mut positions = BTreeSet::new();
    for x in center.x.saturating_sub(radius)..=center.x.saturating_add(radius) {
        for z in center.z.saturating_sub(radius)..=center.z.saturating_add(radius) {
            positions.insert(ChunkPos::new(x, z));
        }
    }
    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn false_rule_spectator_keeps_view_but_not_distance_source() {
        let mut admissions = SpectatorAdmissions::default();
        let section = SectionPos {
            chunk: ChunkPos::new(4, 5),
            section_y: 3,
        };
        let actions = admissions.add_player(1, section, true, false);
        assert!(matches!(
            actions.as_slice(),
            [
                AdmissionAction::PlayerMapAdded { ignored: true, .. },
                AdmissionAction::ResetClientView { .. },
                AdmissionAction::RefreshClientView { .. }
            ]
        ));
        assert_eq!(admissions.admitted_count(section.chunk), 0);
    }

    #[test]
    fn reconciliation_removes_old_then_adds_new_and_tracks_shared_last_source() {
        let mut admissions = SpectatorAdmissions::default();
        let first = SectionPos {
            chunk: ChunkPos::new(0, 0),
            section_y: 0,
        };
        admissions.add_player(1, first, false, true);
        admissions.add_player(2, first, false, true);
        let actions = admissions.move_player(
            1,
            SectionPos {
                chunk: ChunkPos::new(1, 0),
                section_y: 0,
            },
            false,
            true,
            false,
        );
        assert!(matches!(
            actions[1],
            AdmissionAction::RemoveDistanceSource {
                last_at_chunk: false,
                ..
            }
        ));
        assert!(matches!(
            actions[2],
            AdmissionAction::AddDistanceSource {
                first_at_chunk: true,
                ..
            }
        ));
        let removal = admissions.remove_player(2);
        assert!(matches!(
            removal[1],
            AdmissionAction::RemoveDistanceSource {
                last_at_chunk: true,
                ..
            }
        ));
    }

    #[test]
    fn ignored_projection_receives_externally_ready_chunks_and_forgets_sent_exclusions() {
        let mut projection = ChunkProjection::default();
        let center = ChunkPos::new(0, 0);
        let actions = projection.update_view(center, 2, 10, true, |_| false);
        assert_eq!(actions, vec![ChunkProjectionAction::SetCenter(center)]);
        let ready = ChunkPos::new(1, 1);
        assert_eq!(
            projection.on_chunk_ready(ready),
            Some(ChunkProjectionAction::Pending(ready))
        );
        assert!(matches!(
            projection.send_pending(1).as_slice(),
            [
                ChunkProjectionAction::BatchStart,
                ChunkProjectionAction::SendFullChunk(position),
                ChunkProjectionAction::BatchFinish
            ] if *position == ready
        ));
        let actions = projection.update_view(ChunkPos::new(10, 10), 2, 10, true, |_| false);
        assert!(actions.contains(&ChunkProjectionAction::Forget(ready)));
    }
}
