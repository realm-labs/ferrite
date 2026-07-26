# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BLACKSTONE-001` — Blackstone masonry joins Nether terrain, Piglins, ruins and Bastions

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`, `PLY-BREAK-001`, `BLK-003`,
`BLK-005`, `BLK-UPDATE-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-STONECUTTER-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ENT-001`,
`ENT-005`, `ENT-KNOCKBACK-001`, `MOB-AI-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-BASTION-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and reports, complete loot/recipe/advancement/tag/class-
reference searches, both direct features, Nether surface and biome wiring, all 40 processor lists,
all 1,212 decoded templates, the complete Bastion graph and exact client assets exhaust the six
property-free full-block identities. Their stairs, slabs, walls, button and pressure plate retain
their separately audited implementation families.

**Applies when:**

`minecraft:blackstone`, `minecraft:polished_blackstone`,
`minecraft:polished_blackstone_bricks`, `minecraft:chiseled_polished_blackstone`,
`minecraft:cracked_polished_blackstone_bricks` or `minecraft:gilded_blackstone` is placed, mined,
exploded, crafted, cooked, cut, bartered, selected by a Piglin or reloadable tag, used as feature
support or replacement terrain, transformed by a ruined-portal or Bastion processor, written from
a Bastion template, persisted, mapped or rendered.

**Authoritative state:**

All six are ordinary `Block` registrations with no property and no block entity:

| Identity | State | Block protocol ID | Item raw ID | Strength | Sound |
|---|---:|---:|---:|---:|---|
| Blackstone | `21831` | `924` | `1416` | `1.5/6` | Stone |
| Polished Blackstone | `22242` | `928` | `1420` | `2/6` | Stone |
| Polished Blackstone Bricks | `22243` | `929` | `1424` | `1.5/6` | Stone |
| Chiseled Polished Blackstone | `22245` | `931` | `1423` | `1.5/6` | Stone |
| Cracked Polished Blackstone Bricks | `22244` | `930` | `1427` | `1.5/6` | Stone |
| Gilded Blackstone | `22656` | `935` | `1419` | `1.5/6` | Gilded Blackstone |

Blackstone fixes `COLOR_BLACK`, `BASEDRUM`, `requiresCorrectToolForDrops` and strength `1.5/6`.
Polished copies it and changes strength to `2/6`; Bricks, Chiseled and Cracked resolve to
`1.5/6`. Gilded copies Blackstone and changes only its sound type. Every state is a full unit
selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade brightness
`0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone conduction, normal
piston reaction and full sturdy faces. None adds random/scheduled ticks, use, attack, contact,
neighbor, signal, comparator, fluid or block-event behavior.

All six are directly `mineable/pickaxe` and belong to no minimum-tier tag, so every Pickaxe is
correct. The first five use Stone sound IDs break/step/place/hit/fall
`1596/1604/1601/1600/1599`; Gilded uses break/step/place/hit/fall
`715/719/718/717/716`. Both profiles have volume/pitch `1/1`.

Their common block items stack to `64` and directly select
`sulfur_cube_archetype/slow_bouncy`. Blackstone alone also belongs directly to the three-member
item tags `stone_crafting_materials` and `stone_tool_materials`. Gilded alone is a direct
`piglin_loved` item, and its block alone is directly `guarded_by_piglins`.

**Transition and ordering:**

### Placement, harvest and loot

Placement, explicit writes, rotation and mirror retain the selected sole state. A legal
`minecraft:block_state` component cannot add a property. Correct-tool admission precedes each
loot table.

Blackstone, Polished, Bricks, Chiseled and Cracked each have a one-roll self-item entry behind
`survives_explosion`, using random sequence `minecraft:blocks/<identity>`. Silk Touch and Fortune
do not change those five tables. A wrong-tool player removal emits nothing.

Gilded instead tests Silk Touch first and emits one Gilded block without an explosion condition.
Its non-Silk fallback first passes `survives_explosion`; failure emits nothing. Survival runs
Fortune `table_bonus` with Nugget chances `0.1`, `0.14285715`, `0.25` and `1` at levels `0`, `1`,
`2` and `>=3`. Success emits uniform integer `2..5` Gold Nuggets; failure emits one Gilded block.
There is no Fortune count multiplier or per-unit explosion decay. Its sequence is
`minecraft:blocks/gilded_blackstone`.

### Exact masonry, tool-material and template recipes

Exactly 39 bundled recipe records name one of the six identities: 15 crafting, 23 Stonecutting and
one smelting. Their graph is:

- four Blackstone in `##/##` produce four Polished; four Polished likewise produce four Bricks;
- two vertically stacked Polished slabs produce one Chiseled;
- Blackstone, Polished and Bricks each use the conventional three-in-a-row slab recipe to six,
  six-input stair recipe to four and six-input wall recipe to six;
