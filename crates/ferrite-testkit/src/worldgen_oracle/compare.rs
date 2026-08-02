use serde::{Deserialize, Serialize};

use crate::worldgen_oracle::model::{CanonicalNbt, SemanticChunk, SemanticSection};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDivergence {
    pub stage: String,
    pub field: String,
    pub coordinate: Option<[i32; 3]>,
    pub official: String,
    pub ferrite: String,
}

pub fn compare_chunks(
    official: &SemanticChunk,
    ferrite: &SemanticChunk,
) -> Result<(), SemanticDivergence> {
    scalar("input", "schema", &official.schema, &ferrite.schema)?;
    scalar(
        "input",
        "reference_version",
        &official.reference_version,
        &ferrite.reference_version,
    )?;
    scalar(
        "input",
        "dimension",
        &official.dimension,
        &ferrite.dimension,
    )?;
    scalar("input", "position", &official.position, &ferrite.position)?;
    scalar(
        "input",
        "data_version",
        &official.data_version,
        &ferrite.data_version,
    )?;

    nbt(
        "structure_starts",
        "structure_starts",
        &official.structure_starts,
        &ferrite.structure_starts,
    )?;
    nbt(
        "structure_references",
        "structure_references",
        &official.structure_references,
        &ferrite.structure_references,
    )?;
    compare_section_field(official, ferrite, SectionField::Biomes)?;
    compare_section_field(official, ferrite, SectionField::Blocks)?;
    compare_section_field(official, ferrite, SectionField::Fluids)?;
    compare_heightmaps(official, ferrite)?;
    nbt(
        "carvers",
        "post_processing",
        &official.post_processing,
        &ferrite.post_processing,
    )?;
    scalar(
        "features",
        "block_entities",
        &official.block_entities,
        &ferrite.block_entities,
    )?;
    nbt(
        "noise_or_later",
        "scheduled_block_ticks",
        &official.scheduled_block_ticks,
        &ferrite.scheduled_block_ticks,
    )?;
    nbt(
        "noise_or_later",
        "scheduled_fluid_ticks",
        &official.scheduled_fluid_ticks,
        &ferrite.scheduled_fluid_ticks,
    )?;
    compare_section_field(official, ferrite, SectionField::SkyLight)?;
    compare_section_field(official, ferrite, SectionField::BlockLight)?;
    scalar(
        "initialize_light",
        "light_initialized",
        &official.light_initialized,
        &ferrite.light_initialized,
    )?;
    scalar(
        "full",
        "inhabited_time",
        &official.inhabited_time,
        &ferrite.inhabited_time,
    )?;
    scalar(
        "empty_or_later",
        "generation_metadata",
        &official.generation_metadata,
        &ferrite.generation_metadata,
    )?;
    scalar("status", "chunk_status", &official.status, &ferrite.status)
}

fn compare_section_field(
    official: &SemanticChunk,
    ferrite: &SemanticChunk,
    field: SectionField,
) -> Result<(), SemanticDivergence> {
    if official.sections.len() != ferrite.sections.len() {
        return Err(divergence(
            field.stage(),
            "sections",
            None,
            &official.sections.len(),
            &ferrite.sections.len(),
        ));
    }
    for (official_section, ferrite_section) in official.sections.iter().zip(&ferrite.sections) {
        if official_section.y != ferrite_section.y {
            return Err(divergence(
                field.stage(),
                "section_y",
                None,
                &official_section.y,
                &ferrite_section.y,
            ));
        }
        match field {
            SectionField::Blocks => compare_cells(
                official,
                official_section,
                &official_section.block_states,
                &ferrite_section.block_states,
                "block_states",
                false,
            )?,
            SectionField::Fluids => compare_cells(
                official,
                official_section,
                &official_section.fluid_states,
                &ferrite_section.fluid_states,
                "fluid_states",
                false,
            )?,
            SectionField::Biomes => compare_cells(
                official,
                official_section,
                &official_section.biomes,
                &ferrite_section.biomes,
                "biomes",
                true,
            )?,
            SectionField::SkyLight => compare_light(
                official,
                official_section,
                official_section.sky_light.as_deref(),
                ferrite_section.sky_light.as_deref(),
                "sky_light",
            )?,
            SectionField::BlockLight => compare_light(
                official,
                official_section,
                official_section.block_light.as_deref(),
                ferrite_section.block_light.as_deref(),
                "block_light",
            )?,
        }
    }
    Ok(())
}

