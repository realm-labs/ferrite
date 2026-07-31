use std::collections::BTreeMap;

use ferrite_foundation::coordinate::BlockPos;
use ferrite_world::generation::flat::{
    FLAT_PRESET_PARTITION_SHA256, FlatFillWorld, FlatHeightmap, FlatLayer, FlatSettings,
    FlatShareResolver, StructureOverrides, export_flat_share, fill_flat_chunk, flat_base_column,
    flat_base_height, parse_flat_share,
};
use ferrite_world::id::{BiomeId, BlockStateId};

#[test]
fn void_is_computed_before_nonopaque_layers_become_decoration_features() {
    let settings = settings::<()>(vec![FlatLayer {
        height: 1,
        state: AIR,
    }]);

    let prepared = settings
        .prepare_layers(|state| state == AIR, |state| state == STONE)
        .unwrap();

    assert!(prepared.void);
    assert_eq!(prepared.base_layers, [None]);
    assert_eq!(prepared.decoration_layers, [(0, AIR)]);
}

#[test]
fn base_fill_uses_offset_x_z_order_and_updates_both_heightmaps() {
    let mut world = FillFixture::default();
    fill_flat_chunk(&mut world, -16, 32, &[Some(STONE)]).unwrap();

    assert_eq!(world.offers.len(), 256);
    assert_eq!(world.offers[0].0, BlockPos::new(-16, -64, 32));
    assert_eq!(world.offers[1].0, BlockPos::new(-16, -64, 33));
    assert_eq!(world.offers[16].0, BlockPos::new(-15, -64, 32));
    assert_eq!(
        &world.heightmaps[..2],
        &[
            (
                FlatHeightmap::OceanFloorWorldGeneration,
                BlockPos::new(-16, -64, 32)
            ),
            (
                FlatHeightmap::WorldSurfaceWorldGeneration,
                BlockPos::new(-16, -64, 32)
            ),
        ]
    );
}

#[test]
fn parser_resolves_blocks_after_capacity_and_falls_back_on_late_unknown() {
    let resolver = Resolver::new();
    let selected = settings::<()>(vec![FlatLayer {
        height: 1,
        state: STONE,
    }]);
    let fallback = settings::<()>(vec![FlatLayer {
        height: 1,
        state: BEDROCK,
    }]);

    let parsed = parse_flat_share(
        "4064*minecraft:stone,1*minecraft:missing;minecraft:desert",
        selected,
        &fallback,
        PLAINS,
        &resolver,
    );

    assert_eq!(parsed, fallback);
}

#[test]
fn valid_share_preserves_flags_and_overrides_and_exports_height_one_without_prefix() {
    let resolver = Resolver::new();
    let selected = FlatSettings {
        structure_overrides: StructureOverrides::Present(vec!["village"]),
        layers: Vec::new(),
        lakes: true,
        features: true,
        biome: PLAINS,
    };
    let fallback = settings::<&str>(Vec::new());
    let parsed = parse_flat_share(
        "minecraft:bedrock,2*minecraft:stone;minecraft:desert",
        selected,
        &fallback,
        PLAINS,
        &resolver,
    );

    assert!(parsed.lakes);
    assert!(parsed.features);
    assert_eq!(
        parsed.structure_overrides,
        StructureOverrides::Present(vec!["village"])
    );
    assert_eq!(
        export_flat_share(&parsed, &resolver).unwrap(),
        "minecraft:bedrock,2*minecraft:stone;minecraft:desert"
    );
    assert_eq!(FLAT_PRESET_PARTITION_SHA256.len(), 64);
    assert_eq!(
        flat_base_height(-64, 10, &[Some(STONE), None], |state| state == STONE),
        -63
    );
    assert_eq!(flat_base_column(2, &[Some(STONE), None], AIR), [STONE, AIR]);
}

fn settings<T>(layers: Vec<FlatLayer>) -> FlatSettings<T> {
    FlatSettings {
        structure_overrides: StructureOverrides::Absent,
        layers,
        lakes: false,
        features: false,
        biome: PLAINS,
    }
}

const AIR: BlockStateId = BlockStateId::new(0);
const STONE: BlockStateId = BlockStateId::new(1);
const BEDROCK: BlockStateId = BlockStateId::new(2);
const PLAINS: BiomeId = BiomeId::new(1);
const DESERT: BiomeId = BiomeId::new(2);

#[derive(Debug, Default)]
struct FillFixture {
    offers: Vec<(BlockPos, BlockStateId)>,
    heightmaps: Vec<(FlatHeightmap, BlockPos)>,
}

impl FlatFillWorld for FillFixture {
    fn minimum_y(&self) -> i32 {
        -64
    }

    fn height(&self) -> usize {
        384
    }

    fn offer_flat_block(&mut self, position: BlockPos, state: BlockStateId) {
        self.offers.push((position, state));
    }

    fn update_heightmap(
        &mut self,
        heightmap: FlatHeightmap,
        position: BlockPos,
        _state: BlockStateId,
    ) {
        self.heightmaps.push((heightmap, position));
    }
}

#[derive(Debug)]
struct Resolver {
    blocks: BTreeMap<&'static str, BlockStateId>,
    block_names: BTreeMap<BlockStateId, &'static str>,
    biomes: BTreeMap<&'static str, BiomeId>,
    biome_names: BTreeMap<BiomeId, &'static str>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            blocks: BTreeMap::from([
                ("minecraft:air", AIR),
                ("minecraft:stone", STONE),
                ("minecraft:bedrock", BEDROCK),
            ]),
            block_names: BTreeMap::from([
                (AIR, "minecraft:air"),
                (STONE, "minecraft:stone"),
                (BEDROCK, "minecraft:bedrock"),
            ]),
            biomes: BTreeMap::from([("minecraft:plains", PLAINS), ("minecraft:desert", DESERT)]),
            biome_names: BTreeMap::from([
                (PLAINS, "minecraft:plains"),
                (DESERT, "minecraft:desert"),
            ]),
        }
    }
}

impl FlatShareResolver for Resolver {
    fn block_state(&self, identifier: &str) -> Option<BlockStateId> {
        self.blocks.get(identifier).copied()
    }

    fn block_identifier(&self, state: BlockStateId) -> Option<&str> {
        self.block_names.get(&state).copied()
    }

    fn biome(&self, identifier: &str) -> Option<BiomeId> {
        self.biomes.get(identifier).copied()
    }

    fn biome_identifier(&self, biome: BiomeId) -> Option<&str> {
        self.biome_names.get(&biome).copied()
    }
}
