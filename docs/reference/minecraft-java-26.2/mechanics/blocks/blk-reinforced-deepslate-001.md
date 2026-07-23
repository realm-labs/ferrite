# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-REINFORCED-DEEPSLATE-001` — Reinforced deepslate is slowly breakable but identity-immovable

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `PLY-005`, `PLY-006`, `RED-004`,
`RED-006`, `ENT-001`, `MOB-001`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration and report fix the property-free state, hardness,
resistance, sound, map color and inherited full-cube behavior. A direct piston identity check,
three exact block-tag memberships, the empty loot table, creative-tab registration and client
assets fix the remaining exceptional behavior and presentation. Ancient-city placement and
tag-consuming feature algorithms remain with their existing world-generation owners.

**Applies when:**

`minecraft:reinforced_deepslate` is placed, mined, removed, pushed, exploded, scanned by a dragon
or wither, used as a world-generation replacement target, loaded from an ancient-city structure,
serialized into terrain or projected to a client.

**Authoritative state:**

Reinforced deepslate is a plain `Block` with one property-free default state, ID `32085`, and no
block entity. Its registration selects deepslate map color, bass-drum note instrument, deepslate
sound, destroy speed `55.0` and explosion resistance `1200.0`. It does not request the
correct-tool-for-drops property. Inherited shape and render properties make it an opaque full
collision, selection and occlusion cube with `PushReaction.NORMAL`.

The bundled block loot table has a block-table type and random-sequence identifier but no pools,
so every ordinary block-loot evaluation returns no stacks. The block still has an ordinary
common-rarity `BlockItem`: it stacks to `64`, has no special data components or use gate, and is
listed unconditionally in the building-blocks creative tab between deepslate-tile walls and tuff.
Once held, generic block-item placement writes state 32085.

**Transition and ordering:**

Inherited destroy progress divides the player's current destroy speed by `55.0` and by `30`:
because the registration does not require a correct tool, `hasCorrectToolForDrops` is true for this
state regardless of the held stack. An otherwise unmodified grounded player with destroy speed
`1.0` has per-tick progress `1/1650`; the generic elapsed-time calculation reaches `1.0` on its
1650th progress sample, while authoritative completion still follows the generic STOP/delayed
flow. Effects, attributes, game mode, client prediction, abort/restart handling and removal remain
with the generic break owners. Completion yields no block loot regardless of tool or Silk Touch
because the table has no pools; creative and explicit administrative removal remain immediate
paths.

`PistonBaseBlock.isPushable` rejects exact reinforced deepslate together with obsidian, crying
obsidian and respawn anchors before the destroy-speed and `PushReaction` checks. Neither push nor
sticky pull may move or destroy it even though its destroy speed is positive and its inherited
reaction is `NORMAL`. The block adds no scheduled tick, random tick, use, attack, entity-contact,
neighbor, redstone, comparator or block-event callback of its own.

Ordinary explosion calculation observes the finite resistance `1200.0`; this leaf does not claim
universal immunity against arbitrary explosion power or a custom calculator. Wind-charge
calculators are a deliberate exception: their present `blocks_wind_charge_explosions` holder set
contains only barrier and bedrock. Reinforced deepslate is outside it, so
`SimpleExplosionDamageCalculator.getBlockExplosionResistance` returns empty instead of consulting
the registered `1200.0`, while `shouldBlockExplode` remains true. Ray traversal, affected-block
collection, writes, drops and effects remain with `RED-EXPLOSION-001`.

**Entity and world-generation consumers:**

- `EnderDragon.checkWalls` retains a scanned reinforced-deepslate cell because it belongs to
  `dragon_immune`; the cell contributes to the dragon's blocking/in-wall result whether protection
  comes from that tag or from disabled mob griefing.
- `WitherBoss.canDestroy` requires a nonair state outside `wither_immune`. Membership therefore
  makes the predicate false before the wither's later destruction call.
- `features_cannot_replace` membership makes every consumer of that exact holder set reject a live
  reinforced-deepslate replacement target. This includes protected structure processors and the
  locked feature predicates already specified by `WGEN-PIPELINE-001`; traversal, RNG, candidate
  selection, write flags and return values remain with those owners.
- Locked ancient-city structure data can select reinforced deepslate both as template terrain and
  as one of the three jigsaw connector final-state payloads inventoried by
  `WGEN-JIGSAW-ANCIENT-CITY-001`. Structure transform, processor order, protection and commit remain
  owned there; this leaf owns the selected block's runtime identity after a successful write.

All three tag memberships are reloadable snapshot inputs. The registration properties, direct
piston identity check and ordinary item registration are code-locked. Reinforced deepslate is not
in the wind-charge resistance tag, and it is not a special bedrock/obsidian base for end crystals,
a bedrock exception for beacon beams, or a bedrock exclusion for Enderman support.

**Client projection:**

