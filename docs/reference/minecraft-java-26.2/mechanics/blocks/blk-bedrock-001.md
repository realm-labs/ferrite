# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BEDROCK-001` — Bedrock is a zero-progress cube with protected runtime roles

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `RED-004`,
`RED-006`, `ENV-003`, `ENV-005`, `ENT-001`, `MOB-001`, `WGEN-003`, `WGEN-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration fixes the one block state, hardness, resistance, loot,
spawn, sound and map-color properties. Direct identity checks and six exact block-tag memberships
fix its piston, boss, projectile, fire, beacon, map, lighting, portal and generation roles. The
ordinary block item and client assets fix placement and presentation. Algorithms that merely choose
bedrock as a generated state remain with their existing world-generation owners.

**Applies when:**

`minecraft:bedrock` is placed, mined, removed, pushed, exploded, used as a support or scanned by an
entity/environment consumer, serialized into terrain, rendered, sampled for a map, or selected by
locked world-generation and upgrade data.

**Authoritative state:**

Bedrock is a plain `Block` with one property-free default state, ID `85`, and no block entity. Its
registration selects stone map color, bass-drum note instrument, destroy speed `-1.0`, explosion
resistance `3,600,000.0`, no loot table and an entity-spawn predicate that always returns false.
All other inherited shape/render properties make it an ordinary opaque full collision, selection
and occlusion cube. Its inherited piston reaction is `NORMAL`; the destroy-speed gate, not a
special reaction value, makes it immovable.

Its ordinary common-rarity `BlockItem` stacks to `64`, names and models itself as bedrock and has no
special component or use gate. Once acquired, generic block-item placement can place state 85. The
creative inventory or command path used to acquire it is not a property of the item itself.

**Transition and ordering:**

Inherited destroy progress reads destroy speed first and returns exactly `0.0` when it is `-1.0`.
A survival-style continuous break session therefore never accumulates completion, irrespective of
tool. Creative instant destruction and explicit world/administrative writes do not use that
progress as an absolute removal prohibition; when they remove bedrock, the absent loot table emits
no block loot.

`PistonBaseBlock.isPushable` rejects any state whose destroy speed is `-1.0` before consulting
`PushReaction`, so neither push nor pull may move or destroy bedrock. The block adds no scheduled
tick, random tick, use, attack, entity-contact, neighbor, redstone, comparator or block-event
callback of its own.

Ordinary explosion calculation sees the finite registered resistance `3,600,000.0`; this leaf does
not turn that large value into an invented universal immunity for arbitrarily large custom
explosions. Wind-charge calculators separately resolve `blocks_wind_charge_explosions` as their
immune-block holder set. For a member they return resistance `3,600,000.0`; for a nonmember they
return no block resistance, while their `explodesBlocks` flag remains true. Both the ordinary wind
charge and abstract wind-charge calculator use this set. `RED-EXPLOSION-001` retains ray sampling,
strength, decay, interaction and commit order.

**Entity and environment consumers:**

- `EnderDragon.checkWalls` ignores air and `dragon_transparent`, then either marks a scanned solid
  cell as blocking when mob griefing is disabled or it is `dragon_immune`, or removes the cell.
  Bedrock is dragon-immune, so it is retained and contributes to the dragon's in-wall result.
- `WitherBoss.canDestroy` requires a nonair state outside `wither_immune`. Bedrock membership makes
  that predicate false before the wither's later destruction call.
- An Enderman carried-block placement candidate requires air at the target, a nonair full-collision
  support that is not bedrock, survival of the carried state and an empty entity box. Bedrock support
  therefore rejects the placement even though it is a full cube.
- End-crystal item use accepts exact bedrock or obsidian as its clicked base, then retains the shared
  empty-above, two-block entity-clearance and server creation branches.
- Beacon beam scanning treats light dampening `15` as blocking except for exact bedrock. A current
  beam section therefore continues through bedrock; `BLK-BEACON-001` retains scan cadence, section
  color and effect publication.
- In the End dimension, `infiniburn_end` expands the Overworld set and adds bedrock. Fire directly
  above it takes the infiniburn-base branch: rain does not extinguish it and age/no-neighbor-fuel
  does not remove it. Scheduling, aging, neighboring burn/spread and RNG remain `ENV-FIRE-001`.
- The registered spawn predicate rejects bedrock as the supporting state for every entity type.

**World, generation and sentinel consumers:**

The light engine returns bedrock's default state when its requested lighting chunk is absent, giving
the generic opacity/occlusion path a closed boundary sentinel; this is not a write of bedrock into
the world. Non-ceiling filled-map sampling likewise substitutes bedrock when the sampled surface
height is at or below the level minimum, then contributes its stone map color and sampled height to
the ordinary map calculation.

End-gateway safe-position search can exclude bedrock while choosing the highest full-collision
block, and End-crystal placement accepts it as a base. The End fight recognizes bedrock in its exit-
portal pattern and its portal-location scan. These portal/fight algorithms remain owned by
`WGEN-PORTAL-001` and `WGEN-PIPELINE-001`; this leaf owns the exact identity branch.

The locked generation program selects bedrock in the Overworld/Nether surface-rule gradients,
default and bundled flat layers, End gateway matrix, End podium and End-spike crystal cap. Below-
zero retrogen replaces old bedrock with deepslate before applying its missing-bedrock mask. Delta
and basalt-column features hard-code bedrock among their nonreplaceable/non-support states, while
count-on-every-layer refuses a bedrock-supported empty transition as a selectable layer.

After the `test create` command's dimension check and empty-test construction, the command reads the
new structure origin and visits the inclusive rectangle from that origin through
`origin.offset(sizeX - 1, 0, sizeZ - 1)`. It offers bedrock with `setBlockAndUpdate` at every floor
cell, ignores each write result, and only then sends success. Height does not change this one-layer
floor. Command registration, permission, argument admission, empty-test layout and feedback remain
with the command/test-instance owners.

Data-selected protection has two further branches. `features_cannot_replace` makes the shared safe
feature-write predicate reject a live bedrock target. `geode_invalid_blocks` makes a sampled
bedrock cell consume the geode invalid budget; placement aborts as soon as that count exceeds the
configured threshold. All exact traversal, RNG, write flags, feature return values, surface-rule
ordering and structure geometry stay with `WGEN-PIPELINE-001`.

**Client projection:**

The property-free blockstate has four default-weight variants: base cube, mirrored cube, base cube
rotated 180 degrees around Y, and the mirrored cube with the same rotation. Both cubes use the
`block/bedrock` texture. Generic position-dependent block-model selection owns which equal-weight
variant is chosen. The item definition always selects the unmirrored, unrotated base block model.
Ordinary terrain/block updates project state ID 85; the model has no block-entity or special-renderer
payload. Filled maps observe the stone map color through the server-owned map update described
above.

**Branches and aborts:**

Survival progress versus creative/explicit removal; piston admission before reaction; ordinary
versus wind-charge explosion calculator; mob-griefing and boss immunity; Enderman support; crystal
base plus clearance; beacon obstruction exception; End infiniburn; missing-light-chunk and map-min-Y
sentinels; direct, test-command and data-selected generation/protection; world versus item model are
distinct observable branches.

**Constants and randomness:**

State ID `85`; destroy speed `-1.0`; explosion resistance `3,600,000.0`; common-rarity stack `64`;
full unit cube; bass-drum instrument; stone map color; four world variants of default weight `1` and
one fixed item model. The block itself consumes no RNG. Delegated worldgen, map and model-selection
owners retain their RNG and seed rules.

**Side effects:**

Generic placement or authorized removal; no block loot; rejected continuous break and piston plan;
boss collision/protection results; end-crystal entity placement admission; beacon continuation;
fire survival branch; generation/retrogen writes and feature rejection; generic terrain, model and
map projection.

**Gates:**

Generic placement/removal authority; destroy speed; piston pushability; explosion calculator and
tag snapshot; mob griefing and boss tags; carried-block target/support/entity clearance; crystal
base/air/entity clearance; beacon current section and dampening; dimension infiniburn holder set;
lighting-chunk availability; map surface height; worldgen feature/configuration and live-state
predicates; client model context.

**Boundary cases and quirks:**

Destroy speed `-1` prevents progress and pistons but is not a global ban on all explicit state
writes. `PushReaction.NORMAL` is never reached by the piston resolver. The wind-charge tag's name
does not mean its members are easy to explode; the calculator treats it as the high-resistance set.
Opaque bedrock is the beacon's explicit dampening-15 exception. The lighting and map paths use a
bedrock state as a sentinel without proving that a world cell contains bedrock. Tag membership is a
reloadable input; the registered hardness, resistance, shape and direct identity checks are code-
locked.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.state.BlockBehaviour#getDestroyProgress`,
`net.minecraft.world.level.block.piston.PistonBaseBlock#isPushable`,
`net.minecraft.world.item.Items`,
`net.minecraft.world.item.EndCrystalItem#useOn`,
`net.minecraft.world.entity.monster.EnderMan$EndermanLeaveBlockGoal#canPlaceBlock`,
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#checkWalls`,
`net.minecraft.world.entity.boss.wither.WitherBoss#canDestroy`,
`net.minecraft.world.level.SimpleExplosionDamageCalculator#getBlockExplosionResistance`,
`net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.AbstractWindCharge`,
`net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.WindCharge`,
`net.minecraft.world.level.block.FireBlock#tick`,
`net.minecraft.world.level.block.entity.BeaconBlockEntity#tick`,
`net.minecraft.world.level.lighting.LightEngine#getState`,
`net.minecraft.world.item.MapItem#update`,
`net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity#findTallestBlock`,
`net.minecraft.world.level.levelgen.placement.CountOnEveryLayerPlacement#findOnGroundYPosition`,
`net.minecraft.world.level.levelgen.BelowZeroRetrogen#replaceOldBedrock`,
`net.minecraft.gametest.framework.TestCommand#createNewStructure`,
`net.minecraft.world.level.levelgen.feature.GeodeFeature#place`,
`net.minecraft.world.level.levelgen.feature.EndGatewayFeature#place`,
`net.minecraft.world.level.levelgen.feature.EndPodiumFeature#place`,
`net.minecraft.world.level.levelgen.feature.EndSpikeFeature#placeSpike`;
`reports/blocks.json#minecraft:bedrock`, `reports/minecraft/components/item/bedrock.json`,
`data/minecraft/tags/block/{blocks_wind_charge_explosions,dragon_immune,features_cannot_replace,geode_invalid_blocks,infiniburn_end,wither_immune}.json`,
`data/minecraft/worldgen/{noise_settings,flat_level_generator_preset,world_preset}/**/*.json`,
`assets/minecraft/blockstates/bedrock.json`,
`assets/minecraft/models/block/{bedrock,bedrock_mirrored}.json`,
`assets/minecraft/items/bedrock.json`.

**Test vectors:**

Run `EXP-BLK-031` across generic placement, every break/removal authority, piston and explosion
outcome; every boss/tag/game-rule, Enderman, crystal, beacon, fire and spawn branch; missing/present
light chunks, map min-Y fallback, gateway search, test-create floor and all locked
generation/protection consumers.
Assert state/properties, read/write/result order and the four world variants versus fixed item model.

**Limits:**

This leaf does not re-specify generic placement/break packets, explosion rays, fire spread, entity
AI, beacon cadence, map pixel shading, portal travel, client model seeding or worldgen traversal.
Those remain with `BLK-002`, `PLY-006`, `RED-PISTON-001`, `RED-EXPLOSION-001`, `ENV-FIRE-001`,
`BLK-BEACON-001`, `BLK-TEST-INSTANCE-001`, `WGEN-PORTAL-001`, `WGEN-PIPELINE-001` and
`CLI-006`.