- one Polished shapelessly produces one button, while two in a row produce one pressure plate;
- Blackstone Stonecuts to twelve outputs: its three shapes; Polished plus its three shapes; Bricks
  plus their three shapes; and Chiseled;
- Polished Stonecuts to eight outputs: its three shapes, Bricks plus their three shapes, and
  Chiseled; Bricks Stonecuts to its three shapes;
- Bricks smelt to one Cracked block in the omitted-field default `200` ticks for `0.1` XP; and
- seven Diamonds, one Snout template and one Blackstone in `#S#/#C#/###` produce two Snout
  templates.

Every cut returns one except slabs, which return two. Each of the 39 recipes has its own
advancement with a two-criterion OR requirement: prior knowledge or possession of the record's
source identity. The Chiseled crafting record uses Polished-slab possession; Snout duplication uses
template possession, not Blackstone. Results are default stacks, so arbitrary input components are
discarded.

Blackstone's two material tags add eight shaped matches outside that exact-ID census: three
material slots can make one Brewing Stand, eight make one Furnace, and the six Stone
axe/hoe/pickaxe/shovel/spear/sword patterns consume respectively `3/2/3/1/1/2` tag members.
Blackstone possession satisfies the tag-based Furnace and tool unlocks; Brewing Stand unlock
instead follows Blaze Rod possession.

### Piglin, barter, guarded-block and equipment joins

A player breaking Gilded Blackstone invokes nearby-Piglin anger through
`Block#playerWillDestroy` before removal. Other destruction paths and the other five identities do
not gain this direct guarded-block hook.

Subject to the generic baby-ignore, repellent, activity, attack/admirer and inventory gates, a
Piglin can want a Gilded item entity. Pickup removes exactly one non-Nugget item, drops a previous
offhand stack, moves Gilded into the off hand and installs `ADMIRING_ITEM=true` for `119` ticks.
Gilded is not the exact Gold-Ingot currency: an adult finishing admiration attempts equipment and
then inventory storage without barter loot. A player holding it also satisfies loved-item sensing
and the thrown route of `nether/distract_piglin` when that advancement's armor/source gates hold.

The one-roll `gameplay/piglin_bartering` table independently emits uniform `8..16` Blackstone at
weight `40/469`. This is acquisition of Blackstone after an exact Gold-Ingot barter, not a
Blackstone input or a property of Gilded.

Four Bastion chest rows can acquire Gilded:

| Table | Rolls in selected pool | Weight/total | Count |
|---|---:|---:|---:|
| `bastion_bridge` | `1..2` | `1/13` | `8..12` |
| `bastion_hoglin_stable` | `3..4` | `1/14` | `2..5` |
| `bastion_other` | `3..4` | `2/13` | `1..5` |
| `bastion_treasure` | `3..4` | `1/9` | `5..15` |

All six items also select slow-bouncy. Its record fixes horizontal/vertical knockback
`0.4125/0.24`, push cooldown `0.5`, impulse threshold `0.05`, additive knockback and explosion-
knockback resistance `0.4000000059604645/0.4000000059604645`, additive bounciness
`0.6000000238418579`, total-multiplied friction `-0.699999988079071` and air drag
`-0.9499999992549419`, plus its hit/push sounds. Matching and modifier lifecycle remain with the
Sulfur-Cube owners.

### Reloadable Nether and Sculk selectors

Blackstone's direct `base_stone_nether` block membership closes to three blocks: Netherrack,
Basalt and Blackstone. It therefore also enters `nether_carver_replaceables` and
`sculk_replaceable`. Nether cave carving and ordinary Sculk replacement may consume it through
those composed tags.

