# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-MUD-001` — Mud sinks collision, supports growth and bridges water to clay

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`BLK-SNOW-FAMILY-001`, `BLK-DRIPSTONE-BLOCK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`PLY-BREAK-001`, `PLY-MOVE-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-POTION-001`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-DISPENSER-001`,
`ENT-001`, `ENT-KNOCKBACK-001`, `MOB-001`, `MOB-AI-001`,
`MOB-SPAWN-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FLUID-001`, `ENV-FIRE-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`,
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, implementation bytecode, reports,
complete loot/recipe/advancement/tag data, every worldgen record, all
`1,212` decoded structure templates and exact client resources close this
property-free block and item. Its identity-specific runtime is the
`14/16` collision subtype, Water-Potion acquisition, pointed-dripstone
conversion to Clay, tag-selected support/mob/equipment behavior, exact
surface/disk/Mangrove generation and 13 raw structure cells.

**Applies when:**

`minecraft:mud` is placed, collided with, used as support, mined,
exploded, produced by a Water Potion, converted by pointed dripstone,
crafted with, selected by a live block or item tag, generated, persisted,
synchronized or rendered.

**Authoritative state:**

Mud is a property-free `MudBlock` with no block entity and sole block-state
ID `30415`. Its block protocol ID is `1150`; its ordinary block-item raw ID
is `59`.

Registration legacy-copies Dirt, then changes map color to
`TERRACOTTA_CYAN`, installs an always-true local spawn predicate, makes
redstone conduction, view blocking and suffocation false, and selects the
Mud sound type. It retains hardness/resistance `0.5/0.5`, Harp instrument,
friction `0.6`, speed/jump factors `1`, emission `0`, normal piston
reaction and no correct-tool requirement.

The inherited outline and occlusion shape is a full unit cube. The custom
collision shape occupies full X/Z but only Y `0..14/16`, so colliding
entities settle `2/16` below the selected/rendered top. Custom block-support
and visual shapes remain full cubes. All six support faces are therefore
sturdy even though collision is shorter. Shade brightness is fixed at
`0.2`, and land, water and air pathfinding all return false.

The state has no random or scheduled tick, placement-state, neighbor,
shape-update, use, attack, entity-contact, signal, comparator, fluid or
block-event override. Its local valid-spawn predicate returns true for
every entity type and position; space, collision, light, biome and the
caller's other spawn gates remain independently authoritative. It is not a
redstone conductor, view blocker or suffocation state.

Mud's sound profile has volume/pitch `1/1` and uses event IDs break `1005`,
step `1009`, place `1008`, hit `1007` and fall `1006`. Its common stack-64
`BlockItem` has ordinary generic components.

**Transition and ordering:**

### Placement, collision and self loot

Ordinary item placement, component placement and command writes select
state `30415`; rotation and mirror are identity operations. Placement
itself has no support predicate. The shortened collision applies through
generic movement without a Mud-owned slowdown, contact effect, fall reset
or velocity rewrite. Its full support shape remains available to blocks
above.

The direct `mineable/shovel` tag gives a suitable Shovel its mining-speed
path but does not gate loot. After an admitted removal, the one-roll block
table offers one Mud item behind `survives_explosion`, using random sequence
`minecraft:blocks/mud`. Hand, another tool, every Shovel, Silk Touch and
Fortune otherwise share that table. Enderman pickup and worldgen
replacement are separate no-loot mutations.

Mud has no `FireBlock.bootStrap` row, lava-ignition property, fuel time or
Composter entry: direct fire encouragement/flammability are `0/0`.

### Water-Potion acquisition

Mud is the output, not the input, of the exact Water-Potion conversion
owned by `ITM-POTION-001`. The clicked block must be in live
`convertable_to_mud`, whose baseline members are Dirt, Coarse Dirt and
Rooted Dirt; the face must not be `DOWN`. Potion contents must hold Water
and have no custom effects, while custom color and custom name are ignored.

