use std::collections::BTreeMap;

use crate::java_26_2::play::clientbound::title_tab::packet::{
    ClearTitles, SelectAdvancementsTab, SetActionBarText, SetSubtitleText, SetTitleText,
    SetTitlesAnimation, TabList,
};
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

const DEFAULT_FADE_IN: i32 = 10;
const DEFAULT_STAY: i32 = 70;
const DEFAULT_FADE_OUT: i32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdvancementTabObject {
    pub object_token: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectedAdvancementTab {
    identity: Identifier,
    object_token: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TitleTabTick {
    pub action_bar_expired: bool,
    pub title_expired: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleTabProjection {
    pub action_bar: Option<TextComponentNbt>,
    pub action_bar_remaining: i32,
    pub action_bar_animated_color: bool,
    pub title: Option<TextComponentNbt>,
    pub subtitle: Option<TextComponentNbt>,
    pub title_remaining: i32,
    pub fade_in: i32,
    pub stay: i32,
    pub fade_out: i32,
    pub header: Option<TextComponentNbt>,
    pub footer: Option<TextComponentNbt>,
    selected_tab: Option<SelectedAdvancementTab>,
}

impl Default for TitleTabProjection {
    fn default() -> Self {
        Self {
            action_bar: None,
            action_bar_remaining: 0,
            action_bar_animated_color: false,
            title: None,
            subtitle: None,
            title_remaining: 0,
            fade_in: DEFAULT_FADE_IN,
            stay: DEFAULT_STAY,
            fade_out: DEFAULT_FADE_OUT,
            header: None,
            footer: None,
            selected_tab: None,
        }
    }
}

impl TitleTabProjection {
    pub fn apply_action_bar(&mut self, packet: SetActionBarText) {
        self.action_bar = Some(packet.text);
        self.action_bar_remaining = 60;
        self.action_bar_animated_color = false;
    }

    pub fn apply_subtitle(&mut self, packet: SetSubtitleText) {
        self.subtitle = Some(packet.text);
    }

    pub fn apply_title(&mut self, packet: SetTitleText) {
        self.title = Some(packet.text);
        self.title_remaining = self.effective_title_duration();
    }

    pub fn apply_animation(&mut self, packet: SetTitlesAnimation) {
        if packet.fade_in >= 0 {
            self.fade_in = packet.fade_in;
        }
        if packet.stay >= 0 {
            self.stay = packet.stay;
        }
        if packet.fade_out >= 0 {
            self.fade_out = packet.fade_out;
        }
        if self.title_remaining > 0 {
            self.title_remaining = self.effective_title_duration();
        }
    }

    pub fn apply_clear(&mut self, packet: ClearTitles) {
        self.title = None;
        self.subtitle = None;
        self.title_remaining = 0;
        if packet.reset_times {
            self.fade_in = DEFAULT_FADE_IN;
            self.stay = DEFAULT_STAY;
            self.fade_out = DEFAULT_FADE_OUT;
        }
    }

    pub fn apply_select(
        &mut self,
        packet: &SelectAdvancementsTab,
        tabs: &BTreeMap<Identifier, AdvancementTabObject>,
    ) -> bool {
        let selected = packet.tab.as_ref().and_then(|identity| {
            tabs.get(identity).map(|tab| SelectedAdvancementTab {
                identity: identity.clone(),
                object_token: tab.object_token,
            })
        });
        if self.selected_tab == selected {
            false
        } else {
            self.selected_tab = selected;
            true
        }
    }

    pub fn apply_tab_list(
        &mut self,
        packet: TabList,
        mut flatten: impl FnMut(&TextComponentNbt) -> String,
    ) {
        self.header = (!flatten(&packet.header).is_empty()).then_some(packet.header);
        self.footer = (!flatten(&packet.footer).is_empty()).then_some(packet.footer);
    }

    #[must_use]
    pub fn selected_tab_token(&self) -> Option<u64> {
        self.selected_tab.as_ref().map(|tab| tab.object_token)
    }

    #[must_use]
    pub fn selected_tab_identity(&self) -> Option<&Identifier> {
        self.selected_tab.as_ref().map(|tab| &tab.identity)
    }

    pub fn client_tick(&mut self) -> TitleTabTick {
        let mut result = TitleTabTick::default();
        if self.action_bar_remaining > 0 {
            self.action_bar_remaining -= 1;
            result.action_bar_expired = self.action_bar_remaining == 0;
        }
        if self.title_remaining > 0 {
            self.title_remaining -= 1;
            if self.title_remaining == 0 {
                self.title = None;
                self.subtitle = None;
                result.title_expired = true;
            }
        }
        result
    }

    fn effective_title_duration(&self) -> i32 {
        self.fade_in
            .wrapping_add(self.stay)
            .wrapping_add(self.fade_out)
    }
}