The same base-stone tag is the target for both Ancient Debris configured features, so exposed-
neighbor admission can replace Blackstone with Ancient Debris exactly as it can Netherrack or
Basalt. The two scattered features have sizes `3/2`, air-exposure discard `1`, and are scheduled in
all five Nether biomes; their attempt/draw/write algorithm remains with `WGEN-PIPELINE-001`.

`spring_lava_nether` separately names Blackstone among its five valid blocks. Blackstone can thus
satisfy the required support, origin and exact four-rock/one-hole counts for a falling-Lava spring.
`GlowstoneFeature` independently accepts exact Blackstone above an empty origin, alongside
Netherrack or Basalt, before its 1,500 growth attempts. Neither join accepts another family member.

### Natural terrain and features

In the Nether surface-rule tree, Basalt Deltas floor cells that reach the state-selector fallback
become vertical Basalt when `nether_state_selector >= 0`, otherwise default Blackstone. Earlier
ceiling and gravel-patch branches take precedence. The locked Nether noise setting otherwise uses
Netherrack as its default state.

The configured `blackstone_blobs` feature searches downward for Netherrack, samples three
independent uniform radii `3..7`, and offers Blackstone throughout the clipped Manhattan
octahedron wherever the current block is Netherrack. Its Basalt-Deltas placement performs count
`25`, in-square, uniform full-build-height and biome filtering.

The configured `ore_blackstone` feature has vein size `33`, target exact Netherrack, result
Blackstone and air-exposure discard `0`. Its placement performs count `2`, in-square, uniform
absolute Y `5..31` and biome filtering. Nether Wastes, Soul Sand Valley, Crimson Forest and Warped
Forest schedule it; Basalt Deltas instead schedule the blob feature. Ore geometry, placement
modifier draws and write admission remain with `WGEN-PIPELINE-001`.

### Ruined-portal replacement

Every Nether-placed ruined portal runs deterministic `blackstone_replace` after block aging. Its
23-entry map produces five locked identities: Cobblestone/Mossy Cobblestone become Blackstone;
Stone becomes Polished; Stone Bricks/Mossy Stone Bricks become Bricks; Chiseled/Cracked Stone
Bricks become the matching Chiseled/Cracked Blackstone state. Corresponding cobblestone/stone/
stone-brick stairs, slabs and walls become their shape-family Blackstone variants, and Iron Bars
becomes Chain.

The processor copies only stair `facing`, stair `half` and slab `type` when present, retaining
position and NBT; other properties are discarded. It consumes no RNG. Consequently Polished can
be generated by a ruined portal even though no raw template contains it. Portal vertical
placement, aging RNG, processor order, protected/live target, transforms, clipping and writes
remain with `WGEN-STRUCTURE-RUINED-PORTAL-001`.

### Bastion payload and degradation

The exhaustive all-template scan finds `146,042` raw family cells, all in Bastion inputs:

| Identity | Templates with live cells | Raw cells |
|---|---:|---:|
| Blackstone | `119` | `55,462` |
| Polished Blackstone | `0` | `0` |
| Polished Blackstone Bricks | `144` | `86,109` |
| Chiseled Polished Blackstone | `17` | `137` |
| Cracked Polished Blackstone Bricks | `98` | `4,234` |
| Gilded Blackstone | `26` | `100` |

All 167 Bastion templates are reachable through the locked graph. Jigsaw final states additionally
name Blackstone `36`, Bricks `299` and Cracked `21` times; those are connector replacements, not
raw palette cells.

The twelve named Bastion lists apply ordered, position-seeded rules. Across the family they use:

- Generic: Bricks `0.3 -> Cracked`, Blackstone `0.0001 -> Air`, Gilded
  `0.5 -> Blackstone`, then Blackstone `0.01 -> Gilded`;
- Bottom Rampart: Cracked `0.15 -> Bricks`, Gilded `0.5 -> Blackstone`, Blackstone
  `0.01 -> Gilded`;
- Bridge: Bricks `0.3 -> Cracked`, Blackstone `0.0001 -> Air`;
- Entrance: Chiseled `0.5 -> Air`, Gilded `0.5 -> Blackstone`, Blackstone
  `0.01 -> Gilded`;
- High Rampart: an intervening all-state Y-linear Air rule rises from chance `0` at distance `0`
  to `0.05` at `100`, then Gilded `0.5 -> Blackstone`;
