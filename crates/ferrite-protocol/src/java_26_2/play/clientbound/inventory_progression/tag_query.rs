use crate::java_26_2::play::clientbound::inventory_progression::packet::TagQuery;
use crate::java_26_2::value::nbt::NetworkNbt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugQueryHandler {
    counter: i32,
    pending: bool,
}

impl Default for DebugQueryHandler {
    fn default() -> Self {
        Self {
            counter: -1,
            pending: false,
        }
    }
}

impl DebugQueryHandler {
    pub fn start_transaction(&mut self) -> i32 {
        self.counter = self.counter.wrapping_add(1);
        self.pending = true;
        self.counter
    }

    pub fn handle_response<E>(
        &mut self,
        packet: &TagQuery,
        callback: impl FnOnce(Option<&NetworkNbt>) -> Result<(), E>,
    ) -> Result<bool, E> {
        if !self.pending || packet.transaction != self.counter {
            return Ok(false);
        }
        callback(packet.tag.as_ref())?;
        self.pending = false;
        Ok(true)
    }

    #[must_use]
    pub const fn current_transaction(&self) -> i32 {
        self.counter
    }

    #[must_use]
    pub const fn has_pending_callback(&self) -> bool {
        self.pending
    }
}

#[must_use]
pub fn block_query_response(
    permitted: bool,
    transaction: i32,
    tag: Option<NetworkNbt>,
) -> Option<TagQuery> {
    permitted.then_some(TagQuery { transaction, tag })
}

#[must_use]
pub fn entity_query_response(
    permitted: bool,
    entity_exists: bool,
    transaction: i32,
    tag: NetworkNbt,
) -> Option<TagQuery> {
    (permitted && entity_exists).then_some(TagQuery {
        transaction,
        tag: Some(tag),
    })
}
