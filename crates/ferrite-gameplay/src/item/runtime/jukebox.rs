//! Jukebox item, song-clock, signal, persistence, and removal rules.

use crate::item::runtime::stack::ItemStack;
use ferrite_foundation::resource::ResourceId;

pub const SONG_PADDING_TICKS: i64 = 20;
pub const PLAY_EVENT_INTERVAL_TICKS: i64 = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct JukeboxSong {
    pub key: ResourceId,
    pub length_seconds: f32,
    pub comparator_output: u8,
}

impl JukeboxSong {
    pub fn padded_finish_tick(&self) -> i64 {
        f64::from(self.length_seconds * 20.0_f32).ceil() as i64 + SONG_PADDING_TICKS
    }

    pub fn has_finished(&self, ticks_since_started: i64) -> bool {
        ticks_since_started >= self.padded_finish_tick()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct JukeboxItem {
    pub stack: ItemStack,
    pub playable: Option<JukeboxSong>,
}

impl JukeboxItem {
    pub fn empty() -> Self {
        Self {
            stack: ItemStack::empty(),
            playable: None,
        }
    }

    pub fn from_default_disc(stack: ItemStack) -> Self {
        let playable = stack.item.as_ref().and_then(default_song_for_item);
        Self { stack, playable }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JukeboxUse {
    Pass,
    TryWithEmptyHand,
    PredictSuccess,
    ServerSuccess,
}

pub fn use_admission(
    captured_has_record: bool,
    live_is_jukebox: bool,
    live_has_record: bool,
    item_is_playable: bool,
    server_side: bool,
) -> JukeboxUse {
    if captured_has_record {
        return JukeboxUse::TryWithEmptyHand;
    }
    if !live_is_jukebox || live_has_record || !item_is_playable {
        return JukeboxUse::Pass;
    }
    if server_side {
        JukeboxUse::ServerSuccess
    } else {
        JukeboxUse::PredictSuccess
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JukeboxEffects {
    pub play_level_event: Option<ResourceId>,
    pub stop_level_events: u8,
    pub play_game_events: u8,
    pub stop_game_events: u8,
    pub neighbor_updates: u8,
    pub dirty_marks: u8,
    pub unsourced_block_changes: u8,
    pub player_block_changes: u8,
    pub note_particle_value: Option<u8>,
    pub item_entity_spawned: bool,
    pub ejection_float_draws: u8,
}

impl JukeboxEffects {
    fn merge(&mut self, other: Self) {
        if other.play_level_event.is_some() {
            self.play_level_event = other.play_level_event;
        }
        self.stop_level_events += other.stop_level_events;
        self.play_game_events += other.play_game_events;
        self.stop_game_events += other.stop_game_events;
        self.neighbor_updates += other.neighbor_updates;
        self.dirty_marks += other.dirty_marks;
        self.unsourced_block_changes += other.unsourced_block_changes;
        self.player_block_changes += other.player_block_changes;
        if other.note_particle_value.is_some() {
            self.note_particle_value = other.note_particle_value;
        }
        self.item_entity_spawned |= other.item_entity_spawned;
        self.ejection_float_draws += other.ejection_float_draws;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Jukebox {
    has_record_state: bool,
    item: JukeboxItem,
    active_song: Option<JukeboxSong>,
    ticks_since_song_started: i64,
}

impl Jukebox {
    pub fn empty() -> Self {
        Self {
            has_record_state: false,
            item: JukeboxItem::empty(),
            active_song: None,
            ticks_since_song_started: 0,
        }
    }

    pub fn placed(has_typed_record_item_field: bool) -> Self {
        Self {
            has_record_state: has_typed_record_item_field,
            ..Self::empty()
        }
    }

    pub const fn has_record_state(&self) -> bool {
        self.has_record_state
    }

    pub const fn item(&self) -> &JukeboxItem {
        &self.item
    }

    pub const fn active_song(&self) -> Option<&JukeboxSong> {
        self.active_song.as_ref()
    }

    pub const fn ticks_since_song_started(&self) -> i64 {
        self.ticks_since_song_started
    }

    pub const fn has_ticker(&self) -> bool {
        self.has_record_state
    }

    pub const fn source_signal(&self) -> u8 {
        if self.active_song.is_some() { 15 } else { 0 }
    }

    pub fn comparator_output(&self) -> u8 {
        self.item
            .playable
            .as_ref()
            .map_or(0, |song| song.comparator_output)
    }

    pub fn can_place_item(&self, item: &JukeboxItem) -> bool {
        self.item.stack.is_empty() && item.playable.is_some()
    }

    pub const fn can_take_to_destination(destination_has_empty_slot: bool) -> bool {
        destination_has_empty_slot
    }

    pub fn insert_from_player(&mut self, item: JukeboxItem) -> JukeboxEffects {
        let mut effects = self.set_item(item);
        effects.player_block_changes = 1;
        effects
    }

    pub fn set_item(&mut self, item: JukeboxItem) -> JukeboxEffects {
        self.item = item;
        self.has_record_state = !self.item.stack.is_empty();
        let mut effects = JukeboxEffects {
            unsourced_block_changes: 1,
            ..JukeboxEffects::default()
        };
        if let Some(song) = self.item.playable.clone() {
            effects.merge(self.play(song));
        } else if self.active_song.is_some() {
            effects.merge(self.stop());
        }
        effects
    }

    pub fn remove_item(&mut self) -> (JukeboxItem, JukeboxEffects) {
        let removed = std::mem::replace(&mut self.item, JukeboxItem::empty());
        self.has_record_state = false;
        let mut effects = JukeboxEffects {
            unsourced_block_changes: 1,
            ..JukeboxEffects::default()
        };
        if self.active_song.is_some() {
            effects.merge(self.stop());
        }
        (removed, effects)
    }

    pub fn eject(&mut self) -> (JukeboxItem, JukeboxEffects) {
        if self.item.stack.is_empty() {
            return (JukeboxItem::empty(), JukeboxEffects::default());
        }
        let (item, mut effects) = self.remove_item();
        effects.neighbor_updates += 1;
        effects.dirty_marks += 1;
        effects.item_entity_spawned = true;
        effects.ejection_float_draws = 2;
        (item, effects)
    }

    pub fn tick(&mut self, note_value: u8) -> JukeboxEffects {
        let Some(song) = self.active_song.as_ref() else {
            return JukeboxEffects::default();
        };
        if song.has_finished(self.ticks_since_song_started) {
            return self.stop();
        }
        let periodic = self
            .ticks_since_song_started
            .rem_euclid(PLAY_EVENT_INTERVAL_TICKS)
            == 0;
        self.ticks_since_song_started += 1;
        if periodic {
            JukeboxEffects {
                play_game_events: 1,
                note_particle_value: Some(note_value % 4),
                ..JukeboxEffects::default()
            }
        } else {
            JukeboxEffects::default()
        }
    }

    pub fn load(&mut self, item: JukeboxItem, persisted_ticks: Option<i64>) -> JukeboxEffects {
        let same_item = self.item.stack.equal_stack(&item.stack);
        let mut effects = JukeboxEffects::default();
        if !same_item && self.active_song.is_some() {
            effects.merge(self.stop());
        }
        self.item = item;

        let Some(ticks) = persisted_ticks else {
            return effects;
        };
        let Some(song) = self.item.playable.clone() else {
            return effects;
        };
        if song.has_finished(ticks) {
            return effects;
        }
        self.active_song = Some(song);
        self.ticks_since_song_started = ticks;
        effects
    }

    pub fn pre_remove(
        &mut self,
        suppress_side_effects: bool,
    ) -> (Option<JukeboxItem>, JukeboxEffects) {
        if suppress_side_effects {
            return (None, JukeboxEffects::default());
        }
        let (item, effects) = self.eject();
        (Some(item), effects)
    }

    pub fn set_removed(&mut self) -> JukeboxEffects {
        self.active_song = None;
        self.ticks_since_song_started = 0;
        JukeboxEffects {
            stop_level_events: 1,
            stop_game_events: 1,
            ..JukeboxEffects::default()
        }
    }

    fn play(&mut self, song: JukeboxSong) -> JukeboxEffects {
        self.active_song = Some(song.clone());
        self.ticks_since_song_started = 0;
        JukeboxEffects {
            play_level_event: Some(song.key),
            neighbor_updates: 1,
            dirty_marks: 1,
            ..JukeboxEffects::default()
        }
    }

    fn stop(&mut self) -> JukeboxEffects {
        self.active_song = None;
        self.ticks_since_song_started = 0;
        JukeboxEffects {
            stop_level_events: 1,
            stop_game_events: 1,
            neighbor_updates: 1,
            dirty_marks: 1,
            ..JukeboxEffects::default()
        }
    }
}

pub fn default_song_for_item(item: &ResourceId) -> Option<JukeboxSong> {
    if item.namespace() != "minecraft" {
        return None;
    }
    let (song, length_seconds, comparator_output) = match item.path() {
        "music_disc_11" => ("11", 71.0, 11),
        "music_disc_13" => ("13", 178.0, 1),
        "music_disc_5" => ("5", 178.0, 15),
        "music_disc_blocks" => ("blocks", 345.0, 3),
        "music_disc_bounce" => ("bounce", 234.0, 8),
        "music_disc_cat" => ("cat", 185.0, 2),
        "music_disc_chirp" => ("chirp", 185.0, 4),
        "music_disc_creator" => ("creator", 176.0, 12),
        "music_disc_creator_music_box" => ("creator_music_box", 73.0, 11),
        "music_disc_far" => ("far", 174.0, 5),
        "music_disc_lava_chicken" => ("lava_chicken", 134.0, 9),
        "music_disc_mall" => ("mall", 197.0, 6),
        "music_disc_mellohi" => ("mellohi", 96.0, 7),
        "music_disc_otherside" => ("otherside", 195.0, 14),
        "music_disc_pigstep" => ("pigstep", 149.0, 13),
        "music_disc_precipice" => ("precipice", 299.0, 13),
        "music_disc_relic" => ("relic", 218.0, 14),
        "music_disc_stal" => ("stal", 150.0, 8),
        "music_disc_strad" => ("strad", 188.0, 9),
        "music_disc_tears" => ("tears", 175.0, 10),
        "music_disc_wait" => ("wait", 238.0, 12),
        "music_disc_ward" => ("ward", 251.0, 10),
        _ => return None,
    };
    Some(JukeboxSong {
        key: ResourceId::minecraft(song).expect("locked jukebox song identifier"),
        length_seconds,
        comparator_output,
    })
}