- High Wall: Bricks test `0.01 -> Air`, then `0.5 -> Cracked`, then `0.3 -> Blackstone`;
  Gilded tests `0.5 -> Blackstone`;
- Housing: Bricks `0.3 -> Cracked`, Blackstone `0.0001 -> Air`, Gilded
  `0.5 -> Blackstone`, then Blackstone `0.01 -> Gilded`;
- Rampart Degradation: Bricks `0.4 -> Cracked`; Blackstone `0.01 -> Cracked`; Bricks and
  Blackstone independently test `0.0001 -> Air`; Gilded `0.5 -> Blackstone`; then Blackstone
  `0.01 -> Gilded`;
- Roof: Bricks test `0.3 -> Cracked`, then `0.15 -> Air`, then `0.3 -> Blackstone`;
- Side Wall: Chiseled `0.5 -> Air`, Gilded `0.5 -> Blackstone`, then Blackstone
  `0.01 -> Gilded`;
- Stable: Bricks `0.1 -> Cracked`, Blackstone `0.0001 -> Air`, Gilded
  `0.5 -> Blackstone`, then Blackstone `0.01 -> Gilded`; and
- Treasure Rooms: Bricks `0.35 -> Cracked`, Chiseled `0.1 -> Cracked`, Gilded
  `0.5 -> Blackstone`, then Blackstone `0.01 -> Gilded`.

Rules targeting Gold or Magma can also create Cracked family cells but remain owned by those input
leaves. First match wins, so later probabilities are conditional on earlier failures. Jigsaw
selection, connector replacement, processor RNG, transforms, destructive Air, clip/live-target
admission and flags-`18` writes remain with the Bastion and processor owners. The census is raw
payload, not a guaranteed final-world count.

**Client projection:**

Every property-free blockstate has one unconditional matching model. Blackstone alone inherits
`cube_column`, using `blackstone_top` on the two ends and `blackstone` on the four sides. The other
five inherit `cube_all` with one matching texture. Each item directly selects its block model.

English names match the six display names in the state table. Natural Blocks publishes Blackstone
once between Bone Block and Basalt. Building Blocks publishes the family after Polished Basalt in
the order Blackstone, Gilded, three Blackstone shapes, Chiseled, Polished, its three shapes,
pressure plate, button, Bricks, Cracked, then the three Brick shapes. Polished Blackstone Bricks is
the icon for `nether/find_bastion`; the location criterion does not inspect the held block.

State updates use the six listed state IDs, inventory paths use the six item IDs, maps use
`COLOR_BLACK`, and sounds use the Stone or Gilded IDs above. No identity adds a packet field or
connection-local state.

**Branches and aborts:**

Six sole states; five self-loot versus Gilded nested loot; correct/wrong tool, Silk/Fortune and
explosion survival; 39 exact-ID and eight tag-keyed recipes with independent unlocks; guarded and
loved Gilded paths versus Blackstone barter output; live material/base-stone/archetype tags;
surface/blob/ore/spring/glowstone/debris/carver/Sculk selectors; deterministic ruined-portal
mapping; every Bastion graph/connector/processor/clip/write outcome; persistence and six client
projections are distinct.

**Constants and randomness:**

States/block/item IDs and strengths as tabulated; common physical constants above; Stone sound IDs
`1596/1604/1601/1600/1599`, Gilded `715/719/718/717/716`; stack `64`; recipes
`15/23/1/39` plus eight tag-keyed matches; smelting `200/0.1/1`; barter `40/469`, count `8..16`;
Gilded loot chances `0.1/0.14285715/0.25/1`, count `2..5`; Gilded template cells `100`; total raw
family cells `146,042`; feature radius/count `3..7/25`, ore size/count/height `33/2/5..31`.
Blocks consume no RNG directly; generic loot, Piglin, feature and structure owners retain their
streams.

**Side effects:**

Full-block placement/removal and conditional loot; crafting/cooking/cutting/knowledge; Piglin
anger, admiration, sensing, barter and chest outputs; tag-selected carving/Sculk/debris/equipment;
surface and feature writes; ruined-portal and Bastion processor/template writes; advancement icon;
ordinary persistence, maps, sounds and opaque block/item projection.

**Gates:**

