use crate::java_26_2::play::serverbound::debug_subscription::packet::DebugSubscriptionSet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugSubscriptionAuthorization {
    pub operator: bool,
    pub ide_singleplayer_owner: bool,
}

impl DebugSubscriptionAuthorization {
    #[must_use]
    pub const fn admitted(self) -> bool {
        self.operator || self.ide_singleplayer_owner
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSynchronizerTransition {
    Unchanged,
    SleepAndClear,
    WakeAndSeed,
    ReplaceMembership,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSubscriptionRebuild {
    pub effective: DebugSubscriptionSet,
    pub transition: DebugSynchronizerTransition,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugSubscriptionRuntime {
    requested: DebugSubscriptionSet,
    effective: DebugSubscriptionSet,
}

impl DebugSubscriptionRuntime {
    pub fn replace_requested(&mut self, requested: DebugSubscriptionSet) {
        self.requested = requested;
    }

    #[must_use]
    pub const fn requested(&self) -> DebugSubscriptionSet {
        self.requested
    }

    #[must_use]
    pub const fn effective(&self) -> DebugSubscriptionSet {
        self.effective
    }

    pub fn rebuild_effective(
        &mut self,
        authorization: DebugSubscriptionAuthorization,
    ) -> DebugSubscriptionRebuild {
        let next = if authorization.admitted() {
            self.requested
        } else {
            DebugSubscriptionSet::empty()
        };
        let transition = match (self.effective.is_empty(), next.is_empty()) {
            (true, false) => DebugSynchronizerTransition::WakeAndSeed,
            (false, true) => DebugSynchronizerTransition::SleepAndClear,
            _ if self.effective != next => DebugSynchronizerTransition::ReplaceMembership,
            _ => DebugSynchronizerTransition::Unchanged,
        };
        self.effective = next;
        DebugSubscriptionRebuild {
            effective: next,
            transition,
        }
    }

    fn clear_for_player_removal(&mut self) {
        self.requested = DebugSubscriptionSet::empty();
        self.effective = DebugSubscriptionSet::empty();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscriptionLifecycleEvent {
    Disconnect,
    ReconfigurationRemoval,
}

#[must_use]
pub fn apply_lifecycle(
    runtime: &mut DebugSubscriptionRuntime,
    _event: DebugSubscriptionLifecycleEvent,
) -> DebugSynchronizerTransition {
    runtime.clear_for_player_removal();
    DebugSynchronizerTransition::SleepAndClear
}
