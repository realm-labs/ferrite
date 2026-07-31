//! Recursive End-city graph construction and grouped collision commits.

use std::num::NonZeroU32;
use std::sync::atomic::{AtomicBool, Ordering};

use ferrite_foundation::coordinate::BlockPos;

use crate::generation::feature::random::GenerationRandom;
use crate::generation::structure::end_city::{EndCityError, EndCityPiece, EndCityRuntime};
use crate::generation::structure::jigsaw::Rotation;
use crate::generation::structure::processor::{Heightmap, ProcessorWorld};
use crate::generation::structure::template_manager::TemplateSource;

static SHIP_CREATED: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Generator {
    House,
    Tower,
    Bridge,
    FatTower,
}

pub fn end_city_start_anchor(
    world: &mut impl ProcessorWorld,
    chunk_minimum: BlockPos,
    random: &mut impl GenerationRandom,
) -> Option<(BlockPos, Rotation)> {
    let rotation = Rotation::ALL[bounded(random, 4) as usize];
    let (dx, dz) = match rotation {
        Rotation::None => (5, 5),
        Rotation::Clockwise90 => (-5, 5),
        Rotation::Clockwise180 => (-5, -5),
        Rotation::CounterClockwise90 => (5, -5),
    };
    let x = chunk_minimum.x.wrapping_add(7);
    let z = chunk_minimum.z.wrapping_add(7);
    let height = [
        (x, z),
        (x.wrapping_add(dx), z),
        (x, z.wrapping_add(dz)),
        (x.wrapping_add(dx), z.wrapping_add(dz)),
    ]
    .map(|(x, z)| world.height(Heightmap::WorldSurfaceWorldgen, x, z))
    .into_iter()
    .min()
    .expect("four End-city height probes");
    (height >= 60).then_some((BlockPos::new(x, height, z), rotation))
}

pub fn generate_end_city<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    anchor: BlockPos,
    rotation: Rotation,
    random: &mut impl GenerationRandom,
) -> Result<Vec<EndCityPiece>, EndCityError>
where
    S: TemplateSource,
{
    SHIP_CREATED.store(false, Ordering::Relaxed);
    let mut retained = Vec::new();
    let base = runtime.create_piece("base_floor", anchor, rotation, true)?;
    retained.push(base.clone());
    let second = runtime.connect_piece(
        &base,
        "second_floor_1",
        BlockPos::new(-1, 0, -1),
        rotation,
        false,
    )?;
    retained.push(second.clone());
    let third = runtime.connect_piece(
        &second,
        "third_floor_1",
        BlockPos::new(-1, 4, -1),
        rotation,
        false,
    )?;
    retained.push(third.clone());
    let roof = runtime.connect_piece(
        &third,
        "third_roof",
        BlockPos::new(-1, 8, -1),
        rotation,
        true,
    )?;
    retained.push(roof.clone());
    let _ = recursive(
        runtime,
        Generator::Tower,
        1,
        &roof,
        None,
        &mut retained,
        random,
    )?;
    Ok(retained)
}

fn recursive<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    generator: Generator,
    depth: i32,
    parent: &EndCityPiece,
    offset: Option<BlockPos>,
    retained: &mut Vec<EndCityPiece>,
    random: &mut impl GenerationRandom,
) -> Result<bool, EndCityError>
where
    S: TemplateSource,
{
    if depth > 8 {
        return Ok(false);
    }
    let mut candidates = Vec::new();
    let generated = match generator {
        Generator::House => generate_house(
            runtime,
            depth,
            parent,
            offset.unwrap_or(BlockPos::new(0, 0, 0)),
            &mut candidates,
            random,
        )?,
        Generator::Tower => generate_tower(runtime, depth, parent, &mut candidates, random)?,
        Generator::Bridge => generate_bridge(runtime, depth, parent, &mut candidates, random)?,
        Generator::FatTower => generate_fat_tower(runtime, depth, parent, &mut candidates, random)?,
    };
    if !generated {
        return Ok(false);
    }
    let tag = random.next_i32();
    for candidate in &mut candidates {
        candidate.generation_depth = tag;
        if let Some(collider) = retained
            .iter()
            .find(|piece| piece.bounding_box.intersects(candidate.bounding_box))
            && collider.generation_depth != parent.generation_depth
        {
            return Ok(false);
        }
    }
    retained.extend(candidates);
    Ok(true)
}

