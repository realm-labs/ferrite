use std::fs;
use std::path::{Path, PathBuf};

use ferrite_world::generation::structure::nbt::{NbtValue, decode_compound};
use ferrite_world::generation::structure::template::StructureTemplate;

#[test]
fn modified_utf_and_numeric_tag_types_are_retained() {
    let bytes = [
        10, 0, 0, // unnamed root compound
        8, 0, 1, b's', 0, 4, b'a', 0xc0, 0x80, b'b', // modified UTF string
        1, 0, 1, b'b', 0xff, // byte -1
        2, 0, 1, b'h', 0xff, 0xfe, // short -2
        0,
    ];
    let value = decode_compound(&bytes).unwrap();
    assert_eq!(value["s"], NbtValue::String("a\0b".into()));
    assert_eq!(value["b"], NbtValue::Byte(-1));
    assert_eq!(value["h"], NbtValue::Short(-2));
}

#[test]
fn locked_multi_palette_and_sparse_templates_decode_exactly() {
    let Some(root) = structure_root() else {
        return;
    };
    let city = load(&root.join("ancient_city/city_center/city_center_2.nbt"));
    assert_eq!(city.size, [18, 31, 41]);
    assert_eq!(city.palettes.len(), 1);
    assert_eq!(city.blocks.len(), 7_901);
    assert_eq!(city.entities.len(), 0);
    assert!(city.duplicate_positions().is_empty());
    assert_eq!(city.volume(), 22_878);

    let shipwreck = load(&root.join("shipwreck/with_mast.nbt"));
    assert_eq!(shipwreck.size, [9, 21, 28]);
    assert_eq!(shipwreck.palettes.len(), 8);
    assert_eq!(shipwreck.blocks.len(), 729);
    assert_eq!(shipwreck.entities.len(), 0);
    assert!(shipwreck.duplicate_positions().is_empty());

    let tower = load(&root.join("end_city/tower_base.nbt"));
    assert_eq!(tower.size, [7, 7, 7]);
    assert_eq!(tower.blocks.len(), 202);
    assert_eq!(tower.volume() - tower.blocks.len(), 141);
}

#[test]
fn every_locked_official_structure_template_is_decodable() {
    let Some(root) = structure_root() else {
        return;
    };
    let mut paths = Vec::new();
    collect_nbt(&root, &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 1_212);
    let mut palettes = 0_usize;
    let mut blocks = 0_usize;
    let mut entities = 0_usize;
    for path in paths {
        let template = load(&path);
        palettes += template.palettes.len();
        blocks += template.blocks.len();
        entities += template.entities.len();
    }
    assert!(palettes >= 1_212);
    assert!(blocks > 1_000_000);
    assert!(entities > 0);
}

fn structure_root() -> Option<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/mc-reference/26.2/client-classes/data/minecraft/structure");
    root.is_dir().then_some(root)
}

fn load(path: &Path) -> StructureTemplate {
    StructureTemplate::decode_gzip(&fs::read(path).unwrap())
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn collect_nbt(directory: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_nbt(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "nbt") {
            output.push(path);
        }
    }
}
