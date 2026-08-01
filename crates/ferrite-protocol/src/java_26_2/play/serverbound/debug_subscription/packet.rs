use crate::java_26_2::play::clientbound::debug_projection::packet::DebugSubscription;

pub const MAX_ENCODED_DEBUG_SUBSCRIPTIONS: usize = 32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscriptionRequestKind {
    Replace,
}

impl DebugSubscriptionRequestKind {
    pub const ALL: [Self; 1] = [Self::Replace];

    #[must_use]
    pub const fn wire_id(self) -> i32 {
        23
    }

    #[must_use]
    pub const fn identity(self) -> &'static str {
        "minecraft:debug_subscription_request"
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DebugSubscriptionSet(u16);

impl DebugSubscriptionSet {
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    pub fn from_raw_ids(raw_ids: &[i32]) -> Result<Self, DebugSubscriptionSetError> {
        if raw_ids.len() > MAX_ENCODED_DEBUG_SUBSCRIPTIONS {
            return Err(DebugSubscriptionSetError::TooManyEncoded {
                count: raw_ids.len(),
            });
        }
        let mut subscriptions = Self::empty();
        for &raw_id in raw_ids {
            let Some(subscription) = DebugSubscription::from_raw_id(raw_id) else {
                return Err(DebugSubscriptionSetError::UnknownRawId { raw_id });
            };
            subscriptions.insert(subscription);
        }
        Ok(subscriptions)
    }

    fn insert(&mut self, subscription: DebugSubscription) {
        self.0 |= 1_u16 << subscription.raw_id();
    }

    #[must_use]
    pub const fn contains(self, subscription: DebugSubscription) -> bool {
        self.0 & (1_u16 << subscription.raw_id()) != 0
    }

    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSubscriptionSetError {
    TooManyEncoded { count: usize },
    UnknownRawId { raw_id: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugSubscriptionRequest {
    pub requested: DebugSubscriptionSet,
}
