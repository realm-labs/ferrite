//! Server-side effect audience and level-event projection.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectPlayer {
    pub id: u64,
    pub level: u64,
    pub position: [f64; 3],
    pub block_position: [i32; 3],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExceptSource {
    Player(u64),
    Other,
}

#[must_use]
pub fn sound_range(volume: f32, fixed_range: Option<f32>) -> f64 {
    f64::from(fixed_range.unwrap_or_else(|| 16.0 * volume.max(1.0)))
}

#[must_use]
pub fn position_sound_recipients(
    players: &[EffectPlayer],
    source_level: u64,
    source: [f64; 3],
    range: f64,
    except: Option<ExceptSource>,
) -> Vec<u64> {
    players
        .iter()
        .filter(|player| {
            player.level == source_level
                && excluded_player(except) != Some(player.id)
                && distance_squared(player.position, source) < range * range
        })
        .map(|player| player.id)
        .collect()
}

#[must_use]
pub fn ordinary_level_event_recipients(
    players: &[EffectPlayer],
    source_level: u64,
    source: [i32; 3],
    except: Option<ExceptSource>,
) -> Vec<u64> {
    position_sound_recipients(players, source_level, source.map(f64::from), 64.0, except)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedLevelEvent {
    pub player_id: u64,
    pub position: [i32; 3],
    pub global: bool,
}

#[must_use]
pub fn level_event_recipients(
    players: &[EffectPlayer],
    source_level: u64,
    source: [i32; 3],
    except: Option<ExceptSource>,
    global_sound_events: bool,
) -> Vec<ProjectedLevelEvent> {
    if !global_sound_events {
        return ordinary_level_event_recipients(players, source_level, source, except)
            .into_iter()
            .map(|player_id| ProjectedLevelEvent {
                player_id,
                position: source,
                global: false,
            })
            .collect();
    }
    let source_center = source.map(|coordinate| f64::from(coordinate) + 0.5);
    players
        .iter()
        .map(|player| {
            let position = if player.level != source_level {
                player.block_position
            } else if distance_squared(player.position, source_center) < 32.0 * 32.0 {
                source_center.map(floor_to_i32)
            } else {
                projected_global_position(player.position, source_center)
            };
            ProjectedLevelEvent {
                player_id: player.id,
                position,
                global: true,
            }
        })
        .collect()
}

#[must_use]
pub fn particle_recipients(
    players: &[EffectPlayer],
    source_level: u64,
    source: [f64; 3],
    override_limiter: bool,
) -> Vec<u64> {
    let range = if override_limiter { 512.0 } else { 32.0 };
    players
        .iter()
        .filter(|player| {
            let integer_position = player.block_position.map(f64::from);
            player.level == source_level
                && distance_squared(integer_position, source) < range * range
        })
        .map(|player| player.id)
        .collect()
}

#[must_use]
pub fn tracking_and_self_recipients(
    tracking_players: &[u64],
    self_player: Option<u64>,
) -> Vec<u64> {
    let mut recipients = tracking_players.to_vec();
    if let Some(player) = self_player.filter(|player| !recipients.contains(player)) {
        recipients.push(player);
    }
    recipients
}

fn excluded_player(except: Option<ExceptSource>) -> Option<u64> {
    match except {
        Some(ExceptSource::Player(player)) => Some(player),
        Some(ExceptSource::Other) | None => None,
    }
}

fn projected_global_position(player: [f64; 3], source: [f64; 3]) -> [i32; 3] {
    let direction = [
        source[0] - player[0],
        source[1] - player[1],
        source[2] - player[2],
    ];
    let length = distance_squared(source, player).sqrt();
    if length == 0.0 {
        return player.map(floor_to_i32);
    }
    [
        floor_to_i32(player[0] + direction[0] / length * 32.0),
        floor_to_i32(player[1] + direction[1] / length * 32.0),
        floor_to_i32(player[2] + direction[2] / length * 32.0),
    ]
}

fn distance_squared(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum()
}

fn floor_to_i32(value: f64) -> i32 {
    value.floor() as i32
}
