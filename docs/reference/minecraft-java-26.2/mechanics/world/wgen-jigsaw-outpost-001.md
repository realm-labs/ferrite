# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-OUTPOST-001` — Outpost legacy elements suppress air, overlay sparse decay, and finalize captive mobs

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the four locked pillager-outpost pools, one rot processor list, 11 present NBT
templates and one chest loot record fix the complete outpost payload supplied to generic
jigsaw/template placement. Every locator exists and is reachable. Legacy-single air filtering makes
the full-cuboid raw corpus nondestructive; exact material writes, NBT and raw entities are specified
here. Village payloads are owned by `WGEN-JIGSAW-VILLAGES-001`.

**Applies when:**

`pillager_outpost` starts with its virtual base plate, expands a tower/feature plate/feature
connector, places the two-child tower list or a legacy single, writes a tower chest/banner, or
creates a cage iron golem/allay. The record-owned pillager natural-spawn override is separate from
these raw payloads.

**Authoritative state:**

Record/core inputs; four ordered pool records; legacy/list element projection and processor state;
11 full-cuboid template palettes, connector/NBT/entity fields; position-derived rot randomness;
chunk clip and live block/fluid/entity state; caller structure RNG for chest seeds; level RNG for
mob finalization; the pillager-outpost loot record.

**Transition and ordering:**

All four pools fall back to `minecraft:empty`. `base_plates` and `feature_plates` each contain one
empty-processor legacy single, rigid and terrain-matching respectively. `features` expands seven
equal rigid legacy templates followed by Empty weight `6`, for total weight `13`. `towers` contains
one rigid list: empty-processor `watchtower` first and `watchtower_overgrown` with `outpost_rot`
second. Thus the four pools have 11 top-level entries and expanded weight `16`: nine legacy singles,
one list and one Empty. Counting list children, all 11 present template locators are named exactly
once; none is missing or unreferenced.

The list exposes only `watchtower` connectors, uses the `15×23×15` union/max extent, and places both
children at one origin in order. Its rigid projection is propagated to both children. Every legacy
child starts with generic single settings, then replaces the initial structure-block ignore with
`STRUCTURE_AND_AIR` at the end of the processor chain. Natural placement therefore runs jigsaw
replacement, configured processors and optional projection gravity before rejecting resulting air.
All 55 connectors have final state air and are discarded along with 17,984 raw air cells; none
writes air. With `keepJigsaws`, the separately owned operator path retains jigsaws because they are
not air.

`base_plate` contains only 7,675 air plus five jigsaws, and `feature_plate` only 2,032 air plus 16
jigsaws. They consequently place no block or entity, but retain their real boxes/connectors for
graph discovery, terrain alignment, expansion and saved-piece/junction semantics. Feature-plate
gravity runs before air filtering, so it can align child topology without leaving material.

The tower's first child offers 1,155 nonair/nonconnector cells. The second child's block-rot has no
rottable filter: it obtains position-derived randomness for every one of its 5,175 processed cells,
retains on `nextFloat() <= 0.05`, and runs before legacy air rejection. At most its 1,302
nonair/nonconnector cells can therefore add or overwrite material; retained air and air-final
jigsaws are still rejected. The selection is stable for world position and rotation and consumes no
caller structure RNG. A rejected second-child cell leaves the first child's result unchanged.

**Locked payload census:**

All 11 templates have one palette, cover every coordinate of their 20,741 combined bounding volume
and have no duplicate coordinates, structure void or structure block. They use 25 block IDs and 77
exact states, with 17,984 air, 55 jigsaws, 18 non-jigsaw block-NBT cells and three raw entities.
After natural jigsaw replacement/legacy filtering, 2,702 raw cells are eligible before the overgrown
rot gate:

| Template under `pillager_outpost/` | Size | Cells | Air | Jigsaws | Eligible material | Other NBT | Entities |
|---|---:|---:|---:|---:|---:|---:|---:|
| `base_plate` | `16×30×16` | 7,680 | 7,675 | 5 | 0 | 0 | 0 |
| `feature_cage1` | `7×4×7` | 196 | 143 | 5 | 48 | 0 | 1 |
| `feature_cage2` | `7×4×7` | 196 | 143 | 5 | 48 | 0 | 0 |
| `feature_cage_with_allays` | `7×4×7` | 196 | 139 | 5 | 52 | 0 | 2 |
| `feature_logs` | `6×3×7` | 126 | 101 | 4 | 21 | 0 | 0 |
| `feature_plate` | `16×4×32` | 2,048 | 2,032 | 16 | 0 | 0 | 0 |
| `feature_targets` | `3×3×7` | 63 | 48 | 5 | 10 | 0 | 0 |
| `feature_tent1` | `6×4×7` | 168 | 133 | 4 | 31 | 0 | 0 |
| `feature_tent2` | `6×4×7` | 168 | 129 | 4 | 35 | 0 | 0 |
| `watchtower` | `15×21×15` | 4,725 | 3,569 | 1 | 1,155 | 9 | 0 |
| `watchtower_overgrown` | `15×23×15` | 5,175 | 3,872 | 1 | 1,302 | 9 | 0 |

