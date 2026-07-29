use bevy_ecs::prelude::Component;
use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::id::BlockStateId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct BlockBreakSession {
    pub position: BlockPos,
    pub expected_state: BlockStateId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakAction {
    Start,
    Abort,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakDecision {
    Track(BlockBreakSession),
    Clear,
    Remove(BlockPos),
    Correct(BlockPos),
}

#[must_use]
pub fn decide_break(
    action: BreakAction,
    active: Option<BlockBreakSession>,
    position: BlockPos,
    current: BlockStateId,
    air: BlockStateId,
) -> BreakDecision {
    match action {
        BreakAction::Start if current == air => BreakDecision::Correct(position),
        BreakAction::Start => BreakDecision::Track(BlockBreakSession {
            position,
            expected_state: current,
        }),
        BreakAction::Abort => BreakDecision::Clear,
        BreakAction::Stop => match active {
            Some(session) if session.position == position && session.expected_state == current => {
                BreakDecision::Remove(position)
            }
            _ => BreakDecision::Correct(position),
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MiningInputs {
    pub hardness: f32,
    pub item_speed: f32,
    pub mining_efficiency: f64,
    pub dig_speed_amplifier: Option<u8>,
    pub mining_fatigue_amplifier: Option<u8>,
    pub block_break_speed: f64,
    pub submerged_mining_speed: f64,
    pub eyes_in_water: bool,
    pub on_ground: bool,
    pub correct_tool: bool,
}

pub fn destroy_progress(inputs: MiningInputs) -> f32 {
    if inputs.hardness == -1.0 {
        return 0.0;
    }
    let mut speed = inputs.item_speed;
    if speed > 1.0 {
        speed += inputs.mining_efficiency as f32;
    }
    if let Some(amplifier) = inputs.dig_speed_amplifier {
        speed *= 1.0 + (f32::from(amplifier) + 1.0) * 0.2;
    }
    if let Some(amplifier) = inputs.mining_fatigue_amplifier {
        speed *= match amplifier {
            0 => 0.3,
            1 => 0.09,
            2 => 0.0027,
            _ => 0.00081,
        };
    }
    speed *= inputs.block_break_speed as f32;
    if inputs.eyes_in_water {
        speed *= inputs.submerged_mining_speed as f32;
    }
    if !inputs.on_ground {
        speed /= 5.0;
    }
    speed / inputs.hardness / if inputs.correct_tool { 30.0 } else { 100.0 }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressRecord {
    pub position: BlockPos,
    pub started_at: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProgressEffect {
    Correct(BlockPos),
    Publish { position: BlockPos, stage: i32 },
    Destroy(BlockPos),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakProgressTracker {
    pub game_ticks: i32,
    pub is_destroying: bool,
    pub destroy_record: Option<ProgressRecord>,
    pub delayed: Option<ProgressRecord>,
    pub last_sent_stage: i32,
}

impl Default for BreakProgressTracker {
    fn default() -> Self {
        Self {
            game_ticks: 0,
            is_destroying: false,
            destroy_record: None,
            delayed: None,
            last_sent_stage: -1,
        }
    }
}

impl BreakProgressTracker {
    pub fn start(
        &mut self,
        position: BlockPos,
        is_air: bool,
        per_tick: f32,
    ) -> Vec<ProgressEffect> {
        if !is_air && per_tick >= 1.0 {
            if let Some(record) = &mut self.destroy_record {
                record.started_at = self.game_ticks;
            }
            return vec![ProgressEffect::Destroy(position)];
        }
        let mut effects = Vec::with_capacity(2);
        if self.is_destroying
            && let Some(record) = self.destroy_record
        {
            effects.push(ProgressEffect::Correct(record.position));
        }
        self.is_destroying = true;
        self.destroy_record = Some(ProgressRecord {
            position,
            started_at: self.game_ticks,
        });
        let stage = if is_air { 10 } else { progress_stage(per_tick) };
        self.last_sent_stage = stage;
        effects.push(ProgressEffect::Publish { position, stage });
        effects
    }

    pub fn stop(&mut self, position: BlockPos, is_air: bool, per_tick: f32) -> Vec<ProgressEffect> {
        let Some(record) = self
            .destroy_record
            .filter(|record| record.position == position)
        else {
            return Vec::new();
        };
        if is_air {
            return Vec::new();
        }
        let progress = per_tick * elapsed(self.game_ticks, record.started_at);
        if progress >= 0.7 {
            self.is_destroying = false;
            return vec![
                ProgressEffect::Publish {
                    position,
                    stage: -1,
                },
                ProgressEffect::Destroy(position),
            ];
        }
        if self.delayed.is_none() {
            self.is_destroying = false;
            self.delayed = Some(record);
        }
        Vec::new()
    }

    pub fn abort(&mut self, packet_position: BlockPos) -> Vec<ProgressEffect> {
        self.is_destroying = false;
        let mut effects = Vec::with_capacity(2);
        if let Some(record) = self
            .destroy_record
            .filter(|record| record.position != packet_position)
        {
            effects.push(ProgressEffect::Publish {
                position: record.position,
                stage: -1,
            });
        }
        effects.push(ProgressEffect::Publish {
            position: packet_position,
            stage: -1,
        });
        effects
    }

    pub fn tick(&mut self, is_air: bool, per_tick: f32) -> Vec<ProgressEffect> {
        self.game_ticks = self.game_ticks.wrapping_add(1);
        if let Some(delayed) = self.delayed {
            if is_air {
                self.delayed = None;
                return Vec::new();
            }
            let progress = per_tick * elapsed(self.game_ticks, delayed.started_at);
            let mut effects = self.publish_changed(delayed.position, progress);
            if progress >= 1.0 {
                self.delayed = None;
                effects.push(ProgressEffect::Destroy(delayed.position));
            }
            return effects;
        }
        if !self.is_destroying {
            return Vec::new();
        }
        let Some(record) = self.destroy_record else {
            return Vec::new();
        };
        if is_air {
            self.is_destroying = false;
            self.last_sent_stage = -1;
            return vec![ProgressEffect::Publish {
                position: record.position,
                stage: -1,
            }];
        }
        let progress = per_tick * elapsed(self.game_ticks, record.started_at);
        self.publish_changed(record.position, progress)
    }

    fn publish_changed(&mut self, position: BlockPos, progress: f32) -> Vec<ProgressEffect> {
        let stage = progress_stage(progress);
        if stage == self.last_sent_stage {
            return Vec::new();
        }
        self.last_sent_stage = stage;
        vec![ProgressEffect::Publish { position, stage }]
    }
}

fn elapsed(current: i32, start: i32) -> f32 {
    current.wrapping_sub(start).wrapping_add(1) as f32
}

fn progress_stage(progress: f32) -> i32 {
    (progress * 10.0) as i32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BreakCommitInputs {
    pub item_allows_destroy: bool,
    pub game_master_allows_destroy: bool,
    pub action_restricted: bool,
    pub removal_succeeded: bool,
    pub prevents_drops: bool,
    pub tool_component_present: bool,
    pub damage_per_block: u32,
    pub destroyed_hardness_nonzero: bool,
    pub shears_on_fire: bool,
    pub correct_tool: bool,
    pub block_drops_enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakCommitEffect {
    ItemDestroyCheck,
    GameMasterCorrection { flags: u32 },
    AdventureCheck,
    PlayerWillDestroy,
    DestroyParticles,
    GuardedPiglinAnger,
    DestroyGameEvent,
    RestoreFluid { flags: u32 },
    BlockDestroyHook,
    CopyTool,
    MineBlock,
    DamageTool(u32),
    AwardItemUsed,
    AwardBlockMined,
    AddExhaustion,
    EvaluateLoot,
    SpawnLoot,
    SpawnAfterBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BreakCommit {
    pub effects: Vec<BreakCommitEffect>,
    pub accepted: bool,
}

pub fn plan_break_commit(inputs: BreakCommitInputs) -> BreakCommit {
    let mut effects = vec![BreakCommitEffect::ItemDestroyCheck];
    if !inputs.item_allows_destroy {
        return BreakCommit {
            effects,
            accepted: false,
        };
    }
    if !inputs.game_master_allows_destroy {
        effects.push(BreakCommitEffect::GameMasterCorrection { flags: 3 });
        return BreakCommit {
            effects,
            accepted: false,
        };
    }
    effects.push(BreakCommitEffect::AdventureCheck);
    if inputs.action_restricted {
        return BreakCommit {
            effects,
            accepted: false,
        };
    }

    effects.extend([
        BreakCommitEffect::PlayerWillDestroy,
        BreakCommitEffect::DestroyParticles,
        BreakCommitEffect::GuardedPiglinAnger,
        BreakCommitEffect::DestroyGameEvent,
        BreakCommitEffect::RestoreFluid { flags: 3 },
    ]);
    if inputs.removal_succeeded {
        effects.push(BreakCommitEffect::BlockDestroyHook);
    }
    if inputs.prevents_drops {
        return BreakCommit {
            effects,
            accepted: true,
        };
    }

    effects.extend([BreakCommitEffect::CopyTool, BreakCommitEffect::MineBlock]);
    let mine_succeeded = inputs.tool_component_present;
    if mine_succeeded
        && inputs.destroyed_hardness_nonzero
        && inputs.damage_per_block > 0
        && !inputs.shears_on_fire
    {
        effects.push(BreakCommitEffect::DamageTool(inputs.damage_per_block));
    }
    if mine_succeeded {
        effects.push(BreakCommitEffect::AwardItemUsed);
    }
    if inputs.removal_succeeded && inputs.correct_tool {
        effects.extend([
            BreakCommitEffect::AwardBlockMined,
            BreakCommitEffect::AddExhaustion,
            BreakCommitEffect::EvaluateLoot,
        ]);
        if inputs.block_drops_enabled {
            effects.push(BreakCommitEffect::SpawnLoot);
        }
        effects.push(BreakCommitEffect::SpawnAfterBreak);
    }
    BreakCommit {
        effects,
        accepted: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_requires_the_same_position_and_state_seen_at_start() {
        let position = BlockPos::new(1, 2, 3);
        let stone = BlockStateId::new(1);
        let session = BlockBreakSession {
            position,
            expected_state: stone,
        };
        assert_eq!(
            decide_break(
                BreakAction::Stop,
                Some(session),
                position,
                stone,
                BlockStateId::new(0)
            ),
            BreakDecision::Remove(position)
        );
        assert_eq!(
            decide_break(
                BreakAction::Stop,
                Some(session),
                position,
                BlockStateId::new(2),
                BlockStateId::new(0)
            ),
            BreakDecision::Correct(position)
        );
    }
}
