use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use thiserror::Error;

use crate::java_26_2::play::clientbound::packet::PlayClientboundPacket;
use crate::java_26_2::play::clientbound::special_screen::packet::{
    InteractionHand, MountScreenOpen, OpenSignEditor,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackedMountKind {
    Horse,
    Nautilus,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectedMountMenu {
    pub container_id: i32,
    pub entity_id: i32,
    pub kind: TrackedMountKind,
    pub allocated_inventory_slots: usize,
    pub cargo_slots: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilteredBookPage {
    pub raw: String,
    pub filtered: Option<String>,
}

impl FilteredBookPage {
    fn selected(&self, filtering_enabled: bool) -> String {
        if filtering_enabled {
            self.filtered.clone().unwrap_or_else(|| self.raw.clone())
        } else {
            self.raw.clone()
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BookStackProjection {
    pub written_pages: Option<Vec<FilteredBookPage>>,
    pub writable_pages: Option<Vec<FilteredBookPage>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookViewKind {
    Written,
    Writable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookViewProjection {
    pub hand: InteractionHand,
    pub kind: BookViewKind,
    pub pages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignKind {
    Ordinary,
    Hanging,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignBlockProjection {
    pub kind: SignKind,
    pub front: [String; 4],
    pub back: [String; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignEditorProjection {
    pub position: BlockPos,
    pub kind: SignKind,
    pub front_text: bool,
    pub lines: [String; 4],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecialScreenAction {
    Ignored,
    MountOpened(ProjectedMountMenu),
    BookOpened(BookViewProjection),
    SignOpened(SignEditorProjection),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialScreenClientProjection {
    maximum_mount_allocation: usize,
    tracked_mounts: BTreeMap<i32, TrackedMountKind>,
    signs: BTreeMap<BlockPos, SignBlockProjection>,
    main_hand: BookStackProjection,
    off_hand: BookStackProjection,
    filtering_enabled: bool,
    current_mount: Option<ProjectedMountMenu>,
    current_book: Option<BookViewProjection>,
    current_sign: Option<SignEditorProjection>,
}

impl SpecialScreenClientProjection {
    #[must_use]
    pub fn new(maximum_mount_allocation: usize) -> Self {
        Self {
            maximum_mount_allocation,
            tracked_mounts: BTreeMap::new(),
            signs: BTreeMap::new(),
            main_hand: BookStackProjection::default(),
            off_hand: BookStackProjection::default(),
            filtering_enabled: false,
            current_mount: None,
            current_book: None,
            current_sign: None,
        }
    }

    pub fn track_mount(&mut self, entity_id: i32, kind: TrackedMountKind) {
        self.tracked_mounts.insert(entity_id, kind);
    }

    pub fn track_sign(&mut self, position: BlockPos, sign: SignBlockProjection) {
        self.signs.insert(position, sign);
    }

    pub fn set_hand(&mut self, hand: InteractionHand, stack: BookStackProjection) {
        *self.hand_mut(hand) = stack;
    }

    pub const fn set_filtering_enabled(&mut self, enabled: bool) {
        self.filtering_enabled = enabled;
    }

    pub fn apply(
        &mut self,
        packet: &PlayClientboundPacket,
    ) -> Result<SpecialScreenAction, SpecialScreenProjectionError> {
        match packet {
            PlayClientboundPacket::MountScreenOpen(packet) => self.apply_mount(*packet),
            PlayClientboundPacket::OpenBook(hand) => Ok(self.apply_book(*hand)),
            PlayClientboundPacket::OpenSignEditor(packet) => Ok(self.apply_sign(*packet)),
            _ => Err(SpecialScreenProjectionError::WrongPacketFamily),
        }
    }

    #[must_use]
    pub const fn current_mount(&self) -> Option<ProjectedMountMenu> {
        self.current_mount
    }

    #[must_use]
    pub const fn current_book(&self) -> Option<&BookViewProjection> {
        self.current_book.as_ref()
    }

    #[must_use]
    pub const fn current_sign(&self) -> Option<&SignEditorProjection> {
        self.current_sign.as_ref()
    }

    fn apply_mount(
        &mut self,
        packet: MountScreenOpen,
    ) -> Result<SpecialScreenAction, SpecialScreenProjectionError> {
        let wrapped_size = packet.inventory_columns.wrapping_mul(3);
        let allocated_inventory_slots = usize::try_from(wrapped_size).map_err(|_| {
            SpecialScreenProjectionError::NegativeMountAllocation {
                columns: packet.inventory_columns,
                wrapped_size,
            }
        })?;
        if allocated_inventory_slots > self.maximum_mount_allocation {
            return Err(SpecialScreenProjectionError::MountAllocationLimit {
                requested: allocated_inventory_slots,
                maximum: self.maximum_mount_allocation,
            });
        }
        let Some(kind) = self.tracked_mounts.get(&packet.entity_id).copied() else {
            return Ok(SpecialScreenAction::Ignored);
        };
        if kind == TrackedMountKind::Other {
            return Ok(SpecialScreenAction::Ignored);
        }
        let menu = ProjectedMountMenu {
            container_id: packet.container_id,
            entity_id: packet.entity_id,
            kind,
            allocated_inventory_slots,
            cargo_slots: if kind == TrackedMountKind::Horse {
                allocated_inventory_slots
            } else {
                0
            },
        };
        self.current_mount = Some(menu);
        Ok(SpecialScreenAction::MountOpened(menu))
    }

    fn apply_book(&mut self, hand: InteractionHand) -> SpecialScreenAction {
        let stack = self.hand(hand);
        let view = if let Some(pages) = &stack.written_pages {
            Some(BookViewProjection {
                hand,
                kind: BookViewKind::Written,
                pages: select_pages(pages, self.filtering_enabled),
            })
        } else {
            stack
                .writable_pages
                .as_ref()
                .map(|pages| BookViewProjection {
                    hand,
                    kind: BookViewKind::Writable,
                    pages: select_pages(pages, self.filtering_enabled),
                })
        };
        let Some(view) = view else {
            return SpecialScreenAction::Ignored;
        };
        self.current_book = Some(view.clone());
        SpecialScreenAction::BookOpened(view)
    }

    fn apply_sign(&mut self, packet: OpenSignEditor) -> SpecialScreenAction {
        let Some(sign) = self.signs.get(&packet.position) else {
            return SpecialScreenAction::Ignored;
        };
        let editor = SignEditorProjection {
            position: packet.position,
            kind: sign.kind,
            front_text: packet.front_text,
            lines: if packet.front_text {
                sign.front.clone()
            } else {
                sign.back.clone()
            },
        };
        self.current_sign = Some(editor.clone());
        SpecialScreenAction::SignOpened(editor)
    }

    fn hand(&self, hand: InteractionHand) -> &BookStackProjection {
        match hand {
            InteractionHand::Main => &self.main_hand,
            InteractionHand::Off => &self.off_hand,
        }
    }

    fn hand_mut(&mut self, hand: InteractionHand) -> &mut BookStackProjection {
        match hand {
            InteractionHand::Main => &mut self.main_hand,
            InteractionHand::Off => &mut self.off_hand,
        }
    }
}

fn select_pages(pages: &[FilteredBookPage], filtering_enabled: bool) -> Vec<String> {
    pages
        .iter()
        .map(|page| page.selected(filtering_enabled))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpecialScreenProjectionError {
    #[error("packet does not belong to the special-screen family")]
    WrongPacketFamily,
    #[error("mount columns {columns} wrap to negative allocation {wrapped_size}")]
    NegativeMountAllocation { columns: i32, wrapped_size: i32 },
    #[error("mount inventory allocation {requested} exceeds bounded maximum {maximum}")]
    MountAllocationLimit { requested: usize, maximum: usize },
}