On both logical sides the player path first plays generic splash and
transforms the held stack through the Glass-Bottle filled-result helper.
The server then consumes ten doubles to publish five one-particle splash
packets at `(x+random,y+1,z+random)`, plays Bottle Empty, emits
`FLUID_PLACE`, calls `setBlockAndUpdate` with default Mud and ignores the
write result. Survival consumes one Potion and returns, inserts or drops
the Bottle. An infinite-material player retains the Potion and attempts to
insert a Bottle only when no equal Bottle is already present.

A Potion dispenser applies the same Water predicate to its front
`convertable_to_mud` state. Admission consumes ten doubles for five
particles, plays Bottle Empty, emits `FLUID_PLACE`, offers default Mud and
produces a Glass Bottle through the dispenser remainder transaction. A
nonmatching potion or target runs nested default ejection. Exact dispatch,
residue and outer-event ordering remain with `ITM-DISPENSER-001`.

### Pointed-dripstone conversion to Clay

Mud is the exact source-state selector in the downward pointed-dripstone
Water-transfer branch. After the owning algorithm passes orientation,
source-fluid and at-most-eleven-block tip search, its transfer draw must
fall below `0.17578125`. Exact Mud then selects conversion rather than
cauldron filling.

The server writes default Clay at the Mud source, emits `BLOCK_CHANGE`
there with Clay context, emits level event `1504` at the tip, and returns.
A failed preliminary gate or draw, another source state/fluid, or a
rejected write path leaves the source unchanged as specified by
`BLK-DRIPSTONE-BLOCK-001` and `BLK-CLAY-001`.

### Recipes and advancement knowledge

Two Building-category shapeless recipes consume one exact Mud:

- Mud plus Wheat produces one Packed Mud; and
- Mud plus Mangrove Roots produces one default-state Muddy Mangrove Roots.

Grid order is irrelevant and input component patches are discarded. The
Packed-Mud advancement grants its recipe when exact Mud is possessed or
the recipe is already known, in one OR requirement. The
Muddy-Mangrove-Roots advancement instead tests exact Mangrove Roots or
existing recipe knowledge; Mud possession alone does not unlock it. No
locked recipe produces Mud.

Complete non-block loot and merchant searches find no Mud source.
Water-Potion conversion, block loot, creative publication, worldgen and
commands are its baseline acquisition paths.

### Block tags, support and mob selectors

Mud's seven direct block tags are `frogs_spawnable_on`,
`mangrove_logs_can_grow_through`,
`mangrove_roots_can_grow_through`, `mineable/shovel`, `mud`,
`support_override_snow_layer` and `supports_big_dripleaf`.

The direct `support_override_snow_layer` membership admits a Snow layer
above Mud after the Snow owner's higher-priority
`cannot_support_snow_layer` rejection check. This makes the result explicit
despite Mud's `14/16` collision height. The Big-Dripleaf tag admits leaf
support; a Big Dripleaf Stem still additionally requires a leaf or stem
above. Both Mangrove grow-through tags admit the corresponding trunk/root
preflight.

A Frog natural-spawn candidate accepts Mud immediately below only when raw
brightness at the candidate is strictly greater than `8`. The always-true
registration predicate separately means Mud does not locally reject other
entity types submitted to ordinary full-face spawn validation.

The direct `mud` membership expands into exactly `33` additional ancestors,
for a complete locked closure of `40`:

- `azalea_grows_on`, `azalea_root_replaceable`,
  `beneath_bamboo_podzol_replaceable`,
  `beneath_tree_podzol_replaceable`,
  `cannot_replace_below_tree_trunk`, `enderman_holdable`,
  `forest_rock_can_place_on`, both Huge-Mushroom support tags,
  `ice_spike_replaceable`, `lush_ground_replaceable`,
  `moss_replaceable`, both carver-replaceable tags,
  `sculk_replaceable`, `sculk_replaceable_world_gen`,
  `sniffer_diggable_block` and `substrate_overworld`;
