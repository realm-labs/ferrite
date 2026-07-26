# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SMOOTH-STONE-001` — Smooth stone joins double smelting, slabs, blast furnaces and village sources

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-005`, `PLY-006`, `PLY-BREAK-001`, `BLK-003`,
`BLK-005`, `BLK-UPDATE-001`, `PLY-002`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-FURNACE-001`, `ITM-STONECUTTER-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-005`, `ENT-KNOCKBACK-001`,
`MOB-AI-001`, `WGEN-003`, `WGEN-JIGSAW-PROCESSORS-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked ordinary-block registration and reports, complete
loot/recipe/advancement/tag/class-reference searches, village chest table, all 1,212 decoded
templates, owning village pools/processors and exact client assets exhaust this identity. The
Stronghold class named `SmoothStoneSelector` and `BlackstoneReplaceProcessor` contain no reference
to the full Smooth Stone field; similarly named slab paths are included only where they are real
recipe descendants.

**Applies when:**

`minecraft:smooth_stone` is placed, mined, exploded, smelted from Stone, crafted or cut into a
slab, consumed by a Blast Furnace recipe, selected as village loot, equipped on a Sulfur Cube,
placed by a village template, persisted, mapped or rendered.

**Authoritative state:**

Smooth Stone is a property-free ordinary `Block` with no block entity. Its sole default state is
`13480`, its block protocol ID is `624`, and its common ordinary block-item raw ID is `331`.

Registration fixes map color `STONE`, note instrument `BASEDRUM`,
`requiresCorrectToolForDrops`, hardness/resistance `2/6` and ordinary Stone sounds. The state is a
full unit selection/collision/visual/occlusion cube with emission `0`, light dampening `15`, shade
brightness `0.2`, friction `0.6`, speed/jump factors `1`, restitution `0`, solid redstone
conduction, normal piston reaction, full sturdy faces and ordinary spawn support. It adds no
random or scheduled tick, use, attack, contact, neighbor, signal, comparator, fluid or block-event
override.

Its sole direct block tag is `mineable/pickaxe`; no minimum-tier tag contains it. Every Pickaxe is
therefore correct, while hand and other tools can remove the state but fail loot admission. Stone
sound volume/pitch is `1/1`, with exact event IDs break `1596`, step `1604`, place `1601`, hit
`1600` and fall `1599`.

The item stacks to `64`, has only standard block-item components and directly belongs to
`sulfur_cube_archetype/slow_bouncy`.

**Transition and ordering:**

### Placement, transforms and self loot

Placement, explicit writes, rotation and mirror all retain sole state `13480`. No legal block-state
component can add a property.

After successful survival removal, any Pickaxe reaches the one-roll table. It offers one Smooth
Stone behind `survives_explosion`, using random sequence `minecraft:blocks/smooth_stone`; Silk
Touch and Fortune add no branch. Wrong-tool player removal emits nothing, and an admitted
explosion can independently suppress the self entry.

### Furnace production and recipe knowledge

The sole full-block producer is a Furnace-only smelting recipe: one exact Stone becomes one
default Smooth Stone after default `200` cooking ticks and grants `0.1` experience per recipe
application. Input component patches are discarded. There is no Blasting, Smoking, Campfire or
Stonecutter producer and no direct Cobblestone-to-Smooth-Stone step; ordinary acquisition therefore
requires the separately owned Cobblestone-to-Stone transition first. Stone possession or prior
knowledge unlocks only this smelting record.

Smooth Stone is then an exact input to four records:

- three in one shaped row produce six Smooth Stone Slabs;
- one Stonecutter input produces two Smooth Stone Slabs;
- five Iron Ingots, one Furnace and three Smooth Stone in rows `III/IXI/###` produce one Blast
  Furnace; and
- the slab descendant combines with six Sticks in `///`, ` / `, `/_/` to produce one Armor Stand.

The two slab records unlock from Smooth Stone or their own prior knowledge. The Blast Furnace
record does likewise. Armor Stand unlocks from the slab, not the full block. Shape offsets,
Stonecutter publication, Furnace fuel/progress, output admission, result components and
consumption remain with generic owners; slab state, merging, waterlogging and loot remain with
`shape-family`.