fn generate_house<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    depth: i32,
    parent: &EndCityPiece,
    offset: BlockPos,
    output: &mut Vec<EndCityPiece>,
    random: &mut impl GenerationRandom,
) -> Result<bool, EndCityError>
where
    S: TemplateSource,
{
    let base = add(
        runtime,
        output,
        parent,
        "base_floor",
        offset,
        Rotation::None,
        true,
    )?;
    match bounded(random, 3) {
        0 => {
            add(
                runtime,
                output,
                &base,
                "base_roof",
                BlockPos::new(-1, 4, -1),
                Rotation::None,
                true,
            )?;
        }
        1 => {
            let second = add(
                runtime,
                output,
                &base,
                "second_floor_2",
                BlockPos::new(-1, 0, -1),
                Rotation::None,
                false,
            )?;
            let roof = add(
                runtime,
                output,
                &second,
                "second_roof",
                BlockPos::new(-1, 8, -1),
                Rotation::None,
                false,
            )?;
            let _ = recursive(
                runtime,
                Generator::Tower,
                depth + 1,
                &roof,
                None,
                output,
                random,
            )?;
        }
        _ => {
            let second = add(
                runtime,
                output,
                &base,
                "second_floor_2",
                BlockPos::new(-1, 0, -1),
                Rotation::None,
                false,
            )?;
            let third = add(
                runtime,
                output,
                &second,
                "third_floor_2",
                BlockPos::new(-1, 4, -1),
                Rotation::None,
                false,
            )?;
            let roof = add(
                runtime,
                output,
                &third,
                "third_roof",
                BlockPos::new(-1, 8, -1),
                Rotation::None,
                true,
            )?;
            let _ = recursive(
                runtime,
                Generator::Tower,
                depth + 1,
                &roof,
                None,
                output,
                random,
            )?;
        }
    }
    Ok(true)
}

fn generate_tower<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    depth: i32,
    parent: &EndCityPiece,
    output: &mut Vec<EndCityPiece>,
    random: &mut impl GenerationRandom,
) -> Result<bool, EndCityError>
where
    S: TemplateSource,
{
    let dx = bounded(random, 2) as i32;
    let dz = bounded(random, 2) as i32;
    let base = add(
        runtime,
        output,
        parent,
        "tower_base",
        BlockPos::new(3 + dx, -3, 3 + dz),
        Rotation::None,
        true,
    )?;
    let mut latest = add(
        runtime,
        output,
        &base,
        "tower_piece",
        BlockPos::new(0, 7, 0),
        Rotation::None,
        true,
    )?;
    let mut bridge_level = (bounded(random, 3) == 0).then_some(latest.clone());
    let levels = 1 + bounded(random, 3);
    for level in 0..levels {
        latest = add(
            runtime,
            output,
            &latest,
            "tower_piece",
            BlockPos::new(0, 4, 0),
            Rotation::None,
            true,
        )?;
        if level + 1 < levels && random.next_bool() {
            bridge_level = Some(latest.clone());
        }
    }
    if let Some(level) = bridge_level {
        for (extra_rotation, offset) in bridge_specs(false) {
            if random.next_bool() {
                let bridge = add(
                    runtime,
                    output,
                    &level,
                    "bridge_end",
                    offset,
                    extra_rotation,
                    true,
                )?;
                let _ = recursive(
                    runtime,
                    Generator::Bridge,
                    depth + 1,
                    &bridge,
                    None,
                    output,
                    random,
                )?;
            }
        }
        add(
            runtime,
            output,
            &latest,
            "tower_top",
            BlockPos::new(-1, 4, -1),
            Rotation::None,
            true,
        )?;
        return Ok(true);
    }
    if depth == 7 {
        add(
            runtime,
            output,
            &latest,
            "tower_top",
            BlockPos::new(-1, 4, -1),
            Rotation::None,
            true,
        )?;
        Ok(true)
    } else {
        recursive(
            runtime,
            Generator::FatTower,
            depth + 1,
            &latest,
            None,
            output,
            random,
        )
    }
}