fn compare_cells(
    chunk: &SemanticChunk,
    section: &SemanticSection,
    official: &[String],
    ferrite: &[String],
    field: &'static str,
    quart: bool,
) -> Result<(), SemanticDivergence> {
    let stage = if quart { "biomes" } else { "noise_or_later" };
    if official.len() != ferrite.len() {
        return Err(divergence(
            stage,
            field,
            None,
            &official.len(),
            &ferrite.len(),
        ));
    }
    let Some(index) = official
        .iter()
        .zip(ferrite)
        .position(|(left, right)| left != right)
    else {
        return Ok(());
    };
    let coordinate = if quart {
        let x = index % 4;
        let z = (index / 4) % 4;
        let y = index / 16;
        [
            chunk.position.x * 16 + x as i32 * 4,
            section.y * 16 + y as i32 * 4,
            chunk.position.z * 16 + z as i32 * 4,
        ]
    } else {
        let x = index % 16;
        let z = (index / 16) % 16;
        let y = index / 256;
        [
            chunk.position.x * 16 + x as i32,
            section.y * 16 + y as i32,
            chunk.position.z * 16 + z as i32,
        ]
    };
    Err(divergence(
        stage,
        field,
        Some(coordinate),
        &official[index],
        &ferrite[index],
    ))
}

fn compare_light(
    chunk: &SemanticChunk,
    section: &SemanticSection,
    official: Option<&[u8]>,
    ferrite: Option<&[u8]>,
    field: &'static str,
) -> Result<(), SemanticDivergence> {
    if official == ferrite {
        return Ok(());
    }
    let index = official
        .unwrap_or_default()
        .iter()
        .zip(ferrite.unwrap_or_default())
        .position(|(left, right)| left != right);
    let coordinate = index.map(|index| {
        let nibble = index * 2;
        let x = nibble % 16;
        let z = (nibble / 16) % 16;
        let y = nibble / 256;
        [
            chunk.position.x * 16 + x as i32,
            section.y * 16 + y as i32,
            chunk.position.z * 16 + z as i32,
        ]
    });
    Err(divergence(
        "light",
        field,
        coordinate,
        &official.map(blake3::hash),
        &ferrite.map(blake3::hash),
    ))
}

fn compare_heightmaps(
    official: &SemanticChunk,
    ferrite: &SemanticChunk,
) -> Result<(), SemanticDivergence> {
    if official.heightmaps.keys().ne(ferrite.heightmaps.keys()) {
        return Err(divergence(
            "noise_or_later",
            "heightmap_kinds",
            None,
            &official.heightmaps.keys().collect::<Vec<_>>(),
            &ferrite.heightmaps.keys().collect::<Vec<_>>(),
        ));
    }
    for (kind, official_values) in &official.heightmaps {
        let ferrite_values = &ferrite.heightmaps[kind];
        if let Some(index) = official_values
            .iter()
            .zip(ferrite_values)
            .position(|(left, right)| left != right)
        {
            let x = index % 16;
            let z = index / 16;
            return Err(divergence(
                "noise_or_later",
                &format!("heightmaps.{kind}"),
                Some([
                    official.position.x * 16 + x as i32,
                    official_values[index],
                    official.position.z * 16 + z as i32,
                ]),
                &official_values[index],
                &ferrite_values[index],
            ));
        }
    }
    Ok(())
}

