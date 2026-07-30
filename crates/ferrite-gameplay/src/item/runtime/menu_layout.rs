//! Locked menu slot layouts and quick-move routing families.

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuKind {
    Generic9x1,
    Generic9x2,
    Generic9x3,
    Generic9x4,
    Generic9x5,
    Generic9x6,
    Generic3x3,
    Crafter3x3,
    Hopper,
    ShulkerBox,
    Beacon,
    Furnace,
    BlastFurnace,
    Smoker,
    BrewingStand,
    Crafting,
    CartographyTable,
    Enchantment,
    Grindstone,
    Anvil,
    Smithing,
    Loom,
    Merchant,
    Stonecutter,
    Lectern,
}

impl MenuKind {
    pub const ALL: [Self; 25] = [
        Self::Generic9x1,
        Self::Generic9x2,
        Self::Generic9x3,
        Self::Generic9x4,
        Self::Generic9x5,
        Self::Generic9x6,
        Self::Generic3x3,
        Self::Crafter3x3,
        Self::Hopper,
        Self::ShulkerBox,
        Self::Beacon,
        Self::Furnace,
        Self::BlastFurnace,
        Self::Smoker,
        Self::BrewingStand,
        Self::Crafting,
        Self::CartographyTable,
        Self::Enchantment,
        Self::Grindstone,
        Self::Anvil,
        Self::Smithing,
        Self::Loom,
        Self::Merchant,
        Self::Stonecutter,
        Self::Lectern,
    ];

    pub const fn registry_path(self) -> &'static str {
        match self {
            Self::Generic9x1 => "generic_9x1",
            Self::Generic9x2 => "generic_9x2",
            Self::Generic9x3 => "generic_9x3",
            Self::Generic9x4 => "generic_9x4",
            Self::Generic9x5 => "generic_9x5",
            Self::Generic9x6 => "generic_9x6",
            Self::Generic3x3 => "generic_3x3",
            Self::Crafter3x3 => "crafter_3x3",
            Self::Hopper => "hopper",
            Self::ShulkerBox => "shulker_box",
            Self::Beacon => "beacon",
            Self::Furnace => "furnace",
            Self::BlastFurnace => "blast_furnace",
            Self::Smoker => "smoker",
            Self::BrewingStand => "brewing_stand",
            Self::Crafting => "crafting",
            Self::CartographyTable => "cartography_table",
            Self::Enchantment => "enchantment",
            Self::Grindstone => "grindstone",
            Self::Anvil => "anvil",
            Self::Smithing => "smithing",
            Self::Loom => "loom",
            Self::Merchant => "merchant",
            Self::Stonecutter => "stonecutter",
            Self::Lectern => "lectern",
        }
    }

    pub fn profile(self) -> MenuLayout {
        match self {
            Self::Generic9x1 => generic_rows(1),
            Self::Generic9x2 => generic_rows(2),
            Self::Generic9x3 => generic_rows(3),
            Self::Generic9x4 => generic_rows(4),
            Self::Generic9x5 => generic_rows(5),
            Self::Generic9x6 => generic_rows(6),
            Self::Generic3x3 => symmetric(0..9, 9..36, 36..45),
            Self::Crafter3x3 => MenuLayout {
                machine: 0..9,
                player_main: 9..36,
                hotbar: 36..45,
                total_slots: 46,
                routing: QuickMoveRouting::Crafter,
            },
            Self::Hopper => symmetric(0..5, 5..32, 32..41),
            Self::ShulkerBox => MenuLayout {
                machine: 0..27,
                player_main: 27..54,
                hotbar: 54..63,
                total_slots: 63,
                routing: QuickMoveRouting::Shulker,
            },
            Self::Beacon => specialized(0..1, 1..28, 28..37, QuickMoveRouting::Beacon),
            Self::Furnace | Self::BlastFurnace | Self::Smoker => {
                specialized(0..3, 3..30, 30..39, QuickMoveRouting::Furnace)
            }
            Self::BrewingStand => specialized(0..5, 5..32, 32..41, QuickMoveRouting::Brewing),
            Self::Crafting => specialized(0..10, 10..37, 37..46, QuickMoveRouting::Crafting),
            Self::CartographyTable => {
                specialized(0..3, 3..30, 30..39, QuickMoveRouting::Cartography)
            }
            Self::Enchantment => specialized(0..2, 2..29, 29..38, QuickMoveRouting::Enchantment),
            Self::Grindstone => specialized(0..3, 3..30, 30..39, QuickMoveRouting::Grindstone),
            Self::Anvil => specialized(0..3, 3..30, 30..39, QuickMoveRouting::Anvil),
            Self::Smithing => specialized(0..4, 4..31, 31..40, QuickMoveRouting::Smithing),
            Self::Loom => specialized(0..4, 4..31, 31..40, QuickMoveRouting::Loom),
            Self::Merchant => specialized(0..3, 3..30, 30..39, QuickMoveRouting::Merchant),
            Self::Stonecutter => specialized(0..2, 2..29, 29..38, QuickMoveRouting::Stonecutter),
            Self::Lectern => MenuLayout {
                machine: 0..1,
                player_main: 1..1,
                hotbar: 1..1,
                total_slots: 1,
                routing: QuickMoveRouting::Lectern,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuLayout {
    pub machine: Range<usize>,
    pub player_main: Range<usize>,
    pub hotbar: Range<usize>,
    pub total_slots: usize,
    pub routing: QuickMoveRouting,
}

impl MenuLayout {
    pub fn player_slots(&self) -> Range<usize> {
        self.player_main.start..self.hotbar.end
    }

    pub fn simple_quick_move_target(&self, source_slot: usize) -> Option<MoveTarget> {
        match self.routing {
            QuickMoveRouting::Symmetric | QuickMoveRouting::Shulker => {
                if self.machine.contains(&source_slot) {
                    Some(MoveTarget {
                        range: self.player_slots(),
                        reverse: true,
                    })
                } else if self.player_slots().contains(&source_slot) {
                    Some(MoveTarget {
                        range: self.machine.clone(),
                        reverse: false,
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickMoveRouting {
    Symmetric,
    Crafter,
    Shulker,
    Beacon,
    Furnace,
    Brewing,
    Crafting,
    Cartography,
    Enchantment,
    Grindstone,
    Anvil,
    Smithing,
    Loom,
    Merchant,
    Stonecutter,
    Lectern,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveTarget {
    pub range: Range<usize>,
    pub reverse: bool,
}

fn generic_rows(rows: usize) -> MenuLayout {
    let machine_end = rows * 9;
    symmetric(
        0..machine_end,
        machine_end..machine_end + 27,
        machine_end + 27..machine_end + 36,
    )
}

fn symmetric(machine: Range<usize>, player_main: Range<usize>, hotbar: Range<usize>) -> MenuLayout {
    let total_slots = hotbar.end;
    MenuLayout {
        machine,
        player_main,
        hotbar,
        total_slots,
        routing: QuickMoveRouting::Symmetric,
    }
}

fn specialized(
    machine: Range<usize>,
    player_main: Range<usize>,
    hotbar: Range<usize>,
    routing: QuickMoveRouting,
) -> MenuLayout {
    let total_slots = hotbar.end;
    MenuLayout {
        machine,
        player_main,
        hotbar,
        total_slots,
        routing,
    }
}
