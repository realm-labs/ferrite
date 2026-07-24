# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHER-WART-BLOCK-001` — Nether wart blocks join composting, Nether growth, spawn exclusions and the client tutorial

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `MOB-001`, `MOB-004`, `ENV-003`, `WGEN-002`,
`WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, reports, complete loot/recipe/advancement/tag/worldgen
data, exhaustive server/client class-reference sweeps, all 1,212 decoded structure templates and
exact client assets close this property-free ordinary block. Its identity joins tool-independent
self loot, nine-wart compacting, 0.85 composting, slow-sliding equipment, three exact spawn
exclusions, client tutorial completion, Nether-carver replacement, crimson-fungus and
weeping-vines generation, and the Nether surface rule.

**Applies when:**

`minecraft:nether_wart_block` is placed, written, harvested, exploded, crafted, composted,
equipped on a sulfur cube, considered below a Hoglin, Piglin or Zombified Piglin spawn, inspected
by the find-tree tutorial, selected or replaced by Nether generation, persisted, mapped or
rendered.

**Authoritative state:**

Nether wart block is an ordinary property-free `Block` with no block entity and sole state
`14846`. Its locked block protocol ID is `672`, and its block-item raw ID is `604`. Registration
selects map color `COLOR_RED`, the default note instrument `HARP`, hardness/resistance `1/1` and
`WART_BLOCK` sounds. It does not require a correct tool for drops.

The state is a full unit selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`,
solid redstone conduction, normal piston reaction, full sturdy faces and ordinary full-face spawn
support. It adds no random or scheduled tick, use, attack, entity-contact, neighbor, signal,
comparator or block-event override. Its direct block tags are `mineable/hoe` and `wart_blocks`;
the first changes mining speed without gating loot.

The Wart-block sound type has volume/pitch `1/1` and selects sound registry IDs break `1146`, step
`1147`, place `1148`, hit `1149` and fall `1150`. The ordinary block item is common, stacks to
`64`, has standard block-item components and directly belongs to the item `wart_blocks` tag.

**Transition and ordering:**

#### Placement, harvest, compacting and composting

Ordinary placement and authoritative component/command writes always select state `14846`;
rotation and mirror are identity operations. The one-roll loot table offers one matching item
behind `survives_explosion` and uses random sequence `minecraft:blocks/nether_wart_block`.
Tool type, Silk Touch and Fortune do not otherwise alter the table.

The sole recipe is shapeless and consumes exactly nine `minecraft:nether_wart` items to produce one
block. No bundled reverse recipe converts the block back into Nether wart. Its advancement places
`has_nether_wart` and `has_the_recipe` in one OR requirement and grants only this recipe. Generic
grid matching, ingredient consumption, output admission and recipe-book publication remain with
their owners.

`ComposterBlock` registers the block item at Java float chance `0.85f`, widened for the comparison
to `0.8500000238418579`. A player-held insertion at composter level `0` succeeds without an RNG
draw; levels `1..6` consume one `nextDouble()` and increment exactly when the draw is strictly less
than that widened chance. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and
level `6 -> 7` schedules the composter after `20` ticks. Failure keeps the state. In either
level-`0..6` branch the server emits level event `1500` with data `1` on success or `0` on failure,
awards the item-used statistic and calls `consume(1, player)`; client prediction returns success
without authoritative mutation. At level `7`, a compostable held item still returns success but is
not consumed or inserted; level `8` delegates to ordinary item-on-block handling.

Automated insertion admits the item only below level `7`, invokes the same first-level/RNG/state
transition and shrinks the stack by one whether that admitted attempt succeeds or fails. Generic
composter container admission, maturation `7 -> 8`, extraction and client event rendering remain
with the composter and protocol owners.

#### Wart-tag joins and sulfur-cube equipment

The block and item `wart_blocks` tags each contain exactly Nether wart block and warped wart block.
Nested tag closure puts the block in `nether_carver_replaceables` and both block and item identities
in `completes_find_tree_tutorial`; item closure also puts it in
`sulfur_cube_archetype/slow_sliding`.

The `slow_sliding` record fixes horizontal/vertical knockback powers `0.4125/0.09`,
`slow_sliding.hit` and `slow_sliding.push` sounds, push cooldown `1`, impulse threshold `0.02`, and
five attribute entries: additive knockback and explosion-knockback resistance
`0.800000011920929/0.800000011920929`, additive bounciness `0.10000000149011612`,
total-multiplied friction `-0.9499999992549419`, and total-multiplied air drag
`-0.9900000002235174`. Matching order, equipment replacement, modifier application, contact,
knockback math, sounds and entity projection remain with the sulfur-cube and entity owners.

