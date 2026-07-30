use ferrite_foundation::coordinate::BlockPos;

use crate::java_26_2::login::profile::ProfileProperty;
use crate::java_26_2::play::clientbound::entity_effects::particle::Particle;
use crate::java_26_2::play::item::ItemStack;
use crate::java_26_2::value::identifier::Identifier;
use crate::java_26_2::value::nbt::TextComponentNbt;

pub const SERIALIZER_COUNT: i32 = 43;
pub const ACCESSOR_DECLARATION_COUNT: usize = 221;
pub const ACCESSOR_TABLE_SHA1: &str = "b489eec18fc1981ebfb7ac97c54a4485fe2f938a";
pub const SERIALIZER_TABLE_SHA1: &str = "96047ad220ac7064e205594f3222d182c87591d7";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(i32)]
pub enum MetadataSerializer {
    Byte = 0,
    Int = 1,
    Long = 2,
    Float = 3,
    String = 4,
    Component = 5,
    OptionalComponent = 6,
    ItemStack = 7,
    Boolean = 8,
    Rotations = 9,
    BlockPos = 10,
    OptionalBlockPos = 11,
    Direction = 12,
    OptionalLivingEntityReference = 13,
    BlockState = 14,
    OptionalBlockState = 15,
    Particle = 16,
    Particles = 17,
    VillagerData = 18,
    OptionalUnsignedInt = 19,
    Pose = 20,
    CatVariant = 21,
    CatSoundVariant = 22,
    CowVariant = 23,
    CowSoundVariant = 24,
    WolfVariant = 25,
    WolfSoundVariant = 26,
    FrogVariant = 27,
    PigVariant = 28,
    PigSoundVariant = 29,
    ChickenVariant = 30,
    ChickenSoundVariant = 31,
    ZombieNautilusVariant = 32,
    OptionalGlobalPos = 33,
    PaintingVariant = 34,
    SnifferState = 35,
    ArmadilloState = 36,
    CopperGolemState = 37,
    WeatheringCopperState = 38,
    Vector3 = 39,
    Quaternion = 40,
    ResolvableProfile = 41,
    HumanoidArm = 42,
}

impl MetadataSerializer {
    #[must_use]
    pub const fn raw_id(self) -> i32 {
        self as i32
    }

