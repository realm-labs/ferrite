# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-WARPED-WART-BLOCK-001` — Warped wart blocks join composting, warped growth and the client tutorial

**Parent:** `SIM-003`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`,
`ITM-004`, `ITM-006`, `ENT-001`, `ENT-005`, `MOB-001`, `MOB-004`, `ENV-003`, `WGEN-002`,
`WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, reports, complete loot/tag/worldgen data, exhaustive
server/client class-reference sweeps, all 1,212 decoded structure templates and exact client assets
close this property-free ordinary block. Its identity joins tool-independent self loot, 0.85
composting, slow-sliding equipment, client tutorial completion, Nether-carver replacement, warped
surface/fungus generation and twisting-vines support. No bundled recipe, advancement, structure
cell or mob-spawn exact-identity consumer exists.

**Applies when:**

`minecraft:warped_wart_block` is placed, written, harvested, exploded, composted, equipped on a
sulfur cube, inspected by the find-tree tutorial, selected or replaced by Nether generation,
persisted, mapped or rendered.

**Authoritative state:**

Warped wart block is an ordinary property-free `Block` with no block entity and sole state
`20959`. Its locked block protocol ID is `868`, and its block-item raw ID is `605`. Registration
selects map color `WARPED_WART_BLOCK`, the default note instrument `HARP`,
hardness/resistance `1/1` and `WART_BLOCK` sounds. It does not require a correct tool for drops.

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

#### Placement, harvest and composting

Ordinary placement and authoritative component/command writes always select state `20959`;
rotation and mirror are identity operations. The one-roll loot table offers one matching item
behind `survives_explosion` and uses random sequence `minecraft:blocks/warped_wart_block`.
Tool type, Silk Touch and Fortune do not otherwise alter the table. No bundled recipe consumes or
produces warped wart block, and no recipe advancement references it.

`ComposterBlock` registers the item at Java float chance `0.85f`, widened for comparison to
`0.8500000238418579`. A player-held insertion at composter level `0` succeeds without RNG;
levels `1..6` consume one `nextDouble()` and increment exactly when the draw is strictly below
that chance. Success writes level plus one with flags `3`, emits `BLOCK_CHANGE`, and level
`6 -> 7` schedules the composter after `20` ticks; failure preserves state. In either level-`0..6`
branch the server emits event `1500` with success data, awards the item-used statistic and calls
`consume(1, player)`, which does not shrink infinite-material holders. Level `7` returns success
without insertion/consumption, and level `8` delegates to ordinary item-on-block handling.

Automated insertion admits the item only below level `7`, invokes the same first-level/RNG/state
transition and always shrinks the stack by one, including after chance failure. Generic composter
container admission, maturation `7 -> 8`, extraction and client event rendering remain with the
composter and protocol owners.

#### Wart-tag, tutorial and sulfur-cube joins

The block and item `wart_blocks` tags each contain exactly Nether wart block and warped wart block.
Nested closure puts the block in `nether_carver_replaceables` and both block/item identities in
`completes_find_tree_tutorial`; item closure also puts it in
`sulfur_cube_archetype/slow_sliding`.

The `slow_sliding` record fixes horizontal/vertical knockback powers `0.4125/0.09`,
`slow_sliding.hit` and `slow_sliding.push`, cooldown `1`, impulse threshold `0.02`, additive
knockback/explosion-knockback resistance `0.800000011920929/0.800000011920929`, additive
bounciness `0.10000000149011612`, total-multiplied friction `-0.9499999992549419`, and
total-multiplied air drag `-0.9900000002235174`. Equipment replacement, composed modifiers,
contact, knockback, sounds and entity projection remain with their owners.

In the client-local `FIND_TREE` tutorial, looking at projected state `20959` selects `PUNCH_TREE`;
obtaining item `605` selects `CRAFT_PLANKS`. The first survival-mode tick also selects
`CRAFT_PLANKS` when the item is already held or any completed-tree-tag block, including this one,
has a positive mined statistic. Non-survival selects `NONE`; toast timing and tutorial lifecycle
remain with `CLI-001`.

#### Warped generation and exact-identity absences

The Nether surface tree selects state `20959` in warped-forest floor contexts at or above absolute
Y `31` when netherrack noise is below `0.54` and wart noise is at least `1.17`; lower wart noise
selects warped nylium. The corresponding crimson branch selects Nether wart block or crimson
nylium. Full first-match precedence and noise ownership remain with `WGEN-PIPELINE-001`.

Both `warped_fungus` and `warped_fungus_planted` configure warped wart block as their hat, warped
stem as stem and shroomlight as decor; the latter sets `planted=true`. Huge-fungus placement still
consumes its topology draws, but its exact `hatState.is(NETHER_WART_BLOCK)` test is false. The
retained vine draw is therefore compared with threshold zero and never invokes the weeping-vine
helper. Geometry, destruction, probabilities, draws and writes remain with
`WGEN-PIPELINE-001`.

`twisting_vines` admits an empty origin only when the exact lower block is netherrack, warped
nylium or warped wart block. Its randomized ground searches reapply that same exact support gate
before producing twisting-vines body/head states; the feature does not itself write warped wart
block. Search, length, age, write and RNG behavior remain with `WGEN-PIPELINE-001`.

Through `nether_carver_replaceables`, the Nether-cave material kernel may replace state `20959`
with cave air above its lava boundary or lava at/below it. The exhaustive NBT scan finds zero
matching cells across all 1,212 structure templates.

