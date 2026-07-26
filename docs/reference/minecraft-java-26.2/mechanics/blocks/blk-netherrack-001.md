# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NETHERRACK-001` — Netherrack converts to Nylium and anchors Nether terrain, fire and replacement features

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`,
`BLK-STATE-001`, `BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`,
`PLY-002`, `PLY-005`, `PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`PLY-COLLISION-001`, `PLY-AUTOJUMP-001`, `ITM-003`, `ITM-004`,
`ITM-006`, `ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-FURNACE-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-SMITHING-TEMPLATE-001`,
`ENT-KNOCKBACK-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-FIRE-001`, `ENV-FLUID-001`, `ENV-LIGHT-001`, `WGEN-003`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-RUINED-PORTAL-001`,
`WGEN-JIGSAW-BASTION-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration/reports, complete `NetherrackBlock`
control flow, loot/recipe/advancement/tag/worldgen data, exact compiled identity and
tag-consumer searches, all 1,212 decoded templates, inherited Jigsaw-final handling
and exact client resources close the sole state and item.

**Applies when:**

`minecraft:netherrack` is placed, bonemealed, mined, exploded, smelted, consumed by
template duplication, selected by fire/plant/Sculk/carver/feature predicates, produced
by terrain, Nylium decay or Ruined Portals, carried as Sulfur-Cube equipment,
persisted, synchronized or rendered.

**Authoritative state:**

Netherrack is a property-free `NetherrackBlock` without a block entity:

| State | Block protocol ID | Item raw ID | Map color | Instrument |
|---:|---:|---:|---|---|
| `6997` | `285` | `387` | `NETHER` | `BASEDRUM` |

Its registration fixes hardness/resistance `0.4/0.4`,
`requiresCorrectToolForDrops` and Netherrack sounds. The state is a full
`0..16` selection/collision/visual/occlusion cube with emission `0`, light
dampening `15`, shade brightness `0.2`, friction `0.6`, speed/jump factors `1`,
restitution `0`, full sturdy faces, ordinary spawn support, solid redstone
conduction and default `NORMAL` piston reaction. It holds no fluid and produces
no signal or comparator output.

Sound volume/pitch is `1/1`; break/step/place/hit/fall event IDs are
`1156/1157/1158/1159/1160`. The common nondamageable block item stacks to `64`
and has only standard block-item components.

The block directly belongs to `base_stone_nether`, `infiniburn_overworld`,
`mineable/pickaxe` and `supports_wither_rose`. Its item directly belongs only to
`sulfur_cube_archetype/slow_bouncy`. No minimum-tier tag is present, so every
Pickaxe is correct. There is no FireBlock ignite/burn-odds row, fuel time,
Composter entry or other direct block/item tag.

**Transition and ordering:**

### Placement, ticks and transforms

Ordinary placement, explicit writes, rotation and mirror retain sole state `6997`.
Netherrack has no random-tick, scheduled-tick, use, attack, contact, neighbor,
signal, comparator, fluid or block-event override. Its only custom transaction is
the `BonemealableBlock` path below.

### Bone Meal conversion to Nylium

Target admission first reads `origin.above()` and returns false immediately unless
that live state reports `propagatesSkylightDown()`. It performs no build-height or
air-identity test. Admission then scans the inclusive
`origin+(-1,-1,-1)..(+1,+1,+1)` box in X-fastest, then Y, then Z order. The first
live member of the reloadable `nylium` block tag returns true; otherwise all 27
positions are read and the target is rejected.

Success is unconditional and consumes no success draw. On the server, execution
rescans the same ordered box, sets independent flags when it sees exact Warped or
Crimson Nylium, and stops as soon as both flags are true:

- Warped only offers default Warped Nylium.
- Crimson only offers default Crimson Nylium.
- Both consume one `nextBoolean()`; true chooses Warped and false Crimson.
- Neither exact identity performs no write.

