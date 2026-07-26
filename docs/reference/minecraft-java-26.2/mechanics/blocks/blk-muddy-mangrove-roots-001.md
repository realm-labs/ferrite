# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MUDDY-MANGROVE-ROOTS-001` — Muddy mangrove roots rotate and join mud, growth, mob and structure selectors

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `PLY-005`, `PLY-006`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `BLK-003`, `BLK-004`, `BLK-005`,
`BLK-007`, `BLK-UPDATE-001`, `PLY-002`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`, `RED-COMPARATOR-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-005`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`,
`MOB-SPAWN-001`, `ENV-001`, `ENV-002`, `ENV-003`, `ENV-FLUID-001`,
`ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-002`, `WGEN-003`, `WGEN-004`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked `RotatedPillarBlock` registration and implementation, reports,
complete loot/recipe/advancement/tag and class-reference searches, both mangrove configured
features, all 1,212 decoded templates, Trial Chambers pools and exact client resources exhaust this
identity. Generic owners already specify every algorithm selected by its direct and transitive tag
memberships; this leaf fixes their Muddy-Mangrove-Roots inputs, outputs and absences.

**Applies when:**

`minecraft:muddy_mangrove_roots` is placed, explicitly written, transformed, mined, exploded,
crafted, tested as mud or plant support, inspected by Frog, Sniffer or Enderman behavior, used by
mangrove generation, equipped on a Sulfur Cube, placed from a Trial Chambers payload, persisted,
mapped or rendered.

**Authoritative state:**

Muddy Mangrove Roots is a `RotatedPillarBlock` with no block entity. Its sole property is
`axis={x,y,z}`: states `165/166/167` respectively, with Y state `166` the default. Its locked block
protocol ID is `59`; its ordinary block-item raw ID is `171`.