No recipe returns a slab to the full block, and no other locked recipe consumes Smooth Stone.

### Village chest acquisition

`chests/village/village_mason` performs an inclusive uniform `1..5` rolls over entries with total
weight `13`. Smooth Stone has weight `1`, emits exactly one unpatched item on a selected roll and
can repeat because rolls select with replacement. The pool's random sequence is
`minecraft:chests/village/village_mason`. Chest-table selection, seed, container fill order,
collision and overflow remain with the loot and village owners.

No entity table, archaeology table, trade, gift or optional built-in pack names the identity.

### Slow-bouncy Sulfur-Cube equipment

The item directly selects `slow_bouncy`. Its record fixes horizontal/vertical knockback powers
`0.4125/0.24`, hit/push sounds, push cooldown `0.5`, impulse threshold `0.05`, additive knockback
and explosion-knockback resistance `0.4000000059604645/0.4000000059604645`, additive bounciness
`0.6000000238418579`, total-multiplied friction `-0.699999988079071` and total-multiplied air drag
`-0.949999999254942`.

Matching order, equipment replacement, modifier lifecycle, buoyancy, contact, knockback, sound and
entity projection remain with the Sulfur-Cube/entity owners. Reload changes future classification
without mutating placed states.

### Village template payload

The exhaustive scan finds `59` raw Smooth Stone cells in five of all `1,212` templates:

| Village template | Cells |
|---|---:|
| `plains/houses/plains_armorer_house_1` | 8 |
| `plains/houses/plains_tannery_1` | 1 |
| `savanna/houses/savanna_tannery_1` | 1 |
| `savanna/houses/savanna_weaponsmith_2` | 48 |
| `snowy/houses/snowy_butchers_shop_1` | 1 |

All five are reachable with weight `2` in both the matching ordinary and zombie house pool,
yielding ten pool entries over the same five payloads. Ordinary Plains entries use
`mossify_10_percent`; ordinary Savanna and Snowy entries use inline-empty processors. Zombie
entries use `zombie_plains`, `zombie_savanna` or `zombie_snowy`. None of those lists targets
Smooth Stone and none installs live-target protection, so the source state passes the processor
boundary unchanged.

Terrain projection, rotation, overlap, clip, placement admission and later mutation remain with
`WGEN-JIGSAW-VILLAGES-001`; the `59` source cells are not guaranteed final-world writes.

The exact direct data search has 12 JSON files: self loot; four recipes; four recipe advancements;
the Pickaxe and slow-bouncy tags; and the village Mason chest table. Outside registrations, data
generators and generic publication, the class-reference sweep finds no identity-specific runtime
consumer. No configured/placed feature, code-built structure, processor output or additional
template names full Smooth Stone.

**Client projection:**

The property-free blockstate selects `block/smooth_stone`. That model inherits `block/cube_all`
with texture `block/smooth_stone`; the item selects the same block model directly.

English translation is `Smooth Stone`. The Building Blocks tab publishes it once after Mossy
Cobblestone Wall and before Smooth Stone Slab, followed by Stone Bricks. Block updates use state
`13480`, inventory paths use item ID `331`, sounds use IDs `1596/1604/1601/1600/1599`, and maps
use `STONE`. This identity adds no packet field or connection-local state.

**Branches and aborts:**

Sole state; ordinary versus explicit/template write; Pickaxe versus wrong tool; ordinary/explosion
loot; Furnace match/fuel/progress/output/XP; full-block, slab, Blast-Furnace and downstream
Armor-Stand recipes/unlocks; Mason chest rolls; current/reloaded slow-bouncy selection; five
templates through ordinary/zombie pools and five processor modes; every transform/overlap/clip/
write result; save/reload and block/item projection are distinct.

**Constants and randomness:**

State/block/item IDs `13480/624/331`; strength `2/6`; emission `0`, dampening `15`, shade `0.2`,
friction `0.6`, speed/jump `1`, restitution `0`; sound break/step/place/hit/fall IDs
`1596/1604/1601/1600/1599`, volume/pitch `1/1`; stack `64`; Furnace input/output/time/XP
`1/1/200/0.1`; shaped/Stonecutter slab ratios `3:6/1:2`; Blast Furnace inputs `5/1/3`;
Mason chest rolls `1..5`, Smooth-Stone/total weight `1/13`; template files/cells `5/59`, ten
weight-two pool entries; slow-bouncy values as listed. The block consumes no RNG; loot, cooking,
equipment and village owners retain their streams.

