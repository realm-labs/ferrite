//! Honeycomb's ordered unwaxed-copper transaction.

pub const WAX_WRITE_FLAGS: u16 = 11;
pub const WAX_LEVEL_EVENT: u16 = 3003;
pub const COPPER_COLLECTIONS: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopperChestType {
    Single,
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaxEffect {
    Criterion,
    Shrink,
    Write { flags: u16 },
    BlockChange { companion: bool },
    LevelEvent { companion: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaxOutcome {
    pub mapped: bool,
    pub remaining: u32,
    pub effects: Vec<WaxEffect>,
}

pub fn use_honeycomb(
    mapped: bool,
    server_player: bool,
    stack_count: u32,
    chest_type: Option<CopperChestType>,
) -> WaxOutcome {
    if !mapped {
        return WaxOutcome {
            mapped: false,
            remaining: stack_count,
            effects: Vec::new(),
        };
    }

    let mut effects = Vec::with_capacity(7);
    if server_player {
        effects.push(WaxEffect::Criterion);
    }
    effects.push(WaxEffect::Shrink);
    effects.push(WaxEffect::Write {
        flags: WAX_WRITE_FLAGS,
    });
    effects.push(WaxEffect::BlockChange { companion: false });
    effects.push(WaxEffect::LevelEvent { companion: false });
    if chest_type.is_some_and(|kind| kind != CopperChestType::Single) {
        effects.push(WaxEffect::BlockChange { companion: true });
        effects.push(WaxEffect::LevelEvent { companion: true });
    }

    WaxOutcome {
        mapped: true,
        remaining: stack_count.saturating_sub(1),
        effects,
    }
}
