use ferrite_foundation::coordinate::BlockPos;
use ferrite_foundation::direction::Direction;

use crate::player::breaking::prediction::ClientPredictionClock;
use crate::player::breaking::{
    BreakingItem, ClientBreakEffect, ClientBreakOutcome, PlayerAction, TargetState,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StartBreakContext {
    pub position: BlockPos,
    pub face: Direction,
    pub item: BreakingItem,
    pub target: TargetState,
    pub action_restricted: bool,
    pub inside_world_border: bool,
    pub instabuild: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContinueBreakContext {
    pub start: StartBreakContext,
    pub selected_slot_changed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClientBreakSession {
    pub destroy_position: BlockPos,
    pub destroying_item: Option<BreakingItem>,
    pub destroy_progress: f32,
    pub destroy_ticks: f32,
    pub destroy_delay: i32,
    pub is_destroying: bool,
    prediction: ClientPredictionClock,
}

impl Default for ClientBreakSession {
    fn default() -> Self {
        Self {
            destroy_position: BlockPos::default(),
            destroying_item: None,
            destroy_progress: 0.0,
            destroy_ticks: 0.0,
            destroy_delay: 0,
            is_destroying: false,
            prediction: ClientPredictionClock::default(),
        }
    }
}

impl ClientBreakSession {
    pub fn start(&mut self, context: StartBreakContext) -> ClientBreakOutcome {
        if context.action_restricted || !context.inside_world_border {
            return ClientBreakOutcome::rejected();
        }
        if context.instabuild {
            let mut effects = vec![ClientBreakEffect::TutorialProgress {
                position: context.position,
                progress: 1.0,
            }];
            self.predict_destroy(
                context.position,
                context.face,
                PlayerAction::StartDestroyBlock,
                &mut effects,
            );
            self.destroy_delay = 5;
            return outcome(effects);
        }
        if self.same_target(context.position, context.item) {
            return outcome(Vec::new());
        }

        let mut effects = Vec::new();
        if self.is_destroying {
            effects.push(ClientBreakEffect::SendAction {
                action: PlayerAction::AbortDestroyBlock,
                position: self.destroy_position,
                face: context.face,
                sequence: 0,
            });
        }
        effects.push(ClientBreakEffect::TutorialProgress {
            position: context.position,
            progress: 0.0,
        });
        let sequence = self.prediction.begin();
        effects.push(ClientBreakEffect::BeginPrediction(sequence));
        if !context.target.is_air && self.destroy_progress == 0.0 {
            effects.push(ClientBreakEffect::AttackBlock(context.position));
        }
        if !context.target.is_air && context.target.destroy_progress >= 1.0 {
            effects.push(ClientBreakEffect::AttemptLocalDestroy(context.position));
        } else {
            self.is_destroying = true;
            self.destroy_position = context.position;
            self.destroying_item = Some(context.item);
            self.destroy_progress = 0.0;
            self.destroy_ticks = 0.0;
            effects.push(ClientBreakEffect::PublishCrack {
                position: context.position,
                stage: -1,
            });
        }
        effects.push(ClientBreakEffect::SendAction {
            action: PlayerAction::StartDestroyBlock,
            position: context.position,
            face: context.face,
            sequence,
        });
        self.prediction.end();
        effects.push(ClientBreakEffect::EndPrediction(sequence));
        outcome(effects)
    }

    pub fn continue_break(&mut self, context: ContinueBreakContext) -> ClientBreakOutcome {
        let mut prefix = Vec::new();
        if context.selected_slot_changed {
            prefix.push(ClientBreakEffect::SendCarriedSlot);
        }
        if self.destroy_delay > 0 {
            self.destroy_delay -= 1;
            return outcome(prefix);
        }
        if context.start.instabuild && context.start.inside_world_border {
            self.destroy_delay = 5;
            prefix.push(ClientBreakEffect::TutorialProgress {
                position: context.start.position,
                progress: 1.0,
            });
            self.predict_destroy(
                context.start.position,
                context.start.face,
                PlayerAction::StartDestroyBlock,
                &mut prefix,
            );
            return outcome(prefix);
        }
        if !self.same_target(context.start.position, context.start.item) {
            let mut started = self.start(context.start);
            prefix.append(&mut started.effects);
            return ClientBreakOutcome {
                continued: started.continued,
                effects: prefix,
            };
        }
        if context.start.target.is_air {
            self.is_destroying = false;
            return ClientBreakOutcome {
                continued: false,
                effects: prefix,
            };
        }

        self.destroy_progress += context.start.target.destroy_progress;
        if self.destroy_ticks % 4.0 == 0.0 {
            prefix.push(ClientBreakEffect::PlayHitSound {
                position: context.start.position,
                volume: (context.start.target.sound_volume + 1.0) / 8.0,
                pitch: context.start.target.sound_pitch * 0.5,
            });
        }
        self.destroy_ticks += 1.0;
        prefix.push(ClientBreakEffect::TutorialProgress {
            position: context.start.position,
            progress: java_clamp(self.destroy_progress, 0.0, 1.0),
        });
        if self.destroy_progress < 1.0 {
            prefix.push(ClientBreakEffect::PublishCrack {
                position: context.start.position,
                stage: java_float_to_int(self.destroy_progress * 10.0),
            });
            return outcome(prefix);
        }

        self.is_destroying = false;
        self.predict_destroy(
            context.start.position,
            context.start.face,
            PlayerAction::StopDestroyBlock,
            &mut prefix,
        );
        self.destroy_progress = 0.0;
        self.destroy_ticks = 0.0;
        self.destroy_delay = 5;
        prefix.push(ClientBreakEffect::PublishCrack {
            position: context.start.position,
            stage: -1,
        });
        outcome(prefix)
    }

    pub fn stop(&mut self) -> ClientBreakOutcome {
        if !self.is_destroying {
            return ClientBreakOutcome::rejected();
        }
        let position = self.destroy_position;
        self.is_destroying = false;
        self.destroy_progress = 0.0;
        outcome(vec![
            ClientBreakEffect::TutorialProgress {
                position,
                progress: -1.0,
            },
            ClientBreakEffect::SendAction {
                action: PlayerAction::AbortDestroyBlock,
                position,
                face: Direction::Down,
                sequence: 0,
            },
            ClientBreakEffect::PublishCrack {
                position,
                stage: -1,
            },
            ClientBreakEffect::ResetAttackStrength,
        ])
    }

    #[must_use]
    pub const fn prediction(&self) -> ClientPredictionClock {
        self.prediction
    }

    fn same_target(&self, position: BlockPos, item: BreakingItem) -> bool {
        self.is_destroying
            && self.destroy_position == position
            && self
                .destroying_item
                .is_some_and(|retained| retained.same_item_and_components(item))
    }

    fn predict_destroy(
        &mut self,
        position: BlockPos,
        face: Direction,
        action: PlayerAction,
        effects: &mut Vec<ClientBreakEffect>,
    ) {
        let sequence = self.prediction.begin();
        effects.extend([
            ClientBreakEffect::BeginPrediction(sequence),
            ClientBreakEffect::AttemptLocalDestroy(position),
            ClientBreakEffect::SendAction {
                action,
                position,
                face,
                sequence,
            },
        ]);
        self.prediction.end();
        effects.push(ClientBreakEffect::EndPrediction(sequence));
    }
}

fn outcome(effects: Vec<ClientBreakEffect>) -> ClientBreakOutcome {
    ClientBreakOutcome {
        continued: true,
        effects,
    }
}

fn java_clamp(value: f32, minimum: f32, maximum: f32) -> f32 {
    if value < minimum {
        minimum
    } else if value > maximum {
        maximum
    } else {
        value
    }
}

fn java_float_to_int(value: f32) -> i32 {
    value as i32
}