In the client-local `FIND_TREE` tutorial step, looking at the projected block state selects
`PUNCH_TREE`; obtaining its item selects `CRAFT_PLANKS`. On the first survival-mode tutorial tick,
already holding this item or having a positive mined statistic for any completed-tree-tag block,
including this block, likewise selects `CRAFT_PLANKS`. The block is therefore deliberately treated
as tutorial “tree” material through tag closure even though it is not wood. Non-survival instead
selects `NONE`; the generic 6,000-tick toast and tutorial lifecycle remain with `CLI-001`.

#### Mob-spawn exclusions and Nether generation

The registered Hoglin and Piglin placement predicates each reject exactly when the block
immediately below the candidate is Nether wart block and otherwise return true, without reading
the supplied RNG. The Zombified Piglin predicate additionally requires difficulty other than
Peaceful, then applies the same exact below-block rejection, also without RNG. Global category
caps, spawn-placement type, biome lists, collision/light/distance gates, group construction and
finalization remain with `MOB-SPAWN-001`.

The locked Nether surface tree selects state `14846` in crimson forest floor contexts at or above
absolute Y `31`, after the ordinary on-floor/ceiling/biome branches, when netherrack noise is below
`0.54` and wart noise is at least `1.17`. Lower wart noise selects crimson nylium; the corresponding
warped-forest branch selects warped wart block or warped nylium. The complete first-match surface
program and noise ownership remain with `WGEN-PIPELINE-001`.

Both `crimson_fungus` and `crimson_fungus_planted` configure Nether wart block as their hat,
crimson stem as stem and shroomlight as decor; the former is ordinary and the latter sets
`planted=true`. Huge-fungus hat placement tests whether the configured hat is exactly Nether wart
block to enable its crimson weeping-vine branches. Geometry, admission, write order, destruction,
all topology probabilities and RNG stay with `WGEN-PIPELINE-001`.

The standalone `weeping_vines` feature requires an empty origin whose upper neighbor is exact
netherrack or Nether wart block. On admission it offers state `14846` at the origin, then its 200
roof attempts both treat exact netherrack/Nether wart neighbors as support and offer state `14846`
at candidates with exactly one such neighbor. Its later 100 vine candidates likewise require
either exact support immediately above. Candidate offsets, neighbor order, lengths, ages, writes
and RNG remain with `WGEN-PIPELINE-001`.

Through `nether_carver_replaceables`, the Nether-cave material kernel may replace state `14846`
with cave air above its lava boundary or lava at/below it. The tag is reload-selected; full carver
admission, path, mask and ordering remain with `WGEN-PIPELINE-001`. The exhaustive NBT scan finds
zero matching cells in all 1,212 bundled structure templates.

**Client projection:**

