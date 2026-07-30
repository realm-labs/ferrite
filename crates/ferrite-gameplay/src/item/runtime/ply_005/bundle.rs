//! Exact bundle capacity, ordered contents, click overrides, and held-use cadence.

use crate::item::runtime::stack::ItemStack;

pub const BUNDLE_CAPACITY: Fraction = Fraction::ONE;
pub const NESTED_BUNDLE_OVERHEAD: Fraction = Fraction::new_unchecked(1, 16);
pub const BUNDLE_USE_DURATION: u32 = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fraction {
    numerator: u64,
    denominator: u64,
}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let left = u128::from(self.numerator) * u128::from(other.denominator);
        let right = u128::from(other.numerator) * u128::from(self.denominator);
        left.cmp(&right)
    }
}

impl Fraction {
    pub const ZERO: Self = Self::new_unchecked(0, 1);
    pub const ONE: Self = Self::new_unchecked(1, 1);

    pub const fn new_unchecked(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    pub fn new(numerator: u64, denominator: u64) -> Option<Self> {
        (denominator != 0).then(|| Self::reduced(numerator, denominator))
    }

    pub const fn numerator(self) -> u64 {
        self.numerator
    }

    pub const fn denominator(self) -> u64 {
        self.denominator
    }

    pub fn checked_add(self, other: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .checked_mul(other.denominator)?
            .checked_add(other.numerator.checked_mul(self.denominator)?)?;
        let denominator = self.denominator.checked_mul(other.denominator)?;
        Some(Self::reduced(numerator, denominator))
    }

    pub fn checked_mul(self, multiplier: u64) -> Option<Self> {
        Some(Self::reduced(
            self.numerator.checked_mul(multiplier)?,
            self.denominator,
        ))
    }

    pub fn checked_sub(self, other: Self) -> Option<Self> {
        let left = self.numerator.checked_mul(other.denominator)?;
        let right = other.numerator.checked_mul(self.denominator)?;
        let numerator = left.checked_sub(right)?;
        let denominator = self.denominator.checked_mul(other.denominator)?;
        Some(Self::reduced(numerator, denominator))
    }

    pub fn floor_div(self, divisor: Self) -> Option<u64> {
        if divisor.numerator == 0 {
            return None;
        }
        self.numerator
            .checked_mul(divisor.denominator)?
            .checked_div(self.denominator.checked_mul(divisor.numerator)?)
    }

    fn reduced(numerator: u64, denominator: u64) -> Self {
        let divisor = gcd(numerator, denominator);
        Self {
            numerator: numerator / divisor,
            denominator: denominator / divisor,
        }
    }
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleEntry {
    pub stack: ItemStack,
    pub nonempty_bees: bool,
    pub nested_weight: Option<Fraction>,
}

impl BundleEntry {
    pub fn ordinary(stack: ItemStack) -> Self {
        Self {
            stack,
            nonempty_bees: false,
            nested_weight: None,
        }
    }

    pub fn unit_weight(&self) -> Option<Fraction> {
        if self.nonempty_bees {
            Some(Fraction::ONE)
        } else if let Some(weight) = self.nested_weight {
            weight.checked_add(NESTED_BUNDLE_OVERHEAD)
        } else {
            let maximum = u64::try_from(self.stack.maximum).ok()?;
            Fraction::new(1, maximum)
        }
    }

    pub fn weight(&self) -> Option<Fraction> {
        let count = u64::try_from(self.stack.count).ok()?;
        self.unit_weight()?.checked_mul(count)
    }

    fn compatible_with(&self, other: &Self) -> bool {
        self.stack.compatible_with(&other.stack)
            && self.nonempty_bees == other.nonempty_bees
            && self.nested_weight == other.nested_weight
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleContents {
    entries: Vec<BundleEntry>,
    selected: i32,
}

impl Default for BundleContents {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            selected: -1,
        }
    }
}

impl BundleContents {
    pub fn from_persisted(entries: Vec<BundleEntry>) -> Self {
        let mut contents = Self {
            entries,
            selected: -1,
        };
        if contents.total_weight().is_none() {
            contents.entries.clear();
        }
        contents
    }

    pub fn entries(&self) -> &[BundleEntry] {
        &self.entries
    }

    pub const fn selected(&self) -> i32 {
        self.selected
    }

    pub fn persisted_eq(&self, other: &Self) -> bool {
        self.entries == other.entries
    }

    pub fn weight(&self) -> Option<Fraction> {
        self.total_weight()
    }

    fn total_weight(&self) -> Option<Fraction> {
        self.entries
            .iter()
            .try_fold(Fraction::ZERO, |weight, entry| {
                weight.checked_add(entry.weight()?)
            })
    }

    pub fn capacity_for(&self, entry: &BundleEntry, insertable: bool) -> u64 {
        if !insertable || entry.stack.is_empty() {
            return 0;
        }
        let Some(weight) = self.weight() else {
            return 0;
        };
        BUNDLE_CAPACITY
            .checked_sub(weight)
            .and_then(|remaining| remaining.floor_div(entry.unit_weight()?))
            .unwrap_or(0)
    }

    pub fn insert(
        &mut self,
        source: &mut BundleEntry,
        insertable: bool,
        split_identity: u64,
    ) -> u64 {
        let admitted = self
            .capacity_for(source, insertable)
            .min(u64::try_from(source.stack.count).unwrap_or(0));
        let Ok(admitted_i32) = i32::try_from(admitted) else {
            return 0;
        };
        if admitted_i32 == 0 {
            return 0;
        }
        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.compatible_with(source))
        {
            let mut entry = self.entries.remove(index);
            entry.stack.grow(admitted_i32);
            source.stack.shrink(admitted_i32);
            self.entries.insert(0, entry);
        } else {
            let mut inserted = source.clone();
            inserted.stack = source.stack.split(admitted_i32, split_identity);
            self.entries.insert(0, inserted);
        }
        admitted
    }

    pub fn remove(&mut self) -> Option<BundleEntry> {
        if self.entries.is_empty() {
            self.selected = -1;
            return None;
        }
        let index = usize::try_from(self.selected)
            .ok()
            .filter(|index| *index < self.entries.len())
            .unwrap_or(0);
        self.selected = -1;
        Some(self.entries.remove(index))
    }

    pub fn put_back(&mut self, entry: BundleEntry) {
        self.entries.insert(0, entry);
    }

    pub fn set_selected(&mut self, selected: i32) -> Result<(), SelectionDecodeError> {
        if selected < -1 {
            return Err(SelectionDecodeError { selected });
        }
        self.selected = usize::try_from(selected)
            .ok()
            .filter(|index| *index < self.entries.len())
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(-1);
        Ok(())
    }

    pub fn clear_selection(&mut self) {
        self.selected = -1;
    }

    pub fn destroy(&mut self) -> Vec<BundleEntry> {
        self.selected = -1;
        std::mem::take(&mut self.entries)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionDecodeError {
    pub selected: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickAction {
    Primary,
    Secondary,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleSound {
    Insert,
    InsertFail,
    RemoveOne,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClickOutcome {
    pub handled: bool,
    pub transferred: u64,
    pub sound: Option<BundleSound>,
    pub rewrite_and_broadcast: bool,
}

pub fn bundle_on_cursor_insert(
    contents: &mut BundleContents,
    action: ClickAction,
    source: &mut BundleEntry,
    insertable: bool,
    split_identity: u64,
) -> ClickOutcome {
    if action != ClickAction::Primary || source.stack.is_empty() {
        return unhandled();
    }
    let transferred = contents.insert(source, insertable, split_identity);
    ClickOutcome {
        handled: true,
        transferred,
        sound: Some(if transferred == 0 {
            BundleSound::InsertFail
        } else {
            BundleSound::Insert
        }),
        rewrite_and_broadcast: true,
    }
}

pub fn bundle_on_cursor_remove(
    contents: &mut BundleContents,
    action: ClickAction,
    target_capacity: u64,
) -> (ClickOutcome, Option<BundleEntry>) {
    if action != ClickAction::Secondary || target_capacity == 0 {
        return (unhandled(), None);
    }
    let Some(mut removed) = contents.remove() else {
        return (
            ClickOutcome {
                handled: true,
                transferred: 0,
                sound: None,
                rewrite_and_broadcast: true,
            },
            None,
        );
    };
    let count = u64::try_from(removed.stack.count).unwrap_or(0);
    let transferred = count.min(target_capacity);
    let complete = transferred == count;
    if !complete {
        let remainder = count - transferred;
        removed.stack.count = i32::try_from(remainder).unwrap_or(i32::MAX);
        contents.put_back(removed);
    }
    (
        ClickOutcome {
            handled: true,
            transferred,
            sound: complete.then_some(BundleSound::RemoveOne),
            rewrite_and_broadcast: true,
        },
        None,
    )
}

pub fn bundle_in_slot_insert(
    contents: &mut BundleContents,
    action: ClickAction,
    cursor: &mut BundleEntry,
    allow_modification: bool,
    insertable: bool,
    split_identity: u64,
) -> ClickOutcome {
    if action == ClickAction::Primary && !cursor.stack.is_empty() {
        let transferred = contents.insert(cursor, allow_modification && insertable, split_identity);
        return ClickOutcome {
            handled: true,
            transferred,
            sound: Some(if transferred == 0 {
                BundleSound::InsertFail
            } else {
                BundleSound::Insert
            }),
            rewrite_and_broadcast: true,
        };
    }
    contents.clear_selection();
    unhandled()
}

pub fn bundle_in_slot_remove(
    contents: &mut BundleContents,
    action: ClickAction,
    cursor_empty: bool,
    allow_modification: bool,
) -> (ClickOutcome, Option<BundleEntry>) {
    if action != ClickAction::Secondary || !cursor_empty {
        contents.clear_selection();
        return (unhandled(), None);
    }
    let removed = allow_modification.then(|| contents.remove()).flatten();
    (
        ClickOutcome {
            handled: true,
            transferred: removed
                .as_ref()
                .and_then(|entry| u64::try_from(entry.stack.count).ok())
                .unwrap_or(0),
            sound: removed.as_ref().map(|_| BundleSound::RemoveOne),
            rewrite_and_broadcast: true,
        },
        removed,
    )
}

pub const fn held_output_attempt(remaining_duration: u32) -> bool {
    remaining_duration == BUNDLE_USE_DURATION
        || (remaining_duration < 190
            && remaining_duration >= 2
            && remaining_duration.is_multiple_of(2))
}

pub const fn recolor_allowed(same_result: bool, occupied_inputs: u8, matching_dyes: u8) -> bool {
    !same_result && occupied_inputs == 2 && matching_dyes == 1
}

const fn unhandled() -> ClickOutcome {
    ClickOutcome {
        handled: false,
        transferred: 0,
        sound: None,
        rewrite_and_broadcast: false,
    }
}
