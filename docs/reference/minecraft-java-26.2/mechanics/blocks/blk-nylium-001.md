# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-NYLIUM-001` — Nylium decay and color-specific Nether growth

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`, `PLY-006`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`, `PLY-AUTOJUMP-001`,
`ITM-003`, `ITM-004`, `ITM-006`, `ITM-LOOT-001`, `ITM-ENCHANT-001`,
`ENT-KNOCKBACK-001`, `MOB-004`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`ENV-LIGHT-001`, `WGEN-003`, `WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — both locked registrations and reports, complete `NyliumBlock`
control flow, its `NetherrackBlock` acquisition caller, exact loot/tag/archetype/worldgen
data, hard-coded Hoglin and feature consumers, exhaustive exact-ID and compiled-field
searches, all 1,212 decoded templates and exact client resources close the family.

**Applies when:**

`minecraft:crimson_nylium` or `minecraft:warped_nylium` is placed, random-ticked,
bonemealed, mined, exploded, selected by Netherrack conversion, chest loot, Hoglin
pathing, Enderman carriage, sulfur-cube equipment or Nether generation, persisted,
synchronized or rendered.

**Authoritative state:**

| Identity | Map color | State | Block ID | Item ID |
|---|---|---:|---:|---:|
| Crimson Nylium | `CRIMSON_NYLIUM` | `20974` | `875` | `60` |
| Warped Nylium | `WARPED_NYLIUM` | `20957` | `866` | `61` |

Both are property-free `NyliumBlock` full cubes without block entities. Their registrations
set `BASEDRUM`, hardness/resistance `0.4/0.4`, correct-tool-required, Nylium sounds and
random ticks. Emission is zero. Collision/occlusion and every face shape are the full
`0..16` cube; friction is `0.6`, speed/jump factors are `1`, and piston reaction is
the default `NORMAL`. They are sturdy, suffocating, view-blocking and valid spawn-floor
geometry, but produce no signal/comparator output and hold no fluid.

Nylium sound volume/pitch is `1/1`; break/step/place/hit/fall sound IDs are
`1126/1127/1128/1129/1130`. Each item is a common nondamageable 64-stack with standard
block-item components.

Both blocks directly belong to `enderman_holdable`, `huge_brown_mushroom_can_place_on`,
`huge_red_mushroom_can_place_on`, `mineable/pickaxe`, `nylium` and
`overrides_mushroom_light_requirement`. Neither requires a minimum tool tier. Both block
items directly belong to `sulfur_cube_archetype/slow_bouncy`. They have no FireBlock
odds, fuel time, recipe, Composter entry or other direct block/item tag.

**Transition and ordering:**

### Random-tick decay

Every selected server random tick reads the state directly above. It then computes

```text
LightEngine.getLightDampeningInto(nylium, above, UP, above.getLightDampening())
```

using the Nylium's upward face, the above state and that state's own dampening. An ingress
result strictly below `15` preserves the Nylium. Result `15` converts the selected state
to default Netherrack through `setBlockAndUpdate`; the Boolean write result is ignored.
The method consumes no RNG. Random-tick selection remains with `SIM-RANDOM-001`.

This is a shape/dampening calculation, not a raw block-light or sky-light threshold.
Opaque full cover normally reaches the decay endpoint; partial and transparent cases
depend on the shared light-engine face calculation. Removing the cover does not spread
Nylium back by random tick.

Ordinary placement writes the sole state. Rotation and mirror are identity operations.
Pistons use normal movement, fluids are excluded by the full block, and neither state
has a neighbor-update override.

### Bonemeal vegetation dispatch

A Nylium is a valid bonemeal target exactly when the state above is canonical air and
that above position is inside build height. Success is always true and consumes no
success RNG. The generic server caller consumes one Bone Meal and invokes
`performBonemeal`; client validation predicts the admitted interaction. The block's
bonemealable type is `NEIGHBOR_SPREADER`, so growth particles use the Nylium position.

Execution rereads the live state at the original position after admission, obtains the
live configured-feature registry and dispatches by exact block identity:

- Crimson calls only `crimson_forest_vegetation_bonemeal` at `origin.above()`.
- Warped calls `warped_forest_vegetation_bonemeal`, then `nether_sprouts_bonemeal`,
  always consumes `nextInt(8)`, and calls `twisting_vines_bonemeal` only on zero.
- A live state of any other identity calls no feature and consumes no Nylium-owned RNG.

Each helper first rechecks that the above position is inside build height, performs an
optional lookup of its key, and invokes a present configured feature with the same
level, generator and RNG. Missing keys are independently skipped; every feature result
is ignored. The Warped `nextInt(8)` occurs after both first lookups/calls regardless of
their presence or results.

The locked Crimson/Warped vegetation configurations use spread width/height `3/1` and
weights Crimson Roots/Crimson/Warped Fungus `87/11/1` or Warped Roots/Crimson Roots/
Warped/Crimson Fungus `85/1/13/1`. Sprouts uses the same `3/1` dimensions and a fixed
Nether Sprouts provider. Twisting uses width/height/max-height `3/1/2`. Their feature
algorithms and writes remain with `WGEN-PIPELINE-001`.

### Netherrack-to-Nylium conversion

Bonemealable Netherrack is the renewable acquisition caller. Its target check first
requires the state above the Netherrack to propagate skylight downward. It then scans
the inclusive `origin+(-1,-1,-1)..(+1,+1,+1)` box and admits on the first live member
of the reloadable `nylium` block tag.

Server execution scans the same 27 positions, tracking exact Warped and Crimson block
identities and stopping once both have been seen:

- Warped only writes default Warped Nylium.
- Crimson only writes default Crimson Nylium.
- Both consume one `nextBoolean()`; true selects Warped and false Crimson.
- Neither exact identity performs no write.

The write uses flags `3` and ignores its result. Thus a data pack can add a third block
to `nylium`: that block can make the Netherrack a valid target and consume Bone Meal,
but cannot select a conversion color unless an exact vanilla Nylium is also found.
The above-state target and the nearby-color scan are separate gates.

### Breaking, loot and acquisition

Both one-roll block tables first test Silk Touch level at least one. A correct Silk
Touch pickaxe returns one matching Nylium without an explosion-survival condition.
Otherwise the table offers one Netherrack behind `survives_explosion`. The generic
correct-tool gate rejects hand and incorrect-tool player loot before either branch.
Each table uses random sequence `minecraft:blocks/<nylium>`.

Crimson alone appears in the Bastion Hoglin-Stable chest table. Its second pool takes
uniform three or four rolls with replacement over 14 equal-weight entries; selecting
Crimson Nylium applies inclusive uniform count `2..7`. No other direct nonblock loot,
recipe, trade or container record names either Nylium item.

Both blocks are Enderman-holdable. Generic take/leave goals own the no-drop removal,
carried state, placement gates and persistence. Because each state is a full cube with
no survival predicate, a valid leave target retains the matching Nylium rather than
shape-transforming it to AIR.

Crimson alone changes Hoglin ground preference. After the repellent-nearby check, a
candidate whose block below is exact Crimson Nylium returns walk-target value `10`;
Warped and every other block return `0`. A nearby remembered repellent takes priority
and returns `-1`.

Both block items select the reloadable `slow_bouncy` sulfur-cube archetype. That record
fixes horizontal/vertical knockback powers `0.4125/0.24`; knockback and explosion-
knockback resistance additions `0.4/0.4`; bounciness `0.6`; total friction/air-drag
multipliers `-0.7/-0.95`; hit/push sounds, cooldown `0.5` and impulse threshold `0.05`.
Matching order, equipment mutation and contact/knockback math remain with the entity owner.

### World-generation and tag joins

The exact Nether surface-rule tree creates both states. On an on-floor cell at or above
absolute Y `31`, Crimson/Warped Forest branches whose netherrack noise is below `0.54`
select their wart block when wart noise is at least `1.17`; otherwise they select the
matching Nylium. The shared earlier bedrock/cap and biome branches, surface-depth tests
and final Netherrack fallback remain with `WGEN-PIPELINE-001`.

Both exact states are valid bases for their natural and planted huge-fungus
configurations. Their shared `nylium` tag admits the Nether Forest Vegetation feature,
and their `overrides_mushroom_light_requirement` membership lets small mushrooms
survive without light/solid-render checks. Both huge-mushroom floor tags admit Brown
and Red huge-mushroom validation.

Warped alone is one of the three exact support blocks accepted by the Twisting Vines
feature, beside Netherrack and Warped Wart Block. The family adds no raw block cells:
the exhaustive census finds zero Crimson and zero Warped Nylium cells across all 1,212
bundled structure templates. Surface rules and features are the generation sources;
Crimson chest acquisition is loot-selected.

**Client projection:**

Each property-free blockstate selects a `cube_bottom_top` model with Netherrack on the
bottom, its matching Nylium side texture on four sides and matching top texture.
The item selector reuses that block model. English names are exactly
`Crimson Nylium` and `Warped Nylium`.

Natural Blocks publishes Crimson then Warped immediately after Netherrack and before
Soul Sand; neither appears in another ordinary tab. Block updates use states
`20974/20957`, inventory paths use item IDs `60/61`, maps use
`CRIMSON_NYLIUM/WARPED_NYLIUM`, note blocks read `BASEDRUM`, and both use the five
Nylium sounds. This family adds no packet field or connection-local state.

**Branches and aborts:**

Both identities; every above-state shape/dampening endpoint; random-tick write result;
air/nonair and build-height bonemeal target; client/server; live state Crimson/Warped/
other; every optional feature key/result and Warped `nextInt(8)` endpoint; Netherrack
above propagation, tag-only/exact one/both nearby colors and write result; correct/
incorrect/Silk/explosion loot; chest selection; Hoglin repellent/preference; Enderman
and sulfur-cube joins; surface/huge/vegetation/vine selectors; 1,212-template census;
persistence and client projection are distinct.

**Constants and randomness:**

States `20974/20957`; block IDs `875/866`; item IDs `60/61`; strength `0.4/0.4`;
emission `0`; sounds `1126..1130`; stack `64`; decay threshold `<15`; Warped
bonemeal `nextInt(8)==0`; Netherrack box `3x3x3=27` and one conditional Boolean;
vegetation `3/1`, weights `87:11:1` and `85:1:13:1`; Sprouts `3/1`; Twisting
`3/1/2`; chest rolls/entries/count `3..4/14/2..7`; slow-bouncy values above;
surface thresholds Y `31`, netherrack `<0.54`, wart `>=1.17`; raw templates/cells
`1212/0/0`.

**Side effects:**

Random-tick Netherrack conversion; Bone Meal consumption/particles and feature writes;
Netherrack color conversion; block loot and chest inventory; Hoglin navigation score;
Enderman carried state/removal/placement; sulfur-cube equipment/attributes/contact;
surface/feature generation; ordinary persistence; maps, sounds, cube models, names
and tab projection.

**Gates:**

Random-tick selection and upward light-engine inputs; world-write authority; bonemeal
air/build-height/server/live identity; configured-feature registry; Netherrack
skylight/tag/exact-color scan; correct tool, Silk and explosion context; chest RNG;
Hoglin repellent memory/ground; Enderman gamerule/placement; sulfur-cube archetype;
surface biome/noises/depth; feature base/tag/support; registry and client resources.

**Boundary cases and quirks:**

- Decay consumes no RNG and tests computed ingress dampening, not raw brightness.
- Nylium bonemeal success is unconditional after the air/height target gate.
- Execution rereads the origin; a changed live block can switch color dispatch or do nothing.
- Warped always consumes `nextInt(8)` after vegetation and sprouts, even when either key
  is missing or its feature returns false.
- Netherrack target admission accepts any `nylium` tag member, but conversion recognizes
  only the two exact vanilla identities.
- Correct Silk loot is not explosion-conditioned; ordinary Netherrack loot is.
- Crimson alone has Bastion chest and Hoglin preference joins; Warped alone supports
  Twisting Vines and has the three-stage bonemeal path.

**Failure semantics:**

Failed decay writes are not retried inside the tick. Invalid Nylium bonemeal targets do
not consume; admitted targets consume and ignore every feature absence/result. Netherrack
tag-only admission can consume without a conversion, and writes have no rollback.
Incorrect-tool player breaks drop nothing. Failed loot/chest/Hoglin/Enderman/archetype/
worldgen admission commits only the generic owner's stated effects. Client failure affects
projection, not authoritative identity.

**Client/server authority split:**

The server owns identity, random ticks, light-dampening decay, bonemeal, loot, mob state,
world generation and persistence. Clients validate target prediction and project states,
particles, maps, sounds, models, names and tabs.

**Observability:**

Commands/state packets, light/shape probes, random-tick traces, Bone Meal counts and
feature traces, drops, chest inventories, Hoglin navigation, carried blocks, sulfur-cube
attributes/sounds, generated terrain, maps, note/sounds, tabs and rendering expose the
listed branches.

**Persistence and reload:**

Placed states persist only identity. Item stacks persist ordinary components. Loot,
block/item tags, sulfur-cube archetype, configured features, noise settings and templates
are reload-selected. Registrations, `NyliumBlock`/`NetherrackBlock` control flow, Hoglin
preference and tab order remain code-built. Reload does not retroactively decay existing
Nylium or regenerate terrain.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`;
`OFF-REPORT-001`. Anchors: `net.minecraft.world.level.block.Blocks`;
`NyliumBlock#canBeNylium`, `#randomTick`, `#isValidBonemealTarget`,
`#isBonemealSuccess`, `#performBonemeal`, `#place` and `#getType`;
`NetherrackBlock#isValidBonemealTarget` and `#performBonemeal`;
`BoneMealItem#growCrop`; `Hoglin#getWalkTargetValue`;
`NetherForestVegetationFeature#place`; `HugeFungusFeature#place`;
`TwistingVinesFeature#place`; `CreativeModeTabs`; both reports/components/loot/resource
sets, six block tags, slow-bouncy tag/record, Hoglin-Stable table, four bonemeal
configurations, four huge configurations, Nether noise settings and all 1,212 templates.
Complete exact-ID and compiled-field-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-102` across both states/IDs; every dampening decay boundary; bonemeal
target/live-state/feature outcome; Netherrack proximity/color conversion; tool/Silk/
explosion/chest/Hoglin/Enderman/sulfur-cube joins; surface/huge/vegetation/vine
selectors and all 1,212 templates; persistence, maps, sounds, particles, tabs and
models. Assert exact constants, read/draw/write order, absence claims and convergence.

**Limits:**

Generic random-tick selection, light-engine geometry, placement/break/loot/explosion,
Bone Meal, configured-feature algorithms, Bastion/container loot, Hoglin/Enderman AI,
sulfur-cube equipment/contact, surface evaluation, packet encoding and rendering remain
with their named owners. This leaf fixes both Nylium identities, custom decay/bonemeal
dispatch, acquisition, exact joins, asymmetries and projection.