The only blockstate variant unconditionally selects `minecraft:block/nether_wart_block`. That model
inherits `cube_all` and maps every face to `minecraft:block/nether_wart_block`; the item selector
points directly to the same model. Authoritative block updates publish state `14846`, inventory
projection uses item ID `604`, material sounds use IDs `1146/1147/1148/1149/1150`, map projection
uses `COLOR_RED`, composter event `1500` retains its existing sound/particle mapping, and the
tutorial consumes the projected block/item plus synchronized tags. This leaf adds no packet field,
acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component placement; every tool and explosion survival; recipe absent/matched/output
blocked and either unlock criterion; player versus automated composter input, levels `0`, `1..6`,
`7`, `8`, success/failure and reload; empty/other/Nether-wart body equipment; tutorial mode,
first-tick inventory/stat, look and item pickup; three species, Peaceful/non-Peaceful and exact/other
below block; crimson surface noise boundaries; ordinary/planted fungus and hat/vine branches;
weeping initial/roof/vine support; carver tag snapshot and replacement; zero template selection;
ordinary state versus block/item/sound/map/model projection; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `14846/672/604`; hardness/resistance `1/1`; sound volume/pitch `1/1`; sound
IDs break/step/place/hit/fall `1146/1147/1148/1149/1150`; emission `0`, dampening `15`, shade
`0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; recipe ratio `9:1`; composter
chance `0.85f`/`0.8500000238418579`, levels `0..8`, maturation delay `20`, event `1500`;
slow-sliding powers `0.4125/0.09`, cooldown `1`, threshold `0.02` and five modifier amounts as
listed above; surface thresholds Y `31`, netherrack `<0.54`, wart `>=1.17`; weeping roof/vine
attempts `200/100`; scanned templates/cells `1212/0`. The ordinary block and three spawn predicates
consume no RNG. Composter and worldgen owners retain the exact streams described above.

**Side effects:**

Ordinary full-block placement/removal and tool-independent self loot; one recipe result/grant;
composter item consumption, optional level write/game event/schedule and level event; reload-selected
slow-sliding equipment and tutorial/tag membership; three spawn vetoes; surface, fungus, weeping
and carver writes; ordinary palette/inventory persistence; Wart-block sounds, red map shading and
opaque cube-all projection.

**Gates:**

World-write and break authority; explosion context; active loot, recipe, advancement, tag,
archetype and worldgen snapshots; recipe output admission; composter level/input and RNG; sulfur
body-equipment admission; client tutorial mode/step; registered species placement pipeline and
difficulty; biome/surface/feature/carver admission; valid registry, map, sound and client-resource
context.

**Boundary cases and quirks:**

Hoe membership speeds mining but no correct-tool gate exists. Nine Nether wart compact one-way.
The first composter level succeeds without RNG; failed admitted player insertions still call
`consume` (so only non-infinite players lose one), and automated insertion always shrinks one.
Player level `7` returns success without either call. All three spawn checks compare the exact block
rather than the wart tag. The tutorial treats this nonwood block and item as tree-completion
material. Huge fungus uses exact hat identity for crimson vines, while the carver uses reloadable
tag membership. Zero template cells does not mean normal Nether generation is absent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.entity.monster.hoglin.Hoglin#checkHoglinSpawnRules`;
`net.minecraft.world.entity.monster.piglin.Piglin#checkPiglinSpawnRules`;
`net.minecraft.world.entity.monster.zombie.ZombifiedPiglin#checkZombifiedPiglinSpawnRules`;
`net.minecraft.world.level.levelgen.feature.HugeFungusFeature#placeHat`;
`net.minecraft.world.level.levelgen.feature.WeepingVinesFeature#place`;
`net.minecraft.client.tutorial.FindTreeTutorialStepInstance`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:nether_wart_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:nether_wart_block`;
`reports/registries.json#minecraft:sound_event/minecraft:block.wart_block.*`;
`reports/minecraft/components/item/nether_wart_block.json`;
`data/minecraft/loot_table/blocks/nether_wart_block.json`;
`data/minecraft/recipe/nether_wart_block.json`;
`data/minecraft/advancement/recipes/building_blocks/nether_wart_block.json`;
`data/minecraft/tags/block/{mineable/hoe,wart_blocks,completes_find_tree_tutorial,nether_carver_replaceables}.json`;
`data/minecraft/tags/item/{wart_blocks,completes_find_tree_tutorial,sulfur_cube_archetype/slow_sliding}.json`;
`data/minecraft/sulfur_cube_archetype/slow_sliding.json`;
`data/minecraft/worldgen/configured_feature/{crimson_fungus,crimson_fungus_planted}.json`;
`data/minecraft/worldgen/noise_settings/nether.json`;
`data/minecraft/{worldgen,structure}/**`;
`assets/minecraft/blockstates/nether_wart_block.json`;
`assets/minecraft/models/block/nether_wart_block.json`;
`assets/minecraft/items/nether_wart_block.json`.

**Test vectors:**

Run `EXP-BLK-064` across identity, ordinary/component writes, every tool and explosion loot, the
one recipe/OR unlock, player and automated composting at every level/draw boundary, slow-sliding
reload/equipment, all tutorial transitions, all three spawn predicates, crimson surface
thresholds, ordinary/planted fungus, every exact weeping support role, Nether-carver membership,
all 1,212 structure inputs, persistence, sounds, map and both models. Assert exact constants,
outputs, RNG order, zero template cells and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, crafting, advancement, composter maturation/extraction,
sulfur-cube equipment/contact/knockback, spawning, surface/carver/feature algorithms, tutorial
lifecycle, packet encoding and client rendering remain with `BLK-PLACE-001`, `PLY-BREAK-001`,
`ITM-LOOT-001`, `ITM-RECIPE-001`, `ITM-ADVANCEMENT-001`, `ENT-KNOCKBACK-001`,
`MOB-SPAWN-001`, `WGEN-PIPELINE-001`, `CLI-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