The selected state is offered at the original position with flags `3`; the Boolean
write result is ignored. Execution does not reread the above propagation gate.
Consequently, a data pack can add a third block to `nylium`: it can admit and
consume Bone Meal but cannot select a color unless one of the two exact vanilla
states is also in the box.

Generic server use shrinks the Bone Meal by one after every admitted target,
regardless of conversion or write success, emits the finished-interaction vibration
and level event `1505` at the clicked position, and returns server success. The
block's bonemealable type is `NEIGHBOR_SPREADER`; the event therefore projects
Happy-Villager growth particles around the position above the clicked block.
Client target validation predicts success but does not mutate the stack or world.

Nylium owns the reverse transition. A selected Crimson or Warped Nylium random tick
whose upward ingress dampening is `15` offers state `6997` through
`setBlockAndUpdate`, consumes no RNG and ignores the result. Lower dampening retains
Nylium. Thus Bone Meal renews either color from Netherrack, while opaque cover can
return either color to Netherrack.

### Breaking, loot and furnace conversion

After successful survival removal, any Pickaxe reaches the one-roll block table.
It offers one Netherrack behind `survives_explosion`, with random sequence
`minecraft:blocks/netherrack`. Hand and wrong-tool player removal emits nothing.
Silk Touch and Fortune add no branch; an admitted explosion can independently
suppress the single entry.

The sole cooking record is category-less Furnace smelting:

```text
1 Netherrack -> 1 Nether Brick, 200 ticks, 0.1 recipe XP
```

The omitted cooking time uses the Smelting serializer default `200`. Blast
Furnaces, Smokers and Campfires reject the record. Possessing exact Netherrack or
already knowing the recipe satisfies the recipe advancement's OR requirement.
The emitted Nether Brick is a default stack; arbitrary input components are lost.
No recipe emits Netherrack.

Netherrack is also the center material `C` in both shaped template-duplication
records:

```text
#S#
#C#
###
```

Here `#` is Diamond and `S` is respectively the Netherite-Upgrade or Rib template.
Seven Diamonds, one matching template and one Netherrack produce two default copies.
Possessing Netherrack alone unlocks neither recipe; the matching template or prior
recipe knowledge owns each unlock.

The item selects the reloadable `slow_bouncy` Sulfur-Cube archetype. That record
fixes horizontal/vertical knockback powers `0.4125/0.24`; additive knockback and
explosion-knockback resistance `0.4000000059604645/0.4000000059604645`;
additive bounciness `0.6000000238418579`; total-multiplied friction and air drag
`-0.699999988079071/-0.9499999992549419`; hit/push sounds, cooldown `0.5` and
impulse threshold `0.05`. Matching, equipment mutation and contact math remain
with the Sulfur-Cube owners.

### Fire, plant and composed-tag joins

`infiniburn_nether` includes `infiniburn_overworld`; `infiniburn_end` includes it
plus Bedrock. The locked Overworld, Overworld Caves, Nether and End dimension types
therefore all treat Netherrack below ordinary fire as infiniburn. That bypasses the
rain, unsupported-extinguish and fuel/spread-removal branches after the scheduled
callback's player-radius gate. Netherrack itself is not flammable and does not turn
the fire into Soul Fire.

`WitherRoseBlock.mayPlaceOn` tests only `supports_wither_rose`, so a Wither Rose can
survive directly above Netherrack. The direct `base_stone_nether` membership
transitively joins `nether_carver_replaceables`, `sculk_replaceable` and
`sculk_replaceable_world_gen`. It can consequently be carved by the locked Nether
Cave, replaced by ordinary/worldgen Sculk and Sculk Veins, and selected by both
Ancient-Debris tag-match configurations. Tag reload can broaden or remove these
joins without changing state `6997`.

The large/small Ancient-Debris scattered-ore records use sizes `3/2`, air-exposure
discard chance `1/1` and the shared tag target; their placed profiles use
trapezoid Y `8..24` or uniform bottom+`8`..top-`8` in all five Nether biomes.
Netherrack is one eligible target beside Basalt and Blackstone, not an exact-only
target.

