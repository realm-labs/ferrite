use std::collections::BTreeMap;

use crate::java_26_2::play::serverbound::inventory_auxiliary::packet::EditBook;
use crate::java_26_2::value::identifier::Identifier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterableText {
    pub raw: String,
    pub filtered: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextFilterResult {
    pub raw: String,
    pub filtered: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrittenBookContent {
    pub title: FilterableText,
    pub author: String,
    pub generation: i32,
    pub pages: Vec<FilterableText>,
    pub resolved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookStack {
    pub item: Identifier,
    pub writable_pages: Option<Vec<FilterableText>>,
    pub written_content: Option<WrittenBookContent>,
    pub retained_components: BTreeMap<Identifier, Vec<u8>>,
}

impl BookStack {
    #[must_use]
    pub fn writable(item: Identifier, pages: Vec<FilterableText>) -> Self {
        Self {
            item,
            writable_pages: Some(pages),
            written_content: None,
            retained_components: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookInventory {
    slots: BTreeMap<i32, BookStack>,
    ordinary_projections: u64,
}

impl BookInventory {
    pub fn set(&mut self, slot: i32, stack: BookStack) {
        self.slots.insert(slot, stack);
    }

    pub fn remove(&mut self, slot: i32) -> Option<BookStack> {
        self.slots.remove(&slot)
    }

    #[must_use]
    pub fn get(&self, slot: i32) -> Option<&BookStack> {
        self.slots.get(&slot)
    }

    #[must_use]
    pub const fn ordinary_projections(&self) -> u64 {
        self.ordinary_projections
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookFilterTask {
    pub id: u64,
    pub slot: i32,
    pub pages: Vec<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookFilterOutput {
    pub pages: Vec<TextFilterResult>,
    pub title: Option<TextFilterResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookFilterOutcome {
    IgnoredInvalidSlot,
    UnknownOrCancelledTask,
    FilterFailed,
    MalformedFilterOutput,
    MissingWritableContent,
    UpdatedWritable,
    FinalizedWritten,
}

#[derive(Debug, Clone)]
pub struct BookFilterService {
    next_task_id: u64,
    pending: BTreeMap<u64, BookFilterTask>,
    connected: bool,
}

impl Default for BookFilterService {
    fn default() -> Self {
        Self {
            next_task_id: 0,
            pending: BTreeMap::new(),
            connected: true,
        }
    }
}

impl BookFilterService {
    #[must_use]
    pub fn connected() -> Self {
        Self {
            connected: true,
            ..Self::default()
        }
    }

    pub fn admit(&mut self, packet: EditBook) -> Result<BookFilterTask, BookFilterOutcome> {
        if !self.connected {
            return Err(BookFilterOutcome::UnknownOrCancelledTask);
        }
        if !is_book_inventory_slot(packet.slot) {
            return Err(BookFilterOutcome::IgnoredInvalidSlot);
        }
        let task = BookFilterTask {
            id: self.next_task_id,
            slot: packet.slot,
            pages: packet.pages,
            title: packet.title,
        };
        self.next_task_id = self.next_task_id.wrapping_add(1);
        self.pending.insert(task.id, task.clone());
        Ok(task)
    }

    pub fn disconnect(&mut self) {
        self.connected = false;
        self.pending.clear();
    }

    pub fn complete(
        &mut self,
        task_id: u64,
        result: Result<BookFilterOutput, ()>,
        inventory: &mut BookInventory,
        filtering_enabled: bool,
        author: &str,
    ) -> BookFilterOutcome {
        if !self.connected {
            return BookFilterOutcome::UnknownOrCancelledTask;
        }
        let Some(task) = self.pending.remove(&task_id) else {
            return BookFilterOutcome::UnknownOrCancelledTask;
        };
        let Ok(output) = result else {
            return BookFilterOutcome::FilterFailed;
        };
        if output.pages.len() != task.pages.len() || output.title.is_some() != task.title.is_some()
        {
            return BookFilterOutcome::MalformedFilterOutput;
        }
        let Some(stack) = inventory.slots.get_mut(&task.slot) else {
            return BookFilterOutcome::MissingWritableContent;
        };
        if stack.writable_pages.is_none() {
            return BookFilterOutcome::MissingWritableContent;
        }
        let pages = output
            .pages
            .into_iter()
            .map(|result| normalize_filter_result(result, filtering_enabled))
            .collect::<Vec<_>>();
        let outcome = if let Some(title) = output.title {
            stack.item = Identifier::minecraft("written_book")
                .expect("minecraft:written_book is a valid constant identifier");
            stack.writable_pages = None;
            stack.written_content = Some(WrittenBookContent {
                title: normalize_filter_result(title, filtering_enabled),
                author: author.to_owned(),
                generation: 0,
                pages,
                resolved: true,
            });
            BookFilterOutcome::FinalizedWritten
        } else {
            stack.writable_pages = Some(pages);
            BookFilterOutcome::UpdatedWritable
        };
        inventory.ordinary_projections = inventory.ordinary_projections.wrapping_add(1);
        outcome
    }

    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

pub struct BookClient;

impl BookClient {
    #[must_use]
    pub fn done(slot: i32, mut pages: Vec<String>) -> EditBook {
        while pages.last().is_some_and(String::is_empty) {
            pages.pop();
        }
        EditBook {
            slot,
            pages,
            title: None,
        }
    }

    #[must_use]
    pub fn finalize(slot: i32, pages: Vec<String>, title: &str) -> EditBook {
        EditBook {
            slot,
            pages,
            title: Some(title.trim().to_owned()),
        }
    }

    #[must_use]
    pub const fn escape() -> Option<EditBook> {
        None
    }

    #[must_use]
    pub const fn cancel_signing() -> Option<EditBook> {
        None
    }
}

#[must_use]
pub const fn is_book_inventory_slot(slot: i32) -> bool {
    matches!(slot, 0..=8 | 40)
}

fn normalize_filter_result(result: TextFilterResult, filtering_enabled: bool) -> FilterableText {
    if filtering_enabled {
        FilterableText {
            raw: result.filtered.unwrap_or_default(),
            filtered: None,
        }
    } else {
        FilterableText {
            raw: result.raw,
            filtered: result.filtered,
        }
    }
}
