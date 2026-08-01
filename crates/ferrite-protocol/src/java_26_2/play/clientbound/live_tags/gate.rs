use crate::java_26_2::play::clientbound::live_tags::packet::LiveTagsPacketKind;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveTagsGates {
    pub live_reload: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LiveTagsContext {
    /// True only after a separately owned live-reload service has been registered.
    pub service_registered: bool,
    /// Publication is for a completed resource reload, not an intermediate snapshot.
    pub reload_committed: bool,
    /// Remote clients bind network tags; in-memory clients retain their local bindings.
    pub remote_connection: bool,
    /// Every named registry was prepared successfully before any replacement is applied.
    pub all_registries_prepared: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTagsEffect {
    ReplaceBindingsThenRefreshFuelAndSearchTrees,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveTagsDecision {
    OmitDisabled(LiveTagsPacketKind),
    DegradeServiceUnavailable,
    OmitUncommittedReload,
    OmitInMemoryConnection,
    PreserveExistingBindings,
    Emit(LiveTagsEffect),
}

impl LiveTagsGates {
    #[must_use]
    pub const fn decide(self, context: LiveTagsContext) -> LiveTagsDecision {
        if !self.live_reload {
            return LiveTagsDecision::OmitDisabled(LiveTagsPacketKind::UpdateTags);
        }
        if !context.service_registered {
            return LiveTagsDecision::DegradeServiceUnavailable;
        }
        if !context.reload_committed {
            return LiveTagsDecision::OmitUncommittedReload;
        }
        if !context.remote_connection {
            return LiveTagsDecision::OmitInMemoryConnection;
        }
        if !context.all_registries_prepared {
            return LiveTagsDecision::PreserveExistingBindings;
        }
        LiveTagsDecision::Emit(LiveTagsEffect::ReplaceBindingsThenRefreshFuelAndSearchTrees)
    }
}