The server class-reference sweep finds no warped-wart-block identity in Hoglin, Piglin or
Zombified Piglin spawn predicates; each vetoes only exact Nether wart block. Warped wart block
therefore retains ordinary full-face support and the generic/entity-specific gates without that
additional veto.

**Client projection:**

The sole blockstate variant selects `minecraft:block/warped_wart_block`. Its `cube_all` model maps
every face to `minecraft:block/warped_wart_block`, and the item selector points to the same model.
Block updates publish state `20959`, inventory projection uses item ID `605`, material sounds use
IDs `1146..1150`, map projection uses `WARPED_WART_BLOCK`, and synchronized tags feed the
client-local tutorial. Composter event `1500` retains its existing sound/particle owner. This leaf
adds no packet field, acknowledgement or connection-local state.

**Branches and aborts:**

Ordinary/component placement; every tool and explosion survival; recipe absence; finite/infinite
player versus automated composting at levels `0`, `1..6`, `7`, `8`; slow-sliding reload/equipment;
tutorial mode, first-tick inventory/stat, look and pickup; warped surface thresholds;
ordinary/planted fungus and false crimson-vine identity; twisting-vines exact/other support;
carver snapshot/replacement; three spawn-predicate absence probes; zero structure selection;
ordinary state versus block/item/sound/map/model projection; save/reload are distinct branches.

**Constants and randomness:**

State/block/item IDs `20959/868/605`; hardness/resistance `1/1`; sound volume/pitch `1/1`; sound
IDs break/step/place/hit/fall `1146/1147/1148/1149/1150`; emission `0`, dampening `15`, shade
`0.2`, friction `0.6`, speed/jump `1`, restitution `0`, stack `64`; composter chance
`0.85f`/`0.8500000238418579`, maturation delay `20`, event `1500`; slow-sliding constants as
listed above; surface thresholds Y `31`, netherrack `<0.54`, wart `>=1.17`; scanned
templates/cells `1212/0`. The ordinary block consumes no RNG. Composter and worldgen owners retain
the exact conditional streams.

**Side effects:**

Ordinary placement/removal and tool-independent self loot; composter call/optional write,
game event, schedule and level event; reload-selected slow-sliding/tutorial/carver membership;
warped surface and fungus writes; twisting-vines admission; ordinary persistence; Wart sounds,
warped-wart map shading and opaque cube-all projection.

**Gates:**

World-write and break authority; explosion context; active loot/tag/archetype/worldgen snapshots;
composter level/input/RNG and infinite-material policy; sulfur body equipment; tutorial mode/step;
biome/surface/feature/carver admission; valid registry, map, sound and client-resource context.

**Boundary cases and quirks:**

Hoe membership speeds mining but does not gate loot. There is no bundled compacting or reverse
recipe. Level-zero composting has no draw; failed player attempts still call `consume`, while
automation always shrinks one. The tutorial treats the nonwood identity as tree material. Warped
fungus uses it as a hat but exact Nether-wart-only logic prevents hanging weeping vines. Twisting
vines use it only as support. The three piglin-family spawn vetoes do not apply. Zero structure
cells does not mean normal Nether generation is absent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.SoundType`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.level.block.ComposterBlock#useItemOn`;
`net.minecraft.world.level.block.ComposterBlock#insertItem`;
`net.minecraft.world.level.block.ComposterBlock#addItem`;
`net.minecraft.world.item.ItemStack#consume`;
`net.minecraft.world.level.levelgen.feature.HugeFungusFeature#placeHat`;
`net.minecraft.world.level.levelgen.feature.TwistingVinesFeature#place`;
`net.minecraft.client.tutorial.FindTreeTutorialStepInstance`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`reports/blocks.json#minecraft:warped_wart_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:warped_wart_block`;
`reports/registries.json#minecraft:sound_event/minecraft:block.wart_block.*`;
`reports/minecraft/components/item/warped_wart_block.json`;
`data/minecraft/loot_table/blocks/warped_wart_block.json`;
`data/minecraft/tags/block/{mineable/hoe,wart_blocks,completes_find_tree_tutorial,nether_carver_replaceables}.json`;
`data/minecraft/tags/item/{wart_blocks,completes_find_tree_tutorial,sulfur_cube_archetype/slow_sliding}.json`;
`data/minecraft/sulfur_cube_archetype/slow_sliding.json`;
`data/minecraft/worldgen/configured_feature/{warped_fungus,warped_fungus_planted}.json`;
`data/minecraft/worldgen/noise_settings/nether.json`;
`data/minecraft/{recipe,advancement,structure}/**`;
`assets/minecraft/blockstates/warped_wart_block.json`;
`assets/minecraft/models/block/warped_wart_block.json`;
`assets/minecraft/items/warped_wart_block.json`.

**Test vectors:**

Run `EXP-BLK-065` across identity, writes, every tool/explosion loot, recipe/advancement absence,
finite/infinite player and automated composting at every level/draw boundary, slow-sliding reload,
tutorial transitions, three spawn-veto absences, warped surface thresholds, both fungus modes and
false crimson-vine identity, every twisting support branch, Nether carver, all 1,212 templates,
persistence, sounds, map and models. Assert exact constants, RNG order, absences and client
convergence.

**Limits:**

Generic placement, breaking, loot, composter maturation/extraction, sulfur-cube equipment,
spawning, surface/carver/feature algorithms, tutorial lifecycle, packet encoding and rendering
remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ENT-KNOCKBACK-001`,
`MOB-SPAWN-001`, `WGEN-PIPELINE-001`, `CLI-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`.
