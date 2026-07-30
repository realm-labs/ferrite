use crate::java_26_2::play::serverbound::anvil_beacon::packet::RenameItem;

const CLIENT_NAME_LIMIT: usize = 50;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilInputProjection {
    pub hover_name: String,
    pub has_custom_name: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilClientProjection {
    input: Option<AnvilInputProjection>,
    edit_text: String,
    accepted_name: String,
    result_present: bool,
    result_custom_name: Option<String>,
    recomputations: u64,
}

impl AnvilClientProjection {
    #[must_use]
    pub fn new(result_present: bool) -> Self {
        Self {
            input: None,
            edit_text: String::new(),
            accepted_name: String::new(),
            result_present,
            result_custom_name: None,
            recomputations: 0,
        }
    }

    pub fn set_input(&mut self, input: Option<AnvilInputProjection>) {
        self.edit_text = input
            .as_ref()
            .map_or_else(String::new, |stack| stack.hover_name.clone());
        self.input = input;
    }

    pub fn set_result_present(&mut self, present: bool) {
        self.result_present = present;
        if !present {
            self.result_custom_name = None;
        }
    }

    pub fn edit(&mut self, entered: &str) -> AnvilClientEdit {
        let Some(input) = &self.input else {
            return AnvilClientEdit::IgnoredMissingInput;
        };
        if entered.encode_utf16().count() > CLIENT_NAME_LIMIT {
            return AnvilClientEdit::RejectedClientLength;
        }
        self.edit_text.clear();
        self.edit_text.push_str(entered);
        let proposed = if !input.has_custom_name && entered == input.hover_name {
            ""
        } else {
            entered
        };
        let filtered = filter_name(proposed);
        if filtered == self.accepted_name {
            return AnvilClientEdit::Unchanged;
        }
        self.accepted_name = filtered.clone();
        self.result_custom_name = projected_custom_name(self.result_present, &filtered);
        self.recomputations = self.recomputations.wrapping_add(1);
        AnvilClientEdit::PredictedAndSend(RenameItem {
            name: proposed.to_owned(),
        })
    }

    #[must_use]
    pub fn edit_text(&self) -> &str {
        &self.edit_text
    }

    #[must_use]
    pub fn accepted_name(&self) -> &str {
        &self.accepted_name
    }

    #[must_use]
    pub fn result_custom_name(&self) -> Option<&str> {
        self.result_custom_name.as_deref()
    }

    #[must_use]
    pub const fn recomputations(&self) -> u64 {
        self.recomputations
    }
}

impl Default for AnvilClientProjection {
    fn default() -> Self {
        Self::new(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnvilClientEdit {
    IgnoredMissingInput,
    RejectedClientLength,
    Unchanged,
    PredictedAndSend(RenameItem),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilMenuState {
    pub still_valid: bool,
    pub accepted_name: String,
    pub result_present: bool,
    pub result_custom_name: Option<String>,
    pub recomputations: u64,
    pub broadcasts: u64,
}

impl AnvilMenuState {
    #[must_use]
    pub fn new(still_valid: bool, result_present: bool) -> Self {
        Self {
            still_valid,
            accepted_name: String::new(),
            result_present,
            result_custom_name: None,
            recomputations: 0,
            broadcasts: 0,
        }
    }
}

pub fn handle_rename(
    current_anvil: Option<&mut AnvilMenuState>,
    packet: &RenameItem,
) -> AnvilRenameOutcome {
    let Some(menu) = current_anvil else {
        return AnvilRenameOutcome::IgnoredWrongMenu;
    };
    if !menu.still_valid {
        return AnvilRenameOutcome::IgnoredInvalidMenu;
    }
    let filtered = filter_name(&packet.name);
    if filtered.encode_utf16().count() > CLIENT_NAME_LIMIT || filtered == menu.accepted_name {
        return AnvilRenameOutcome::NoChange;
    }
    menu.accepted_name = filtered.clone();
    menu.result_custom_name = projected_custom_name(menu.result_present, &filtered);
    menu.recomputations = menu.recomputations.wrapping_add(1);
    menu.broadcasts = menu.broadcasts.wrapping_add(1);
    AnvilRenameOutcome::Applied(AnvilConvergence {
        accepted_name: filtered,
        result_custom_name: menu.result_custom_name.clone(),
        recomputations: menu.recomputations,
        broadcasts: menu.broadcasts,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnvilRenameOutcome {
    IgnoredWrongMenu,
    IgnoredInvalidMenu,
    NoChange,
    Applied(AnvilConvergence),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnvilConvergence {
    pub accepted_name: String,
    pub result_custom_name: Option<String>,
    pub recomputations: u64,
    pub broadcasts: u64,
}

#[must_use]
pub fn filter_name(value: &str) -> String {
    let filtered = value
        .encode_utf16()
        .filter(|unit| !matches!(*unit, 0x00a7 | 0x0000..=0x001f | 0x007f))
        .collect::<Vec<_>>();
    String::from_utf16(&filtered).expect("filtering valid UTF-16 cannot create invalid pairs")
}

fn projected_custom_name(result_present: bool, value: &str) -> Option<String> {
    (result_present && !java_is_blank(value)).then(|| value.to_owned())
}

fn java_is_blank(value: &str) -> bool {
    value.chars().all(java_is_blank_character)
}

fn java_is_blank_character(character: char) -> bool {
    matches!(
        character as u32,
        0x0009..=0x000d
            | 0x001c..=0x0020
            | 0x00a0
            | 0x1680
            | 0x2000..=0x200a
            | 0x2028..=0x2029
            | 0x202f
            | 0x205f
            | 0x3000
    )
}