Registration fixes map color `PODZOL`, the default `HARP` note instrument, one-argument
hardness/resistance `0.7/0.7` and `MUDDY_MANGROVE_ROOTS` sounds. Every state is a full unit
selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade brightness
`0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone conduction, normal
piston reaction, full sturdy faces and ordinary spawn support. It adds no random or scheduled tick,
use, attack, contact, neighbor, signal, comparator, fluid or block-event override and is not
ignited by lava.

Its direct block tags are `mineable/shovel`, `mud`, `frogs_spawnable_on`,
`supports_big_dripleaf`, `mangrove_logs_can_grow_through` and
`mangrove_roots_can_grow_through`. Registration does not require a correct tool, so a Shovel only
accelerates mining: hand, other tools and every Shovel all admit self loot. The sound type has
volume/pitch `1/1` and exact event IDs break `1015`, step `1019`, place `1018`, hit `1017` and fall
`1016`.

The ordinary common block item stacks to `64`, has only standard block-item components and directly
belongs to the item `mud` tag and `sulfur_cube_archetype/regular`.

**Transition and ordering:**

### Axis placement, transforms and self loot

Ordinary placement starts from state `166` and replaces `axis` with the clicked face's axis:
East/West selects X, Up/Down Y and North/South Z. Explicit component, command and template writes
preserve a supplied legal state.

Clockwise or counterclockwise quarter turns exchange X and Z while retaining Y. No rotation, a
half turn and every mirror retain the axis. Structure-template transforms use the same operation.
Axis affects only state/model orientation.

After successful survival removal, the one-roll self table offers one Muddy Mangrove Roots behind
`survives_explosion`, with random sequence `minecraft:blocks/muddy_mangrove_roots`. Tool type,
Silk Touch and Fortune add no loot branch. Enderman removal is a separate no-drop world mutation.

### Shapeless conversion and recipe knowledge

One Building-category shapeless recipe consumes one exact Mud plus one exact Mangrove Roots in
either grid order and returns one default-state Muddy Mangrove Roots. Input component patches are
discarded. Its advancement grants only this recipe when either exact Mangrove Roots enters the
inventory or the recipe is already known; Mud possession alone does not unlock it. No locked
recipe consumes Muddy Mangrove Roots or substitutes its item-mud membership for an ingredient.

### Direct support and mob selectors

- A Frog natural-spawn candidate accepts this block immediately below only when the candidate
  position also has raw brightness strictly greater than `8`; remaining placement, collision,
  biome and spawn-cycle gates stay with `MOB-SPAWN-001`.
- A Big Dripleaf survives when its below state is this direct support member. A Big Dripleaf Stem
  additionally requires Big Dripleaf Stem or Big Dripleaf immediately above. Their placement,
  waterlogging, support-loss scheduling and growth remain with the plant owners.
- Both mangrove trunk and root configurations may traverse this direct grow-through member.
  Root-placement consequences are specified below.

The direct block `mud` membership expands through exactly `33` ancestor tags. Its five immediate
parents are `substrate_overworld`, `moss_replaceable`, `enderman_holdable`,
`sniffer_diggable_block` and `cannot_replace_below_tree_trunk`. The complete locked closure then
selects:

- Overworld and Nether carver replacement; Sculk ordinary/worldgen replacement; ice-spike,
  forest-rock, huge-mushroom, azalea/root-system, lush-ground, moss-patch and podzol-decoration
  inputs;
- Bamboo, Sugar Cane, Azalea, dry vegetation, Wither Rose, Mangrove Propagule, stem-fruit,
  Nether-sprout, crimson/warped root and fungus support;
- Sniffer digging after age, water, ground, passenger, history and navigation admission; and
- Enderman carrying, plus the below-trunk rule used by every non-Azalea locked tree record.

That below-trunk rule offers Dirt only when the original cell is outside
`cannot_replace_below_tree_trunk`; Muddy Mangrove Roots therefore prevents that particular Dirt
rewrite. Sculk, carver, vegetation, feature and plant owners retain their exact read, RNG, support
and write order.

An empty-handed Enderman under `mobGriefing` may select an unobstructed Muddy-Mangrove-Roots state
through `enderman_holdable`, remove it without loot, emit `BLOCK_DESTROY`, and store the block's
default state. Pickup therefore normalizes X or Z to Y state `166`. Its later generic placement
retains that Y state and requires the placement target and support/entity gates owned by
`MOB-AI-001`.

The direct item `mud` tag has no locked code consumer, recipe use or parent tag. It remains an
exact reload-visible classification rather than an extra behavior.

### Regular Sulfur-Cube equipment

The item directly selects the `regular` archetype. Its record fixes horizontal/vertical knockback
powers `0.4125/0.09`, hit/push sounds, push cooldown `0.5`, impulse threshold `0.2`, additive
knockback and explosion-knockback resistance `-1/-1`, additive bounciness `0.5`,
total-multiplied friction `-0.699999988079071`, total-multiplied air drag
`-0.8999999985098839`, and buoyancy enabled.

Matching order, equipment replacement, modifier lifecycle, buoyancy, contact, knockback, sound and
entity projection remain with the Sulfur-Cube/entity owners. Reload changes future classification
without mutating placed states.

### Mangrove generation

Both `mangrove` and `tall_mangrove` configured trees name Muddy Mangrove Roots in two roles.
Their root preflight can traverse existing Mud or Muddy Mangrove Roots, and an admitted staged cell
whose live state is either identity samples the simple provider for default axis-Y Muddy Mangrove
Roots. Waterlogging is attempted only when the selected state exposes that property, so this
non-waterloggable result stays dry. That muddy branch performs no common recheck and no above-root
chance.

Other admitted root cells use Mangrove Roots and may place Moss Carpet above at strict chance
`0.5`. Both records use maximum root width `8`, maximum length `15` and skew chance `0.2`;
ordinary/tall trunk offsets are inclusive `1..3` and `3..7`. Root direction preflight, shared
budgets, staged mutation, trunk grow-through, provider timing and all-or-nothing abort order remain
with `WGEN-PIPELINE-001`. Generation supplies an unbounded world-dependent number of Y states, not
a fixed template census.

### Trial Chambers payload

The exhaustive template scan finds one raw occurrence across all `1,212` templates:
`trial_chambers/corridor/addon/display_2` stores state X at local `[3,1,3]`. It is one of three
equal-weight rigid entries in `trial_chambers/entrance`, each with inline-empty processors.
Reachable `corridor/entrance_1` and `corridor/entrance_2` templates each contain a connector to that
pool. Its first shuffled proposal is therefore conditionally `1/3` before attachment/collision
admission; rotation can retain X or transform it to Z.

No processor alters or protects the cell at the element boundary. Ordinary overlap, clip,
placement admission and downstream Trial Chambers assembly remain with
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`. The source occurrence is not a guaranteed final-world write.

The complete direct server-data search has 13 JSON files: self loot, one recipe, one advancement,
six block tags, two item tags and two configured features. Outside registrations, data generators,
generic publication and the named selector consumers, the class-reference sweep finds no other
identity-specific runtime path. No other loot, recipe, advancement, trade, configured feature or
optional built-in-pack record names the identity.

**Client projection:**

The blockstate maps X to `block/muddy_mangrove_roots` with model rotations X/Y `90/90`, Y to the
unrotated model and Z to rotation X `90`. The model inherits `block/cube_column`, using
`block/muddy_mangrove_roots_top` on axis ends and `block/muddy_mangrove_roots_side` elsewhere. The
item directly selects the vertical block model.

English translation is `Muddy Mangrove Roots`. The Natural Blocks tab publishes it once after
Mangrove Roots and before Cherry Log, in the local order Mangrove Log, Mangrove Roots, Muddy
Mangrove Roots, Cherry Log. Block updates use states `165..167`, inventory paths use item ID `171`,
sounds use IDs `1015..1019`, and maps use `PODZOL`. This identity adds no packet field or
connection-local state.

**Branches and aborts:**

Three axes and six clicked faces; ordinary versus explicit/template writes; quarter/half/no
rotation and mirror; Shovel/other/hand mining and ordinary/explosion/Enderman removal; shapeless
grid order, malformed inputs, output capacity and two OR unlock routes; six direct block tags,
33 transitive mud ancestors and two item tags under reload; Frog brightness; Sniffer and Enderman
admission; Dripleaf support; ordinary/tall mangrove root/trunk branches; current/reloaded regular
equipment; Trial entrance selection, transform, clip and write result; persistence and exact client
projection are distinct.