Write and break authority; correct Pickaxe, Silk/Fortune and explosion context; active recipe,
advancement, loot, tag and archetype snapshots; Piglin age/activity/inventory/armor state; feature
origin/support/target/biome and write admission; portal/Bastion graph, processor, RNG, transform,
clip and target admission; valid registry/map/sound/client-resource context.

**Boundary cases and quirks:**

Every Pickaxe is correct without a tier gate. Gilded Silk loot is not explosion-conditioned, while
its non-Silk fallback is. Gilded is loved but is not barter currency; Blackstone is barter output
but is not loved. Blackstone can craft stone tools and a Furnace through tags, yet does not unlock
a Brewing Stand. Polished has zero raw template cells but is a deterministic ruined-portal output.
Basalt Deltas use surface/blob Blackstone, while the other four Nether biomes schedule its ore.
Bastion processor probabilities are ordered and can both remove and create family identities, so
raw counts are not expected final counts.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.Block#playerWillDestroy`;
`net.minecraft.world.entity.monster.piglin.PiglinAi`;
`net.minecraft.world.level.levelgen.feature.GlowstoneFeature#place`;
`net.minecraft.world.level.levelgen.structure.templatesystem.BlackstoneReplaceProcessor#processBlock`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{blackstone,polished_blackstone,polished_blackstone_bricks,chiseled_polished_blackstone,cracked_polished_blackstone_bricks,gilded_blackstone}`;
`reports/registries.json#minecraft:{block,item,sound_event}`;
`reports/minecraft/components/item/{blackstone,polished_blackstone,polished_blackstone_bricks,chiseled_polished_blackstone,cracked_polished_blackstone_bricks,gilded_blackstone}.json`;
`data/minecraft/loot_table/blocks/{blackstone,polished_blackstone,polished_blackstone_bricks,chiseled_polished_blackstone,cracked_polished_blackstone_bricks,gilded_blackstone}.json`;
`data/minecraft/loot_table/{chests/bastion_bridge,chests/bastion_hoglin_stable,chests/bastion_other,chests/bastion_treasure,gameplay/piglin_bartering}.json`;
`data/minecraft/{recipe,advancement/recipes}/**/*blackstone*.json`;
`data/minecraft/{recipe,advancement/recipes/misc}/snout_armor_trim_smithing_template.json`;
`data/minecraft/recipe/{brewing_stand,furnace,stone_axe,stone_hoe,stone_pickaxe,stone_shovel,stone_spear,stone_sword}.json`;
`data/minecraft/advancement/{nether/find_bastion,nether/distract_piglin}.json`;
`data/minecraft/tags/block/{base_stone_nether,guarded_by_piglins,mineable/pickaxe,nether_carver_replaceables,sculk_replaceable}.json`;
`data/minecraft/tags/item/{piglin_loved,stone_crafting_materials,stone_tool_materials,sulfur_cube_archetype/slow_bouncy}.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/{configured_feature/{blackstone_blobs,ore_blackstone,spring_lava_nether},placed_feature/{blackstone_blobs,ore_blackstone},noise_settings/nether,biome/*.json,processor_list/*.json}`;
`data/minecraft/structure/bastion/**/*.nbt`;
`assets/minecraft/{blockstates,models/block,items}/{blackstone,polished_blackstone,polished_blackstone_bricks,chiseled_polished_blackstone,cracked_polished_blackstone_bricks,gilded_blackstone}.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-095` across all six states, physical/tool/Silk/Fortune/explosion/loot branches; every
39 exact and eight tag-keyed recipe/unlock; guarded/loved/barter/chest/equipment paths; composed
tag consumers; Nether surface, blob, ore, spring, glowstone and debris fixtures; all ruined-portal
replacement inputs/properties; all 1,212 templates and every reachable Bastion graph/connector/
processor/rotation/clip/write path; persistence, IDs, sounds, maps, icon, tabs and exact assets.
Assert the six identities, ordered probabilities, zero raw Polished cells, the per-identity and
`146,042` total raw census, and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, cooking, Stonecutting, advancements, Piglin AI,
Sulfur-Cube behavior, terrain/features, template processing, packet encoding and rendering remain
with their named generic owners. Shape-family stairs/slabs/walls, button and pressure plate remain
under their existing leaves. This leaf fixes the exact six full-block identities, joins, absences
and projection.