    #[must_use]
    pub const fn from_raw_id(raw_id: i32) -> Option<Self> {
        Some(match raw_id {
            0 => Self::Byte,
            1 => Self::Int,
            2 => Self::Long,
            3 => Self::Float,
            4 => Self::String,
            5 => Self::Component,
            6 => Self::OptionalComponent,
            7 => Self::ItemStack,
            8 => Self::Boolean,
            9 => Self::Rotations,
            10 => Self::BlockPos,
            11 => Self::OptionalBlockPos,
            12 => Self::Direction,
            13 => Self::OptionalLivingEntityReference,
            14 => Self::BlockState,
            15 => Self::OptionalBlockState,
            16 => Self::Particle,
            17 => Self::Particles,
            18 => Self::VillagerData,
            19 => Self::OptionalUnsignedInt,
            20 => Self::Pose,
            21 => Self::CatVariant,
            22 => Self::CatSoundVariant,
            23 => Self::CowVariant,
            24 => Self::CowSoundVariant,
            25 => Self::WolfVariant,
            26 => Self::WolfSoundVariant,
            27 => Self::FrogVariant,
            28 => Self::PigVariant,
            29 => Self::PigSoundVariant,
            30 => Self::ChickenVariant,
            31 => Self::ChickenSoundVariant,
            32 => Self::ZombieNautilusVariant,
            33 => Self::OptionalGlobalPos,
            34 => Self::PaintingVariant,
            35 => Self::SnifferState,
            36 => Self::ArmadilloState,
            37 => Self::CopperGolemState,
            38 => Self::WeatheringCopperState,
            39 => Self::Vector3,
            40 => Self::Quaternion,
            41 => Self::ResolvableProfile,
            42 => Self::HumanoidArm,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MetadataEntry {
    pub slot: u8,
    pub serializer: MetadataSerializer,
    pub value: MetadataValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    Byte(i8),
    Int(i32),
    Long(i64),
    Float(f32),
    String(String),
    Component(TextComponentNbt),
    OptionalComponent(Option<TextComponentNbt>),
    ItemStack(ItemStack),
    Boolean(bool),
    Rotations([f32; 3]),
    BlockPos(BlockPos),
    OptionalBlockPos(Option<BlockPos>),
    Direction(u8),
    OptionalLivingEntityReference(Option<u128>),
    BlockState(Option<i32>),
    OptionalBlockState(Option<i32>),
    Particle(Particle),
    Particles(Vec<Particle>),
    VillagerData(VillagerData),
    OptionalUnsignedInt(Option<i32>),
    Pose(u8),
    Holder {
        serializer: MetadataSerializer,
        identity: Identifier,
    },
    OptionalGlobalPos(Option<GlobalPos>),
    EnumState {
        serializer: MetadataSerializer,
        value: u8,
    },
    Vector3([f32; 3]),
    Quaternion([f32; 4]),
    ResolvableProfile(ResolvableProfile),
    HumanoidArm(HumanoidArm),
}

impl MetadataValue {
    #[must_use]
    pub const fn serializer(&self) -> MetadataSerializer {
        match self {
            Self::Byte(_) => MetadataSerializer::Byte,
            Self::Int(_) => MetadataSerializer::Int,
            Self::Long(_) => MetadataSerializer::Long,
            Self::Float(_) => MetadataSerializer::Float,
            Self::String(_) => MetadataSerializer::String,
            Self::Component(_) => MetadataSerializer::Component,
            Self::OptionalComponent(_) => MetadataSerializer::OptionalComponent,
            Self::ItemStack(_) => MetadataSerializer::ItemStack,
            Self::Boolean(_) => MetadataSerializer::Boolean,
            Self::Rotations(_) => MetadataSerializer::Rotations,
            Self::BlockPos(_) => MetadataSerializer::BlockPos,
            Self::OptionalBlockPos(_) => MetadataSerializer::OptionalBlockPos,
            Self::Direction(_) => MetadataSerializer::Direction,
            Self::OptionalLivingEntityReference(_) => {
                MetadataSerializer::OptionalLivingEntityReference
            }
            Self::BlockState(_) => MetadataSerializer::BlockState,
            Self::OptionalBlockState(_) => MetadataSerializer::OptionalBlockState,
            Self::Particle(_) => MetadataSerializer::Particle,
            Self::Particles(_) => MetadataSerializer::Particles,
            Self::VillagerData(_) => MetadataSerializer::VillagerData,
            Self::OptionalUnsignedInt(_) => MetadataSerializer::OptionalUnsignedInt,
            Self::Pose(_) => MetadataSerializer::Pose,
            Self::Holder { serializer, .. } | Self::EnumState { serializer, .. } => *serializer,
            Self::OptionalGlobalPos(_) => MetadataSerializer::OptionalGlobalPos,
            Self::Vector3(_) => MetadataSerializer::Vector3,
            Self::Quaternion(_) => MetadataSerializer::Quaternion,
            Self::ResolvableProfile(_) => MetadataSerializer::ResolvableProfile,
            Self::HumanoidArm(_) => MetadataSerializer::HumanoidArm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VillagerData {
    pub villager_type: Identifier,
    pub profession: Identifier,
    pub level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalPos {
    pub dimension: Identifier,
    pub position: BlockPos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvableProfile {
    Resolved {
        uuid: u128,
        name: String,
        properties: Vec<ProfileProperty>,
        skin: PlayerSkinPatch,
    },
    Partial {
        name: Option<String>,
        uuid: Option<u128>,
        properties: Vec<ProfileProperty>,
        skin: PlayerSkinPatch,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PlayerSkinPatch {
    pub body: Option<Identifier>,
    pub cape: Option<Identifier>,
    pub elytra: Option<Identifier>,
    pub model: Option<PlayerSkinModel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSkinModel {
    Wide,
    Slim,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HumanoidArm {
    Left,
    Right,
}