**Constants and randomness:**

X/Y/Z states `165/166/167`; block/item IDs `59/171`; strength `0.7/0.7`; emission `0`, dampening
`15`, shade `0.2`, friction `0.6`, speed/jump `1`, restitution `0`; sound
break/step/place/hit/fall IDs `1015/1019/1018/1017/1016`, volume/pitch `1/1`; stack `64`; one Mud
plus one Mangrove Roots to one result; Frog brightness `>8`; direct block/item tags `6/2`, mud
ancestors `33`; mangrove width/length/skew `8/15/0.2`, above-root chance `0.5`, offsets `1..3` and
`3..7`; Trial templates/files/cells `1/1`, first-proposal weight `1/3`; regular-archetype values as
listed. Axis/block behavior consumes no RNG; loot, recipe, mob, equipment and worldgen owners
retain their streams.

**Side effects:**

Axis-selected full-block placement/removal; explosion-gated self loot; one shapeless result and
knowledge grant; support/spawn/dig/carry/growth/replacement admissions; Enderman normalization to
default Y; reload-selected regular equipment; mangrove root writes; one Trial payload offer;
ordinary persistence, Podzol maps, dedicated sounds and oriented cube-column projection.

**Gates:**

World-write/transform/break authority; explosion survival; recipe/advancement snapshot and output
admission; active block/item tags; Frog light/spawn context; Sniffer/Enderman AI state, gamerule,
path/ray/support/entity gates; Dripleaf and vegetation survival; mangrove preflight/provider/write
admission; Sulfur-Cube archetype; Trial pool, transform, overlap, clip and write admission; valid
registry/map/sound/client-resource context.

**Boundary cases and quirks:**

Ordinary placement uses clicked-face axis, while mangrove providers and Enderman carrying produce
Y. Every tool can harvest it despite the Shovel mining tag. Mud possession does not unlock its
recipe. It is a direct grow-through state and can also be rewritten to itself by mangrove roots.
Its mud membership both admits destructive replacement families and blocks below-trunk Dirt
replacement. The item-mud tag has no locked consumer. The sole template cell is X and uses no
processor, while dynamic mangrove output is Y and has no finite cell count.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.RotatedPillarBlock`;
`net.minecraft.world.entity.animal.Animal#isBrightEnoughToSpawn`;
`net.minecraft.world.entity.animal.frog.Frog#checkFrogSpawnRules`;
`net.minecraft.world.entity.animal.sniffer.Sniffer#canDig`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal#tick`;
`net.minecraft.world.level.block.BigDripleafBlock#canSurvive`;
`net.minecraft.world.level.block.BigDripleafStemBlock#canSurvive`;
`net.minecraft.world.level.levelgen.feature.rootplacers.MangroveRootPlacer`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:muddy_mangrove_roots`;
`reports/registries.json#minecraft:{block,item}/minecraft:muddy_mangrove_roots`;
`reports/registries.json#minecraft:sound_event/minecraft:block.muddy_mangrove_roots.*`;
`reports/minecraft/components/item/muddy_mangrove_roots.json`;
`data/minecraft/loot_table/blocks/muddy_mangrove_roots.json`;
`data/minecraft/recipe/muddy_mangrove_roots.json`;
`data/minecraft/advancement/recipes/building_blocks/muddy_mangrove_roots.json`;
`data/minecraft/tags/block/{frogs_spawnable_on,mangrove_logs_can_grow_through,mangrove_roots_can_grow_through,mineable/shovel,mud,supports_big_dripleaf}.json`;
`data/minecraft/tags/item/{mud,sulfur_cube_archetype/regular}.json`;
`data/minecraft/sulfur_cube_archetype/regular.json`;
`data/minecraft/worldgen/configured_feature/{mangrove,tall_mangrove}.json`;
`data/minecraft/worldgen/template_pool/trial_chambers/entrance.json`;
`data/minecraft/structure/trial_chambers/corridor/{entrance_1,entrance_2,addon/display_2}.nbt`;
`assets/minecraft/blockstates/muddy_mangrove_roots.json`;
`assets/minecraft/models/block/muddy_mangrove_roots.json`;
`assets/minecraft/items/muddy_mangrove_roots.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-092` across all axes, faces and transforms; every tool and removal mode; recipe/unlock
case; direct and transitive tag reload; Frog, Dripleaf, Sniffer and Enderman boundary; both
mangrove records and every root/trunk preflight/provider/write branch; regular equipment; all
1,212 templates and Trial selection/transforms; persistence, IDs, sounds, map and block/item
projection. Assert exact constants, the 33-tag closure, sole X template cell and vanilla-client
convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancements, mob goals/spawning, plant support,
Sulfur-Cube behavior, tree/feature/carver/Sculk algorithms, jigsaw processing, packet encoding and
rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `MOB-AI-001`, `MOB-SPAWN-001`, `ENT-KNOCKBACK-001`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001`,
`PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf fixes the exact identity, axis
specialization, selector joins, generation inputs/outputs, absences and projection.
