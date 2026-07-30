//! Book-family component and live-tag dispatch.

use crate::item::runtime::catalog::ItemKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookRole {
    Bookshelf,
    Lectern,
    TableEnchantable,
    WritableContent,
    WrittenContent,
    StoredEnchantments,
}

pub const fn has_book_role(item: ItemKind, role: BookRole) -> bool {
    match role {
        BookRole::Bookshelf => matches!(
            item,
            ItemKind::Book
                | ItemKind::EnchantedBook
                | ItemKind::WritableBook
                | ItemKind::WrittenBook
        ),
        BookRole::Lectern => matches!(item, ItemKind::WritableBook | ItemKind::WrittenBook),
        BookRole::TableEnchantable => matches!(item, ItemKind::Book),
        BookRole::WritableContent => matches!(item, ItemKind::WritableBook),
        BookRole::WrittenContent => matches!(item, ItemKind::WrittenBook),
        BookRole::StoredEnchantments => matches!(item, ItemKind::EnchantedBook),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BookSound {
    Insert,
    InsertEnchanted,
    Remove,
    RemoveEnchanted,
}

pub const fn bookshelf_sound(item: ItemKind, inserting: bool) -> Option<BookSound> {
    if !has_book_role(item, BookRole::Bookshelf) {
        return None;
    }
    Some(match (inserting, item) {
        (true, ItemKind::EnchantedBook) => BookSound::InsertEnchanted,
        (false, ItemKind::EnchantedBook) => BookSound::RemoveEnchanted,
        (true, _) => BookSound::Insert,
        (false, _) => BookSound::Remove,
    })
}

pub const fn signed_generation(source_generation: u8) -> u8 {
    if source_generation >= 1 { 2 } else { 1 }
}
