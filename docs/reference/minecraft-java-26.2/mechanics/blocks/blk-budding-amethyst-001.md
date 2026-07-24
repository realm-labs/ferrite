# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BUDDING-AMETHYST-001` — Budding amethyst grows the directional bud and cluster chain

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`,
`BLK-007`, `PLY-005`, `PLY-006`, `ITM-006`, `ENV-001`, `ENV-002`, `ENV-003`, `WGEN-003`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registrations, complete budding/cluster implementations, entity
inside-step consumer, loot and tag data, geode consumer, registry reports, all 1,212 decoded
structure templates and client assets exhaust the five-identity growth chain.

**Applies when:**

`minecraft:budding_amethyst` receives an admitted random tick or projectile hit, an entity walks on
it, a bud or cluster is placed or loses support, water occupies a stage, any stage is mined,
amethyst geode generation selects a stage, or the five identities are persisted or projected.

**Authoritative state:**

Budding amethyst is a property-free `BuddingAmethystBlock` with sole state `23403`, block protocol
ID `979` and item raw ID `116`. It has purple map color, random ticks, strength/resistance `1.5/1.5`,
the volume/pitch-`1/1` `AMETHYST` sound type, correct-tool-required registration and `DESTROY`
piston reaction. Its direct block tags are exactly `crystal_sound_blocks` and `mineable/pickaxe`.
It inherits `AmethystBlock`'s authoritative projectile-chime override, and its crystal-sound
membership is the second and final member of the tagged entity-footstep accumulator.

The growth stages are `AmethystClusterBlock` instances with `facing` in
`north,east,south,west,up,down` and `waterlogged` in `true,false`, producing twelve states each:

| Identity | State range; default | Block/item raw IDs | Height/width | Light | Sound type |
|---|---:|---:|---:|---:|---|
| `small_amethyst_bud` | `23440..23451`; `23449` | `983` / `1446` | `3` / `8` | `1` | `SMALL_AMETHYST_BUD` |
| `medium_amethyst_bud` | `23428..23439`; `23437` | `982` / `1447` | `4` / `10` | `2` | `MEDIUM_AMETHYST_BUD` |
| `large_amethyst_bud` | `23416..23427`; `23425` | `981` / `1448` | `5` / `10` | `4` | `LARGE_AMETHYST_BUD` |
| `amethyst_cluster` | `23404..23415`; `23413` | `980` / `1449` | `7` / `10` | `5` | `AMETHYST_CLUSTER` |

Each default faces up and is dry. All four copy the cluster's purple, forced-solid/non-occluding,
strength-`1.5` and `DESTROY` profile. Their centered directional shapes use the listed square width
and protrusion height from the supporting face. They have no block entity or random tick. All four
are direct `mineable/pickaxe` members; only the small bud is also a direct
`inside_step_sound_blocks` member. Their ordinary items are common stack-64 block items.

**Transition and ordering:**

#### Budding random tick

For each random-tick callback, budding amethyst first consumes `nextInt(5)`. Values `1..4` abort
without choosing a face. Only zero consumes a second draw, `nextInt(6)`, selecting one of the six
`Direction.values()` entries uniformly, with indices `0..5` equal to
`down,up,north,south,west,east`. The target is the adjacent cell on that face.

If the target is air, or is the exact water block with a full fluid state, the result is a small
bud. Otherwise only these same-facing identities advance:

`small_amethyst_bud -> medium_amethyst_bud -> large_amethyst_bud -> amethyst_cluster`.

A different-facing stage, a cluster, flowing/non-full water, or any other state aborts after the
two draws. An admitted result starts from the next stage's default state, sets `facing` to the
selected direction, sets `waterlogged` when the old target fluid is water, then calls
`setBlockAndUpdate`; its boolean result is ignored. Thus new growth into water requires a full
source, dry stages stay dry while advancing, and waterlogged stages remain waterlogged.

#### Placement, support and water

Player placement selects the clicked face and sets `waterlogged` whenever the target fluid state
is water. A stage survives only when the block behind it—opposite its facing—has a sturdy face
toward the stage. On every neighbor-shape update, a waterlogged stage first schedules a water tick
at its own position with the world's water delay. If the changed neighbor is its support and the
support test now fails, the update returns air; otherwise inherited shape-update behavior runs.
Rotation rotates `facing`; mirror delegates through its direction-derived rotation; waterlogging
does not change. `getFluidState` returns a non-falling source-water state exactly when waterlogged.

The entity primary-step selector checks the cell above the ordinary support position before the
generic step sound is chosen. Because only the small bud is in `inside_step_sound_blocks`, an
entity occupying that stage selects its small-bud sound position; the other three stages do not
enter that exact tag branch. Generic movement and sound volume remain entity-owned.

#### Sounds and loot

Budding amethyst uses the five amethyst-block material sounds and retains the separate inherited
projectile/footstep chime behavior. Cluster uses five dedicated cluster sounds at registry IDs
`36..40`. Each bud uses its own break/place sounds—large `922/923`, medium `973/974`, small
`1503/1504`—but shares cluster step, hit and fall sounds. Every sound type has volume/pitch `1/1`.

Budding amethyst's block loot table has no pools and therefore drops nothing even with a correct
tool or Silk Touch. Each bud drops exactly its own item only when the tool has Silk Touch level at
least one; otherwise it drops nothing. Cluster loot selects, in order: its own item for Silk Touch;
otherwise four amethyst shards with the Fortune ore-drops bonus when the tool is one of the seven
locked `cluster_max_harvestables` pickaxes (wooden, stone, copper, iron, golden, diamond or
netherite); otherwise two shards with explosion decay. These tables use their matching
`minecraft:blocks/<identity>` random sequences. No recipe, advancement, trade or non-block loot
table names any of the five block identities.

#### Geode join

The locked amethyst-geode configuration uses budding amethyst as the alternate inner-layer state
with probability `0.083` and lists the four dry, up-facing growth stages as inner placements.
`GeodeFeature` chooses a direction, rewrites the selected stage's facing, sets waterlogged from
whether the destination fluid is a source, and admits the write through the same
`BuddingAmethystBlock.canClusterGrowAtState` air/full-water predicate. Direction retries,
placement selection, protection and safe-write behavior remain with `WGEN-PIPELINE-001`.

An exhaustive scan of all 1,212 locked structure NBT files finds no palette or live-cell occurrence
of any of the five identities. Structures are an explicit absence boundary, not a second source.

**Client projection:**

Budding amethyst uses one opaque `cube_all` block/item model. Each stage blockstate ignores
`waterlogged` for model selection and rotates one cross model by facing: up unrotated, down X=180,
north X=90, east X=90/Y=90, south X=90/Y=180 and west X=90/Y=270. Bud items use generated
stage-specific textures and transforms; the cluster item uses its generated cluster texture and
head translation. Block updates, light changes, water convergence, drops and sounds retain their
existing protocol families; this leaf adds no packet layout or connection state.

**Branches and aborts:**

Random-tick admitted versus 4/5 rejection; six selected faces; air versus full-water creation;
same-facing stage advance versus wrong-facing/terminal/occupied rejection; successful versus
ignored-failed write; player versus growth versus geode placement; six facings and two water
states; support retained versus lost; water tick scheduled versus absent; budding projectile and
crystal step; small-bud inside-step versus other stages; Silk Touch versus harvestable-pickaxe
Fortune versus fallback/explosion loot; persisted state versus model/light/sound projection are
distinct.

**Constants and randomness:**

Budding state `23403`, IDs `979/116`, strength `1.5/1.5`, growth divisor `5`, direction bound `6`;
stage state ranges/defaults and IDs as tabulated; dimensions `3x8`, `4x10`, `5x10`, `7x10`; light
`1,2,4,5`; stack `64`; geode alternate chance `0.083`; cluster base shard counts `4` and `2`.
Only an admitted 1-in-5 growth attempt consumes the direction draw; loot and geode owners retain
their own random streams.

**Side effects:**

One optional adjacent stage write and neighbor/client update; optional scheduled water tick;
support-loss replacement with air; inherited projectile/footstep sounds; block/item drops;
geode terrain writes; ordinary state persistence plus block, item, light, water and sound
projection.

**Gates:**

Random-tick chunk admission and 1-in-5 roll; exact target identity, facing and full-water state;
write authority; clicked face and target fluid; support-face sturdiness; live water, crystal-sound,
inside-step, pickaxe and cluster-harvestable tags; loot/enchantment/explosion context; geode
selection/protection context; client registry/model/light/sound context.

**Boundary cases and quirks:**

The 4/5 growth rejection consumes no face draw. A wrong-facing bud never advances. Flowing water
is not a new-growth target, although player placement can record waterlogged for a water fluid
state. Budding amethyst is correct-tool registered but its empty loot table still yields nothing.
Silk Touch on a non-pickaxe can select bud or cluster self loot because those loot branches test
the enchantment independently of the mineable tag. Only the small bud redirects the primary step
sound through the inside-step tag.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.BuddingAmethystBlock#randomTick`;
`net.minecraft.world.level.block.BuddingAmethystBlock#canClusterGrowAtState`;
`net.minecraft.world.level.block.AmethystClusterBlock#getShape`;
`net.minecraft.world.level.block.AmethystClusterBlock#canSurvive`;
`net.minecraft.world.level.block.AmethystClusterBlock#updateShape`;
`net.minecraft.world.level.block.AmethystClusterBlock#getStateForPlacement`;
`net.minecraft.world.level.block.AmethystClusterBlock#rotate`;
`net.minecraft.world.level.block.AmethystClusterBlock#mirror`;
`net.minecraft.world.level.block.AmethystClusterBlock#getFluidState`;
`net.minecraft.world.entity.Entity#getPrimaryStepSoundBlockPos`;
`net.minecraft.world.level.levelgen.feature.GeodeFeature#place`;
`reports/blocks.json#minecraft:{budding_amethyst,*_amethyst_bud,amethyst_cluster}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`data/minecraft/loot_table/blocks/{budding_amethyst,*_amethyst_bud,amethyst_cluster}.json`;
`data/minecraft/tags/{block/{crystal_sound_blocks,inside_step_sound_blocks,mineable/pickaxe},item/cluster_max_harvestables}.json`;
`data/minecraft/worldgen/configured_feature/amethyst_geode.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{blockstates,models}/**/*amethyst*.json`.

**Test vectors:**

Run `EXP-BLK-053` with fixed RNG across all five first-roll results, all six direction indices,
air/source-water/flowing-water/occupied targets, every same/wrong-facing stage, ignored write
failure, player/geode placements, every support face, dry/waterlogged neighbor updates, rotations,
mirrors, all loot branches and boundary Fortune levels, structure absence, save/reload and every
model rotation. Assert exact draw order, states, water scheduling, drops, light, sounds and client
convergence.

**Limits:**

Random-tick scheduling, generic state publication, projectile collision, entity walking, fluid
ticks, loot evaluation, enchantment bonus arithmetic, geode traversal, packet encoding and client
rendering remain with `SIM-RANDOM-001`, `BLK-UPDATE-001`, `ENT-001`, `ENV-FLUID-001`,
`ITM-LOOT-001`, `WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