fn nbt(
    stage: &'static str,
    field: &'static str,
    official: &CanonicalNbt,
    ferrite: &CanonicalNbt,
) -> Result<(), SemanticDivergence> {
    scalar(stage, field, official, ferrite)
}

fn scalar<T: std::fmt::Debug + PartialEq>(
    stage: &'static str,
    field: &str,
    official: &T,
    ferrite: &T,
) -> Result<(), SemanticDivergence> {
    if official == ferrite {
        Ok(())
    } else {
        Err(divergence(stage, field, None, official, ferrite))
    }
}

fn divergence(
    stage: &str,
    field: &str,
    coordinate: Option<[i32; 3]>,
    official: &impl std::fmt::Debug,
    ferrite: &impl std::fmt::Debug,
) -> SemanticDivergence {
    SemanticDivergence {
        stage: stage.to_owned(),
        field: field.to_owned(),
        coordinate,
        official: bounded_debug(official),
        ferrite: bounded_debug(ferrite),
    }
}

fn bounded_debug(value: &impl std::fmt::Debug) -> String {
    let mut text = format!("{value:?}");
    if text.len() > 512 {
        text.truncate(509);
        text.push_str("...");
    }
    text
}

#[derive(Debug, Clone, Copy)]
enum SectionField {
    Blocks,
    Fluids,
    Biomes,
    SkyLight,
    BlockLight,
}

impl SectionField {
    const fn stage(self) -> &'static str {
        match self {
            Self::Blocks | Self::Fluids => "noise_or_later",
            Self::Biomes => "biomes",
            Self::SkyLight | Self::BlockLight => "light",
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
    use crate::worldgen_oracle::model::{NORMALIZATION_SCHEMA, SemanticSource};

    fn chunk(source: SemanticSource) -> SemanticChunk {
        SemanticChunk {
            schema: NORMALIZATION_SCHEMA.to_owned(),
            source,
            reference_version: "26.2".to_owned(),
            data_version: 1,
            dimension: "minecraft:overworld".to_owned(),
            position: crate::worldgen_oracle::model::ChunkCoordinate { x: -1, z: 2 },
            status: "minecraft:full".to_owned(),
            sections: vec![SemanticSection {
                y: -4,
                block_states: vec!["minecraft:air".to_owned(); 4_096],
                fluid_states: vec!["minecraft:empty".to_owned(); 4_096],
                biomes: vec!["minecraft:plains".to_owned(); 64],
                sky_light: Some(vec![0xff; 2_048]),
                block_light: None,
            }],
            heightmaps: BTreeMap::from([("WORLD_SURFACE".to_owned(), vec![-64; 256])]),
            block_entities: Vec::new(),
            post_processing: CanonicalNbt::empty_list(),
            structure_starts: CanonicalNbt::empty_compound(),
            structure_references: CanonicalNbt::empty_compound(),
            scheduled_block_ticks: CanonicalNbt::empty_list(),
            scheduled_fluid_ticks: CanonicalNbt::empty_list(),
            light_initialized: true,
            inhabited_time: 0,
            generation_metadata: BTreeMap::new(),
        }
    }

    #[test]
    fn source_identity_is_not_part_of_the_semantic_denominator() {
        let official = chunk(SemanticSource::OfficialMinecraft26_2);
        let ferrite = chunk(SemanticSource::Ferrite);
        assert_eq!(compare_chunks(&official, &ferrite), Ok(()));
        assert_eq!(official.canonical_digest(), ferrite.canonical_digest());
    }

    #[test]
    fn reports_the_first_block_coordinate() {
        let official = chunk(SemanticSource::OfficialMinecraft26_2);
        let mut ferrite = chunk(SemanticSource::Ferrite);
        ferrite.sections[0].block_states[273] = "minecraft:stone".to_owned();
        let divergence = compare_chunks(&official, &ferrite).unwrap_err();
        assert_eq!(divergence.stage, "noise_or_later");
        assert_eq!(divergence.field, "block_states");
        assert_eq!(divergence.coordinate, Some([-15, -63, 33]));
    }
}