The sole blockstate variant selects one `cube_bottom_top` model. It maps distinct reinforced-
deepslate top, side and bottom textures to the full cube. The item definition selects that same
block model. Ordinary terrain and block updates project state ID 32085; there is no block-entity,
conditional model, random variant or special-renderer payload. Filled maps observe the registered
deepslate map color through the generic map owner.

**Branches and aborts:**

Baseline versus modified player speed; continuous progress versus abort/restart versus
creative/explicit removal; piston identity rejection before generic mobility; ordinary registered
resistance versus wind-charge nonmember resistance; dragon/wither tag admission; protected target
versus accepted feature write; structure placement versus processor rejection; block versus item
projection are distinct observable branches.

**Constants and randomness:**

State ID `32085`; destroy speed `55.0`; ordinary explosion resistance `1200.0`; common-rarity
stack `64`; full unit cube; bass-drum instrument; deepslate sound and map color; base progress
`1/1650` per tick at destroy speed `1.0`; one world model and the same item model. The block and its
loot table consume no RNG. Delegated break modifiers, explosions, structure placement and map
owners retain their inputs and RNG.

**Side effects:**

Generic placement or removal; long-running break progress and no block loot; rejected piston plan;
ordinary or wind-charge explosion result; dragon/wither protection; feature/structure replacement
admission; generic terrain, item and map projection.

**Gates:**

Placement/removal authority; player destroy speed and correct-tool result; uninterrupted break
session; piston identity; explosion calculator and holder-set snapshot; mob griefing and boss tags;
feature-protection holder set and live target; structure processor chain; client model context.

**Boundary cases and quirks:**

Hardness `55` makes the block slow, not continuously unbreakable. Its no-drop table is independent
of break admission and tool choice. `PushReaction.NORMAL` does not make it piston-movable because
the exact identity gate runs first. Its large ordinary blast resistance is ignored rather than
lowered by the locked wind-charge calculators because the optional immune-block set is present and
the block is not a member. Tag membership is reloadable; positive and negative identity checks are
not inferred from similar deepslate blocks.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`,
`net.minecraft.world.level.block.state.BlockBehaviour#getDestroyProgress`,
`net.minecraft.server.level.ServerPlayerGameMode#incrementDestroyProgress`,
`net.minecraft.world.level.block.piston.PistonBaseBlock#isPushable`,
`net.minecraft.world.item.Items`,
`net.minecraft.world.item.CreativeModeTabs#bootstrap`,
`net.minecraft.data.loot.packs.VanillaBlockLoot#generate`,
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#checkWalls`,
`net.minecraft.world.entity.boss.wither.WitherBoss#canDestroy`,
`net.minecraft.world.level.SimpleExplosionDamageCalculator#getBlockExplosionResistance`,
`net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.AbstractWindCharge`,
`net.minecraft.world.entity.projectile.hurtingprojectile.windcharge.WindCharge`,
`net.minecraft.world.level.levelgen.feature.Feature#isReplaceable`,
`net.minecraft.world.level.levelgen.feature.Feature#safeSetBlock`,
`net.minecraft.world.level.levelgen.feature.MonsterRoomFeature#place`,
`net.minecraft.world.level.levelgen.structure.templatesystem.ProtectedBlockProcessor#processBlock`;
`reports/blocks.json#minecraft:reinforced_deepslate`,
`reports/minecraft/components/item/reinforced_deepslate.json`,
`data/minecraft/loot_table/blocks/reinforced_deepslate.json`,
`data/minecraft/tags/block/{dragon_immune,features_cannot_replace,wither_immune}.json`,
`data/minecraft/structure/ancient_city/**/*.nbt`,
`assets/minecraft/blockstates/reinforced_deepslate.json`,
`assets/minecraft/models/block/reinforced_deepslate.json`,
`assets/minecraft/items/reinforced_deepslate.json`.

**Test vectors:**

Run `EXP-BLK-032` across generic/creative placement, unmodified and modified continuous break,
abort/restart, every loot-relevant tool, explicit removal, piston push/pull, ordinary and both
wind-charge explosion calculators, dragon/wither tag and mob-griefing combinations, protected
feature/processor targets, ancient-city placement, reload, map color and block/item rendering.
Assert progress samples, STOP/delayed completion, state/write/drop order, tag snapshot, explosion
resistance optional, state 32085 convergence and the single shared model.

**Limits:**

This leaf does not re-specify generic placement/break packets, player destroy-speed modifiers,
piston planning, explosion rays, boss movement, structure traversal, tag reload publication, map
pixel shading or client model loading. Those remain with `BLK-002`, `PLY-006`,
`RED-PISTON-001`, `RED-EXPLOSION-001`, entity/mob owners, `WGEN-PIPELINE-001`,
`WGEN-JIGSAW-ANCIENT-CITY-001`, DataReload and `CLI-006`.