### Exact Nether feature joins

The locked Nether noise settings use Netherrack as their default solid state and
as the terminal surface-rule result. Earlier top-cap, biome, wart/Nylium,
Soul-Sand-Valley and Basalt-Delta branches can retain or replace it; the exact
density/surface evaluation remains with `WGEN-PIPELINE-001`.

The exact-block replacement records are:

| Output | Configured selector | Size/radius | Placed profile |
|---|---|---:|---|
| Basalt | replacement blob | radius `3..7` | count `75`, full height, Basalt Deltas |
| Blackstone | replacement blob | radius `3..7` | count `25`, full height, Basalt Deltas |
| Blackstone | ore | size `33` | count `2`, Y `5..31`, four non-delta biomes |
| Gravel | ore | size `33` | count `2`, Y `5..41`, four non-delta biomes |
| Magma Block | ore | size `33` | count `4`, Y `27..36`, all five biomes |
| Nether Gold Ore | ore | size `10` | count `10/20`, bottom+`10`..top-`10` |
| Nether Quartz Ore | ore | size `14` | count `16/32`, bottom+`10`..top-`10` |
| Soul Sand | ore | size `12` | count `12`, bottom..Y `31`, Soul-Sand Valley |

Every listed ore has air-exposure discard chance `0` and one exact
`block_match(netherrack)` target. Gold/Quartz use the lower counts in the four
non-delta biomes and the higher counts in Basalt Deltas. Replacement-blob and ore
traversal, draws and writes remain with the worldgen owner.

The two exact-Netherrack spring configurations emit falling Lava. Closed uses
`rock_count=5`, `hole_count=0`, no required lower block; Open uses its serializer
defaults with no required lower block. Closed runs count `16` over
bottom+`10`..top-`10` in the four non-delta biomes and count `32` in Basalt Deltas;
Open runs count `8` over bottom+`4`..top-`4` in the four non-delta biomes.
`spring_lava_nether` instead admits a five-block set containing Netherrack and runs
count `16` over bottom+`4`..top-`4` in Basalt Deltas.

`patch_fire` is present in all five Nether biomes. Its modifier chain samples
uniform count `0..5`, an in-square/full-height origin, then count `96`,
trapezoid X/Z `-7..7` and Y `-3..3`; its final predicate requires tagged air at
the candidate and exact Netherrack one below. The child simple-block feature offers
default age-zero Fire.

Three code-built features use exact Netherrack as support: Glowstone accepts it
above an empty origin beside Basalt/Blackstone; Twisting Vines accepts it below
beside Warped Nylium/Warped Wart Block; Weeping Vines accepts it above beside
Nether Wart Block. Their exact search, draw and write transactions remain with
`WGEN-PIPELINE-001`.

### Structure payload and Ruined-Portal production

The exhaustive scan finds `1,876` raw Netherrack cells in 15 of all `1,212`
templates:

| Template | Raw cells |
|---|---:|
| `bastion/bridge/starting_pieces/entrance_face` | `12` |
| `bastion/treasure/extensions/fire_room` | `4` |
| `ruined_portal/giant_portal_1` | `263` |
| `ruined_portal/giant_portal_2` | `237` |
| `ruined_portal/giant_portal_3` | `324` |
| `ruined_portal/portal_1` | `54` |
| `ruined_portal/portal_2` | `114` |
| `ruined_portal/portal_3` | `132` |
| `ruined_portal/portal_4` | `129` |
| `ruined_portal/portal_5` | `144` |
| `ruined_portal/portal_6` | `41` |
| `ruined_portal/portal_7` | `92` |
| `ruined_portal/portal_8` | `144` |
| `ruined_portal/portal_9` | `63` |
| `ruined_portal/portal_10` | `123` |

The two reachable Bastion entries use rigid `bastion_generic_degradation` or
`treasure_rooms` processors. Neither list matches Netherrack, so those 16 source
cells reach generic transform, clip, live-target and flags-`18` write admission
unchanged.