- `supports_azalea`, `supports_bamboo`, `supports_crimson_fungus`,
  `supports_crimson_roots`, `supports_dry_vegetation`,
  `supports_mangrove_propagule`, `supports_melon_stem_fruit`,
  `supports_pumpkin_stem_fruit`, `supports_stem_fruit`,
  `supports_nether_sprouts`, `supports_sugar_cane`,
  `supports_vegetation`, `supports_warped_fungus`,
  `supports_warped_roots` and `supports_wither_rose`.

These live joins select generic plant survival, feature/carver/Sculk
replacement, Sniffer digging, Enderman take/place and the non-Azalea tree
below-provider exception. An admitted empty-handed Enderman under
`mobGriefing` removes Mud without loot, emits `BLOCK_DESTROY` and carries
its sole default state. Sniffer age/water/history/navigation, Enderman
obstruction/placement and all feature/support read, draw and write gates
retain their named owners.

### Item tags and regular Sulfur-Cube equipment

The Mud item directly belongs to item `mud` and
`sulfur_cube_archetype/regular`; the latter is nested by
`sulfur_cube_swallowable`, giving a complete three-tag item closure. The
item `mud` tag has no locked parent, recipe ingredient or code consumer.

The regular archetype is buoyant. It fixes horizontal/vertical knockback
powers `0.4125/0.09`, additive knockback and explosion-knockback resistance
`-1/-1`, additive bounciness `0.5`, total-multiplied friction
`-0.699999988079071`, total-multiplied air drag
`-0.8999999985098839`, hit/push sound IDs `1937/1938`, push cooldown `0.5`
and impulse threshold `0.2`.

An accepting adult Sulfur Cube can install one Mud in empty BODY equipment.
Because the swallowable tag nests regular, an otherwise unregistered
dispenser behavior searches the front AABB and lets the first accepting
Sulfur Cube consume one; when none accepts, protected default ejection
runs. Matching, equipment mutation, attribute lifecycle, contact,
knockback, sound, traversal and dispenser residue remain with
`ENT-KNOCKBACK-001` and `ITM-DISPENSER-001`.

### Surface, disk and Mangrove generation

Five locked noise settings—`overworld`, `large_biomes`, `amplified`,
`caves` and `floating_islands`—each contain two exact default-Mud result
nodes inside the shared ordered surface tree:

- a Mangrove-Swamp branch beneath floor stone-depth and water
  `(offset=-1, addStone=false, multiplier=0)` gates; and
- a deeper Mangrove-Swamp branch beneath water
  `(offset=-6, addStone=true, multiplier=-1)` and floor stone-depth with
  surface depth enabled.

The first three settings gate the shared tree with
`above_preliminary_surface`; Caves and Floating Islands leave it ungated.
Only cells identical to each setting's default block are offered to the
surface rule, and first-non-null ordering can select an earlier result.
Traversal, biome optimization, context caching and writes retain
`WGEN-PIPELINE-001`.

Configured feature `disk_grass` accepts exact Dirt and Mud as disk targets,
uses half-height `2`, uniform radius `2..6`, and a provider that returns
default Dirt unless the cell above has neither a solid state nor Water
fluid, in which case it returns default `snowy=false` Grass Block. Its
placed feature performs count `1`, in-square placement, `OCEAN_FLOOR_WG`
height selection, random X/Z offset `0` and Y offset `-1`, then requires
the origin to be exact Mud and passes the biome filter. Disk traversal,
per-cell predicates and write/result aggregation retain the feature owner.

Both `mangrove` and `tall_mangrove` configured trees can traverse existing
Mud through their grow-through tags. Their root preflight also classifies
Mud as a muddy-root substrate; an admitted staged Mud cell samples the
simple provider for default axis-Y Muddy Mangrove Roots. This branch does
not run the common replacement recheck or above-root Moss-Carpet chance.
Maximum root width/length are `8/15`, skew chance is `0.2`, and
ordinary/tall trunk offsets are inclusive `1..3` and `3..7`.