**Side effects:**

Full-block placement/removal; correct-tool/explosion-gated self loot; Furnace result/XP and recipe
knowledge; slab, Blast-Furnace and downstream Armor-Stand results; Mason chest stacks;
reload-selected slow-bouncy equipment; village source writes; ordinary persistence, Stone maps,
sounds and opaque cube projection.

**Gates:**

World-write/break authority; correct Pickaxe and explosion survival; recipe/advancement snapshot;
Furnace fuel/progress/output; grid/Stonecutter result admission; chest table/container admission;
live archetype tag; village pool, processor, terrain, overlap, clip and write admission; valid
registry/map/sound/client-resource context.

**Boundary cases and quirks:**

Smooth Stone requires two Furnace transitions from Cobblestone because its producer accepts exact
Stone only. Blasting cannot accelerate the second transition. Every Pickaxe is correct without a
tier gate. Slab crafting doubles count while Stonecutting also returns two per block. Armor Stand
uses the slab rather than full Smooth Stone. Village zombie processors degrade other materials but
leave all 59 Smooth Stone source cells intact. The Stronghold selector's historical class name is
not evidence that it places this block.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:smooth_stone`;
`reports/registries.json#minecraft:{block,item}/minecraft:smooth_stone`;
`reports/registries.json#minecraft:sound_event/minecraft:block.stone.*`;
`reports/minecraft/components/item/smooth_stone.json`;
`data/minecraft/loot_table/blocks/smooth_stone.json`;
`data/minecraft/recipe/{smooth_stone,smooth_stone_slab,smooth_stone_slab_from_smooth_stone_stonecutting,blast_furnace}.json`;
`data/minecraft/advancement/recipes/{building_blocks/{smooth_stone,smooth_stone_slab,smooth_stone_slab_from_smooth_stone_stonecutting},decorations/blast_furnace}.json`;
`data/minecraft/loot_table/chests/village/village_mason.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/tags/item/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/sulfur_cube_archetype/slow_bouncy.json`;
`data/minecraft/worldgen/template_pool/village/{plains,savanna,snowy}/{houses,zombie/houses}.json`;
`data/minecraft/worldgen/processor_list/{mossify_10_percent,zombie_plains,zombie_savanna,zombie_snowy}.json`;
`data/minecraft/structure/village/{plains,savanna,snowy}/houses/*.nbt`;
`assets/minecraft/blockstates/smooth_stone.json`;
`assets/minecraft/models/block/smooth_stone.json`;
`assets/minecraft/items/smooth_stone.json`;
`assets/minecraft/lang/en_us.json`.

**Test vectors:**

Run `EXP-BLK-093` across placement and every Pickaxe/wrong-tool/ordinary/explosion break; exact
Furnace, slab, Blast-Furnace and downstream Armor-Stand recipe/unlock paths; Mason chest rolls;
slow-bouncy reload/equipment; all 1,212 templates and the ten ordinary/zombie pool paths through
every processor/transform/overlap/clip/write branch; persistence, IDs, sounds, map and exact
projection. Assert exact constants, the `5/59` raw census and vanilla-client convergence.

**Limits:**

Generic placement, breaking, loot, cooking, crafting, Stonecutting, advancements, slab behavior,
container loot, Sulfur-Cube behavior, jigsaw/village processing, packet encoding and rendering
remain with `BLK-PLACE-001`, `PLY-BREAK-001`, `ITM-LOOT-001`, `ITM-FURNACE-001`,
`ITM-RECIPE-001`, `ITM-STONECUTTER-001`, `ITM-ADVANCEMENT-001`, `shape-family`,
`ENT-KNOCKBACK-001`, `WGEN-JIGSAW-PROCESSORS-001`, `WGEN-JIGSAW-VILLAGES-001`,
`PROTO-PLAY-CLIENTBOUND-BLOCK-001`, `PROTO-PLAY-CLIENTBOUND-SOUND-001` and `CLI-006`. This leaf
fixes the exact Smooth-Stone identity, processing and acquisition joins, absences and projection.