The 13 Ruined Portal templates contribute `1,860` raw cells. A noncold
position-seeded processor converts each raw Netherrack cell to Magma on strict
`.07`; cold setups retain it without that draw. Five templates also have one raw
Jigsaw. In inherited post-processing after generic template placement,
`portal_1`, `portal_2`, `portal_4` and `portal_5` parse `final_state=netherrack`
and independently offer Netherrack with flags `3`; `portal_3` offers Air. These
four final writes occur after the ordered Ruined-Portal processor chain and
therefore do not take its `.07` Netherrack-to-Magma rule.

Ruined Portals additionally create a probabilistic Netherrack/Magma apron and
unrestricted downward drip columns. Cold setups choose Netherrack without the
noncold `.07` Magma draw. Resulting Netherrack can receive persistent jungle
leaves or support generated vines. Center ownership, position-local processor RNG,
apron traversal, caller RNG and ignored write results remain with
`WGEN-STRUCTURE-RUINED-PORTAL-001`.

The NBT UTF census finds exactly 19 `minecraft:netherrack` strings: the 15 palette
entries above and those four Jigsaw finals. No structure stores a Netherrack item
stack or additional block-entity reference.

**Client projection:**

The property-free blockstate contains 16 equal default-weight variants: every
combination of X and Y rotation `0/90/180/270`, without `uvlock`, selecting the
same `block/netherrack` model. That model inherits `block/cube_all` and supplies
the same Netherrack texture on every face. The item directly selects the unrotated
block model. There is no tint.

The English name is `Netherrack`. Natural Blocks publishes it after Crying
Obsidian and before Crimson/then Warped Nylium. Building Blocks publishes it after
Dark Prismarine Slab and before the Nether-Bricks family. It appears in no other
ordinary tab. Updates use state `6997`, inventory paths use item ID `387`, maps
use `NETHER`, note blocks read `BASEDRUM`, and the five sounds use IDs
`1156..1160`. No subtype packet or connection-local state is added.

**Branches and aborts:**

Sole placement/save state; correct/wrong Pickaxe and ordinary/explosion loot;
above propagation pass/fail; every ordered tag-only/Crimson/Warped/both 27-cell
scan and write result; Bone Meal client/server/consumption/particles; Nylium decay;
smelting and two template recipes/unlocks; every direct/composed tag and
Sulfur-Cube snapshot; terrain, exact/tag replacement, spring, fire-patch and
support-feature admission; every structure processor/Jigsaw-final/apron/write
outcome; persistence and both client variant domains are distinct.

**Constants and randomness:**

State/block/item IDs `6997/285/387`; strength `0.4/0.4`; emission `0`;
dampening `15`; shade `0.2`; friction `0.6`; speed/jump `1`; sounds
`1156..1160` at `1/1`; stack `64`; Bone Meal box `3³=27` and one conditional
Boolean; smelting `200/0.1/1`; template recipe `7+1+1 -> 2`; feature values as
tabled; raw templates/files/cells `1212/15/1876`; Ruined Portal raw/final
Netherrack `1860/4`; equal client rotations `16`. Generic loot, worldgen,
structure and rendering owners retain their stated streams.

**Side effects:**

Block placement/removal and self loot; Bone Meal consumption, vibration, particles
and color conversion; Nylium decay; Furnace and crafting outputs/knowledge;
reload-selected fire/plant/Sculk/carver/debris/equipment decisions; terrain,
feature, Bastion and Ruined-Portal writes; ordinary persistence, maps, sounds,
models, names and tabs.

**Gates:**

World-write/break authority; correct Pickaxe and explosion survival; above
skylight propagation, live `nylium` tag and exact color; recipe snapshot, Furnace
progress and output capacity; tag/dimension/archetype snapshots; surface/feature/
carver/structure admission, processors, clip/live target and write result; valid
registry, map, sound and client-resource context.