### Structure-template census

An exhaustive decoded scan of all `1,212` bundled templates finds exactly
13 raw Mud cells in four files, with no block NBT:

- Trail Ruins `tower/hall_3` has five at `[4,1,4]`, `[4,1,5]`,
  `[4,1,6]`, `[5,1,5]` and `[5,1,6]`;
- `tower/hall_4` and `tower/hall_5` each have three at `[4,1,4]`,
  `[5,1,5]` and `[5,1,6]`; and
- Trial Chambers `corridor/addon/display_2` has two at `[2,1,3]` and
  `[3,1,2]`.

The three Trail-Ruins halls are distinct weight-one rigid entries in the
25-weight `trail_ruins/tower/additions` pool and use
`trail_ruins_houses_archaeology`. Its Gravel and Mud-Bricks rules do not
match raw Mud, so admitted Mud cells pass through unchanged. The Trial
payload is one of three equal-weight rigid `trial_chambers/entrance`
entries and uses inline-empty processors.

Exact decompressed-string scanning finds only the four palette strings:
there is no extra Jigsaw `final_state`, processor field, block NBT or entity
payload naming exact Mud. Pool reachability, shuffling, attachment,
rotation, overlap, clipping, processor admission and final writes retain
the two Jigsaw owners; raw cells are not guaranteed final-world writes.

### Persistence and client projection

Block persistence and terrain packets preserve only state identity.
Stacks preserve generic components. Mud has no pre-flattening numeric
block/item mapping, old alias or identity-specific data-fix path.

The sole blockstate variant selects `minecraft:block/mud`. Its `cube_all`
model maps every face to the static untinted 16×16
`minecraft:block/mud` texture. The item definition points directly to that
block model. The client renders the full cube model even though server
collision ends at `14/16`.

Its English name is `Mud`. Natural Blocks publishes it once after Farmland
and before Clay, in the local order Dirt, Coarse Dirt, Rooted Dirt,
Farmland, Mud, Clay, Gravel, Sand. It appears in no other baseline creative
tab.

**Branches and aborts:**

- Placement has no support gate; movement uses shortened collision without
  a Mud contact callback.
- Every tool may harvest self loot; explosion survival can suppress it.
- Water-Potion player/dispenser conversion checks face, live tag and exact
  contents; both ignore the Mud write result after admission.
- Drip conversion requires the complete downward Water-transfer path and
  strict threshold; other sources retain cauldron behavior.
- Recipe matching and recipe-knowledge criteria differ between the two
  Mud inputs.
- Reload can change block/item selector admission without mutating an
  existing state.
- Surface first-match, feature/provider, tree and Jigsaw gates can reject
  every offered Mud output.

**Constants and randomness:**

State/block/item IDs `30415/1150/59`; strength/resistance `0.5/0.5`;
collision height `14/16`; shade `0.2`; friction `0.6`; speed/jump `1`;
sound IDs break/step/place/hit/fall `1005/1009/1008/1007/1006`;
stack `64`; Potion particles/doubles `5/10`; drip threshold `0.17578125`;
block/item closures `40/3`; regular-archetype values as listed; disk
half-height/radius `2/2..6`; Mangrove width/length/skew `8/15/0.2`;
structure files/cells `4/13`, with pool weights `3 of 25` and `1 of 3`.

**Side effects:**

Block placement/removal, lowered collision and full support; explosion-gated
self loot; Potion stack/Bottle, particles, sounds, event and Mud write;
dripstone Clay write and game/level events; crafting/results/knowledge;
Snow/plant/mob/equipment/dispenser selection; surface, disk, Mangrove and
Jigsaw writes; state/stack persistence; sounds, map color, model, texture,
name and creative-tab projection.

**Gates:**