Besides air/jigsaw, raw counts are dark-oak planks `694`, birch planks `580`, dark-oak logs `345`,
cobblestone and mossy cobblestone `207` each, dark-oak fence `198`, vines `147`, dark-oak slabs
`102`, dark-oak stairs `56`, white wool `40`, cobblestone and mossy stairs `33` each, white wall
banners `16`, torches `8`, both cobblestone walls `8` each, both cobblestone slabs `4` each,
pumpkins `4`, and two each of carved pumpkin, chest, crafting table and hay block.

**Connector payload:**

All 55 priorities are zero; eight connectors are aligned and 47 rollable. Pool fields are empty
`35`, features `15`, feature plates `3` and towers `2`. Names and targets are identical
distributions: feature `47`, entrance `4`, plate-entry `4`. Every final state is air, so natural
placement removes every connector rather than writing an air block.

**NBT, overlay and loot:**

Each tower child has the same eight ominous wall banners and one west-facing chest at the same local
coordinates. Every banner fixes the eight-pattern cyan/light-gray/gray/black ominous design,
translated ominous-banner item name and uncommon rarity. The first child normally writes all nine
NBT cells. Each second-child counterpart is independently subject to the `0.05` rot gate; if
retained, the generic barrier/state/NBT transaction reloads the same position after the first child.
A retained second chest consumes a second caller `nextLong` and overwrites the first chest and its
loot seed; rejection preserves the first seed. Clip, rot, block write and resulting block-entity
type independently gate every load.

The `chests/pillager_outpost` named-sequence record has six pools: crossbow with uniform `0..1`
rolls; weighted wheat/potato/carrot with uniform `2..3`; dark-oak logs with uniform `1..3`; six
utility entries with uniform `2..3`; regular goat horn with uniform `0..1`; and one roll over Empty
weight `3` versus a weight-one entry that sets sentry-trim count `2`. Exact weights, count ranges,
enchantment and instrument functions are data-only inputs to `ITM-LOOT-001`.

**Raw entities:**

`feature_cage1` stores one health-`100` non-player-created iron golem; `feature_cage_with_allays`
stores two health-`10` empty-handed persistent allays with distinct fractional positions, motion and
rotations. All three records already carry a `minecraft:random_spawn_bonus` follow-range modifier:
golem `+0.0026721257350880157`, allays `-0.032921064749670284` and `+0.00745067659310455`. Entity
placement transforms the integer clip position and fractional position, removes UUID, creates with
STRUCTURE reason, finalizes and adds with passengers. Neither concrete type overrides finalization,
so `Mob#finalizeSpawn` sees the existing modifier and skips a new triangle draw, then consumes one
level-RNG float to replace left-handed state. Golems and allays both override far-distance removal
to false; no raw pillager exists in these templates.

**Branches and aborts:**

Four primary/fallback pools; seven features versus expanded Empty; terrain-matching virtual plate;
first/second list child; all 5,175 rot decisions and equality; resulting-air filtering;
chunk/write/NBT/type gates; base versus overwritten chest seed; cage/empty/entity
clip/decode/finalization; six loot pools.

**Constants and randomness:**

Core topology consumes the structure stream. Overgrown rot is independently position-derived. Each
retained chest load uses caller `nextLong`; each created mob finalization uses the level RNG. Exact
pool, cell, NBT and entity order is observable.

**Side effects:**

Nondestructive rigid/terrain-aligned outpost pieces; sparse tower overgrowth; fixed ominous banners;
one normally present loot chest with an optional second load/seed; optional captive golem or two
allays. Pillager natural spawns remain the record-owned full-box override.

**Gates:**

Record/core and connector/collision/depth/range; pool/template/processor availability; projection
height; position rot; legacy air filter; chunk clip and block/fluid/entity writes; resulting
block-entity type; entity decode and level difficulty; loot registry.

**Boundary cases and quirks:**

Full template volumes do not imply destructive air because legacy ignore runs last. Base/feature
plates are graph-visible but material-empty. The tower list's larger overgrown child affects its box
but not its connectors. Rot samples air and connector-final air before both are ignored. The
duplicated chest can advance and replace the seed twice. Captive mobs are raw finalized entities,
while pillagers come only from the separate natural-spawn override.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`. Anchors: four pillager-outpost pool records;
`outpost_rot`; all 11 templates; pillager-outpost chest loot; legacy-single/list, block-ignore/rot,
template block/NBT/entity and mob-finalization paths.

**Test vectors:**

Query/decode four pools, `outpost_rot` and the loot record; assert 11 top-level entries/weight `16`,
nested child order and all 11 exact locators. Decode every template; assert the 11-row census, 25
blocks/77 states, 55 connector fields, 18 NBT cells, three entity records and zero
absent/duplicate/structure-void/structure-block/missing/unreferenced inputs. Replay virtual plates,
projection, Empty/fallback, list box/connectors/order, every position-stable rot/air-ignore outcome,
duplicate banner/chest NBT and seed ordering, entity transform/finalization and all loot pools
through generic owners.