fn generate_bridge<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    depth: i32,
    parent: &EndCityPiece,
    output: &mut Vec<EndCityPiece>,
    random: &mut impl GenerationRandom,
) -> Result<bool, EndCityError>
where
    S: TemplateSource,
{
    let length = 1 + bounded(random, 4);
    let mut latest = add(
        runtime,
        output,
        parent,
        "bridge_piece",
        BlockPos::new(0, 0, -4),
        Rotation::None,
        true,
    )?;
    latest.generation_depth = -1;
    output
        .last_mut()
        .expect("bridge start was just appended")
        .generation_depth = -1;
    let mut next_y = 0;
    for _ in 0..length {
        if random.next_bool() {
            latest = add(
                runtime,
                output,
                &latest,
                "bridge_piece",
                BlockPos::new(0, next_y, -4),
                Rotation::None,
                true,
            )?;
            next_y = 0;
        } else if random.next_bool() {
            latest = add(
                runtime,
                output,
                &latest,
                "bridge_steep_stairs",
                BlockPos::new(0, next_y, -4),
                Rotation::None,
                true,
            )?;
            next_y = 4;
        } else {
            latest = add(
                runtime,
                output,
                &latest,
                "bridge_gentle_stairs",
                BlockPos::new(0, next_y, -8),
                Rotation::None,
                true,
            )?;
            next_y = 4;
        }
    }
    if !SHIP_CREATED.load(Ordering::Relaxed) && bounded(random, (10 - depth) as u32) == 0 {
        add(
            runtime,
            output,
            &latest,
            "ship",
            BlockPos::new(
                -8 + bounded(random, 8) as i32,
                next_y,
                -70 + bounded(random, 10) as i32,
            ),
            Rotation::None,
            true,
        )?;
        SHIP_CREATED.store(true, Ordering::Relaxed);
    } else if !recursive(
        runtime,
        Generator::House,
        depth + 1,
        &latest,
        Some(BlockPos::new(-3, next_y + 1, -11)),
        output,
        random,
    )? {
        return Ok(false);
    }
    let mut end = add(
        runtime,
        output,
        &latest,
        "bridge_end",
        BlockPos::new(4, next_y, 0),
        Rotation::Clockwise180,
        true,
    )?;
    end.generation_depth = -1;
    output
        .last_mut()
        .expect("bridge end was just appended")
        .generation_depth = -1;
    Ok(true)
}

fn generate_fat_tower<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    depth: i32,
    parent: &EndCityPiece,
    output: &mut Vec<EndCityPiece>,
    random: &mut impl GenerationRandom,
) -> Result<bool, EndCityError>
where
    S: TemplateSource,
{
    let base = add(
        runtime,
        output,
        parent,
        "fat_tower_base",
        BlockPos::new(-3, 4, -3),
        Rotation::None,
        true,
    )?;
    let mut latest = add(
        runtime,
        output,
        &base,
        "fat_tower_middle",
        BlockPos::new(0, 4, 0),
        Rotation::None,
        true,
    )?;
    for _ in 0..2 {
        if bounded(random, 3) == 0 {
            break;
        }
        latest = add(
            runtime,
            output,
            &latest,
            "fat_tower_middle",
            BlockPos::new(0, 8, 0),
            Rotation::None,
            true,
        )?;
        for (extra_rotation, offset) in bridge_specs(true) {
            if random.next_bool() {
                let bridge = add(
                    runtime,
                    output,
                    &latest,
                    "bridge_end",
                    offset,
                    extra_rotation,
                    true,
                )?;
                let _ = recursive(
                    runtime,
                    Generator::Bridge,
                    depth + 1,
                    &bridge,
                    None,
                    output,
                    random,
                )?;
            }
        }
    }
    add(
        runtime,
        output,
        &latest,
        "fat_tower_top",
        BlockPos::new(-2, 8, -2),
        Rotation::None,
        true,
    )?;
    Ok(true)
}

fn bridge_specs(fat: bool) -> [(Rotation, BlockPos); 4] {
    if fat {
        [
            (Rotation::None, BlockPos::new(4, -1, 0)),
            (Rotation::Clockwise90, BlockPos::new(12, -1, 4)),
            (Rotation::CounterClockwise90, BlockPos::new(0, -1, 8)),
            (Rotation::Clockwise180, BlockPos::new(8, -1, 12)),
        ]
    } else {
        [
            (Rotation::None, BlockPos::new(1, -1, 0)),
            (Rotation::Clockwise90, BlockPos::new(6, -1, 1)),
            (Rotation::CounterClockwise90, BlockPos::new(0, -1, 5)),
            (Rotation::Clockwise180, BlockPos::new(5, -1, 6)),
        ]
    }
}

fn add<S>(
    runtime: &mut EndCityRuntime<'_, S>,
    output: &mut Vec<EndCityPiece>,
    parent: &EndCityPiece,
    name: &str,
    offset: BlockPos,
    extra_rotation: Rotation,
    overwrite: bool,
) -> Result<EndCityPiece, EndCityError>
where
    S: TemplateSource,
{
    let rotation = compose_rotation(parent.rotation, extra_rotation);
    let piece = runtime.connect_piece(parent, name, offset, rotation, overwrite)?;
    output.push(piece.clone());
    Ok(piece)
}

fn compose_rotation(left: Rotation, right: Rotation) -> Rotation {
    let left = rotation_index(left);
    let right = rotation_index(right);
    Rotation::ALL[(left + right) % 4]
}

const fn rotation_index(rotation: Rotation) -> usize {
    match rotation {
        Rotation::None => 0,
        Rotation::Clockwise90 => 1,
        Rotation::Clockwise180 => 2,
        Rotation::CounterClockwise90 => 3,
    }
}

fn bounded(random: &mut impl GenerationRandom, bound: u32) -> u32 {
    random.next_u32(NonZeroU32::new(bound).expect("positive End-city bound"))
}
