use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::{NetworkNbt, TextComponentNbt};

pub const MESSAGE_SIGNATURE_BYTES: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageSignature(pub Box<[u8; MESSAGE_SIGNATURE_BYTES]>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackedMessageSignature {
    Full(MessageSignature),
    CacheIndex(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedMessageBodyPacked {
    pub content: String,
    pub timestamp_ms: i64,
    pub salt: i64,
    pub last_seen: Vec<PackedMessageSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterMask {
    Pass,
    FullyFiltered,
    PartiallyFiltered(Vec<i64>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatParameter {
    Sender,
    Target,
    Content,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatDecoration {
    pub translation_key: String,
    pub parameters: Vec<ChatParameter>,
    pub style: NetworkNbt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectChatType {
    pub chat: ChatDecoration,
    pub narration: ChatDecoration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatTypeHolder {
    Direct(Box<DirectChatType>),
    Registered(Identifier),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundChatType {
    pub holder: ChatTypeHolder,
    pub name: TextComponentNbt,
    pub target: Option<TextComponentNbt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeleteChat {
    pub signature: PackedMessageSignature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DisguisedChat {
    pub message: TextComponentNbt,
    pub chat_type: BoundChatType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerChat {
    pub global_index: i32,
    pub sender: u128,
    pub message_index: i32,
    pub signature: Option<MessageSignature>,
    pub body: SignedMessageBodyPacked,
    pub unsigned_content: Option<TextComponentNbt>,
    pub filter_mask: FilterMask,
    pub chat_type: BoundChatType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemChat {
    pub content: TextComponentNbt,
    pub overlay: bool,
}