World-write and break authority; collision context; explosion survival;
Potion face/content/live target tag and inventory capacity; downward
dripstone orientation/source/fluid/tip/draw; recipe ingredients/output and
knowledge; live block/item tags; Snow priority, Frog brightness, mob AI/
game rule/equipment; surface default-state/biome/water/depth/order;
feature/tree/provider/predicate; Jigsaw reachability/transform/clip/write;
registry, reload and client-resource validity.

**Boundary cases and quirks:**

Mud is selected and rendered as a full cube but collides two model units
short. Its full block-support shape and explicit Snow override still
support ordinary sturdy-face consumers and Snow layers. It accepts every
submitted entity type at its local spawn predicate while remaining a
nonconductor, non-view-blocker and nonsuffocating state. Mud possession
unlocks Packed Mud but not Muddy Mangrove Roots. Raw Trail-Ruins Mud is
unaffected by that element's archaeology rules. The client model does not
show the collision depression.

**Failure semantics:**

Generic placement, break and crafting transactions retain their owners'
commit behavior. Admitted player/dispenser Potion conversion ignores the
Mud write result after publishing its side effects. Dripstone, feature,
tree and Jigsaw algorithms retain their documented rejected-write and
partial-commit semantics. Tag reload affects future reads only.

**Client/server authority split:**

The server owns collision/support, placement, break/loot, Potion and
dripstone mutations, crafting/progression, tag/mob/equipment selection,
generation and persistence. The client predicts ordinary placement and the
Potion interaction result, consumes synchronized state/sound/particle
outputs, and renders the full model, texture, name and tab entry.

**Observability:**

Observe state/block/item/sound IDs, collision versus outline/visual/support
shapes, path/spawn/conductor/view/suffocation predicates, mining and loot,
Potion order/RNG/remainders/write failure, drip conversion, recipes and
knowledge, complete tag closures and consumers, every worldgen
read/draw/write, exact 13-cell template census, save/wire identity and
client projection.

**Persistence and reload:**

Mud saves one property-free identity and has no block entity. Its stack uses
generic components. Tags, loot, recipes, advancements, worldgen and client
resources have independent reload boundaries. Registration, subtype
shapes, Potion/dripstone exact-state branches and creative ordering are
code-built.

**Evidence:**

`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.MudBlock`;
`net.minecraft.world.item.PotionItem`;
`net.minecraft.core.dispenser.DispenseItemBehavior$13`;
`net.minecraft.world.level.block.PointedDripstoneBlock`;
`net.minecraft.world.entity.monster.EnderMan$EndermanTakeBlockGoal`;
`net.minecraft.world.entity.SulfurCubeArchetypes`;
`net.minecraft.world.item.CreativeModeTabs`; block/item/sound/component
reports; block loot, two recipes/advancements; complete block/item tag
closure and regular archetype; all five noise settings, disk and Mangrove
records; both template pools/processors; all `1,212` decoded templates and
decompressed strings; exact blockstate/model/item/texture/language
resources. Complete compiled exact-field, data, legacy-fix and decoded-NBT
searches find no other identity-specific runtime path.

**Test vectors:**

Run `EXP-BLK-117` across state/registry identity, every placement,
shape/collision/support/path/spawn/redstone/tool/explosion branch, player
and dispenser Water-Potion transactions, every pointed-dripstone gate,
both recipes/unlocks, complete 40/3 tag closures and all consumers, regular
Sulfur-Cube equipment/dispenser behavior, both surface paths in all five
settings, disk and Mangrove generation, all 13 raw template cells,
persistence/reload and exact client projection. Assert IDs, ordering,
constants, absences, census and vanilla convergence.

**Limits:**

Generic placement/break/movement, loot, Potion containers, crafting/
progression, Snow, dripstone/Clay, plant/mob/Sulfur-Cube behavior,
surface/feature/tree/Jigsaw generation, packet encoding and rendering
retain their named owners. Dirt substrates, Packed Mud, Muddy Mangrove
Roots, Clay and vegetation retain their catalog families. This leaf fixes
exact Mud and every direct join that selects it.