**Boundary cases and quirks:**

- Bone Meal tests skylight propagation above, not air, raw brightness or build height.
- Admission accepts any `nylium` tag member, but execution recognizes only exact
  Crimson and Warped identities.
- Only the both-color branch consumes RNG; true selects Warped.
- Admitted Bone Meal consumes and projects success even when no exact color exists
  or the flags-`3` write fails.
- Netherrack is nonflammable yet is an infiniburn base in every locked dimension.
- The block is an exact target for eight ordinary ore/blob outputs, but Ancient
  Debris, carving and Sculk reach it through reloadable tag composition.
- Four Ruined-Portal Jigsaw finals write Netherrack after, and outside, the
  position-seeded `.07` template processor.
- World rendering randomizes among 16 rotations; the item model is fixed.

**Failure semantics:**

Invalid Bone Meal targets consume nothing. Admitted targets shrink once and do not
roll back for tag-only scans or failed writes. Wrong-tool player breaks drop
nothing; a failed explosion-survival condition emits nothing. Failed cooking,
crafting, tag, feature, structure or archetype admission commits only the generic
owner's stated earlier effects. Ruined-Portal post-placement final writes ignore
their Boolean result.

**Client/server authority split:**

The server owns identity, Bone Meal conversion, Nylium decay, loot, recipes,
reload-selected joins, generation, structures and persistence. Clients predict
valid Bone Meal use and project event particles, state/item IDs, maps, sounds,
rotated cube variants, name and tabs.

**Observability:**

Commands/state packets, shape/light/signal probes, Bone Meal stack and RNG traces,
particles/events, drops, Furnace/crafting inventories, fire/flower/Sculk/carver/
feature outcomes, generated terrain/templates, Sulfur-Cube attributes, maps,
note/sounds, tabs and rendering expose the listed branches.

**Persistence and reload:**

Placed blocks persist only identity; item stacks persist ordinary components.
Loot, recipes, advancements, block/item tags, dimension types, Sulfur-Cube
archetypes, worldgen records and templates are reload-selected. Registration,
`NetherrackBlock` control flow, exact feature consumers, Ruined-Portal code and
tab order remain code-built. Reload does not retroactively convert blocks or
regenerate terrain.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.Blocks`;
`NetherrackBlock#isValidBonemealTarget`, `#isBonemealSuccess`,
`#performBonemeal` and `#getType`; `BoneMealItem#growCrop`;
`NyliumBlock#randomTick`; `WitherRoseBlock#mayPlaceOn`; ordinary-fire
infiniburn dispatch; Sculk/carver/ore/scattered-ore/replacement-blob/spring
owners; `GlowstoneFeature#place`, `TwistingVinesFeature#place` and
`WeepingVinesFeature#place`; `RuinedPortalPiece`; inherited
`TemplateStructurePiece#postProcess`; `CreativeModeTabs`; both reports,
components, loot, recipe/advancement, tag/archetype, Nether worldgen, all 1,212
templates and exact client resources. Complete exact-ID and compiled-field
searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-104` across state/IDs; every above-propagation and ordered
tag-only/one/both-color Bone Meal case; write failure and Nylium decay; tool/
explosion, smelting/template recipes/unlocks; fire/rose/composed-tag/archetype
reloads; every terrain/feature/carver target; all 1,212 templates including five
Ruined-Portal Jigsaws and dynamic apron/drips; persistence, maps, sounds, 16 world
variants, fixed item model, names and both tabs. Assert exact constants,
read/draw/write order, absence claims and vanilla convergence.

**Limits:**

Generic placement, breaking, loot/explosion, Bone Meal use, Furnace/crafting/
advancement publication, Sulfur-Cube behavior, fire scheduling, plant/Sculk/
carver/feature algorithms, Bastion/Ruined-Portal construction, packet encoding
and rendering remain with their named owners. This leaf fixes the sole
Netherrack state/item, custom conversion, exact joins, generation census,
Jigsaw-final correction and projection.
