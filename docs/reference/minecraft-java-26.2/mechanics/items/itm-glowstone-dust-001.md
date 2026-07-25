# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-GLOWSTONE-DUST-001` — Glowstone Dust joins Nether light-block and Witch loot to compaction, Spectral Arrows, twinkling fireworks and ten potion mixes

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`, `PLY-002`,
`PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `PLY-BREAK-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-BREW-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-FIREWORK-STAR-001`, `ENT-001`, `ENT-KNOCKBACK-001`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked Dust/Glowstone identities, complete loot, recipe, advancement,
brewing-mix and merchant records, audited Glowstone-feature wiring, all `1,212` templates and exact
client resources determine every Glowstone-Dust-specific branch. Generic breaking, explosion,
death, loot, crafting, brewing, merchant, structure, worldgen, persistence, packet and rendering
algorithms retain the cited owners.

**Applies when:**

`minecraft:glowstone_dust` is selected from Glowstone or Witch loot, consumed by Glowstone,
Spectral-Arrow or Firework-Star crafting or a brewing stand, moved, renamed, persisted,
synchronized or rendered before and after loot, recipe, advancement, brewing or resource reload.

**Authoritative state:**

Glowstone Dust is raw item ID `1085`, a common nondamageable plain `Item` with maximum stack `64`
and no direct item tag. Its default component map has no food, consumable, remainder, fuel,
compost, equipment, durability, projectile, cooldown, trim, repair, inventory-tick or
identity-specific use branch.

Its renewable block source is `minecraft:glowstone`: block ID `294`, item ID `395`,
sole/default state `7016`. Glowstone is a property-free ordinary `Block` with Sand map color,
Pling note instrument, strength `0.3`, Glass sounds, light emission `15` and no direct block tag or
correct-tool requirement. Block placement, collision, light propagation, sulfur-cube tag use and
client projection retain their existing owners.

**Transition and ordering:**

### Glowstone block loot

`minecraft:blocks/glowstone` makes one alternatives roll under the identically named random
sequence:

1. A tool with Silk Touch level at least one selects one Glowstone block.
2. Otherwise it creates Dust, replaces count with uniform integer `B in 2..4`, then with Fortune
   level `L` adds `V=nextInt(L+1)`. It clamps the result to `1..4` and finally applies
   per-item explosion decay.

Thus nonexplosive Dust count is `min(4,B+V)`. Fortune can improve only a base count below four and
can never raise the result above four. Any hand/tool can obtain the table because Glowstone has no
correct-tool gate, and breaking grants no XP.

The outer Silk branch precedes and bypasses count, Fortune and explosion-decay functions.
Ordinary player-mined Silk therefore returns the block; a synthetic context containing both Silk
and an explosion also takes that first branch. Without Silk, explosion decay runs after the
Fortune clamp and independently filters the at-most-four items.

### Witch death

An admitted Witch death evaluates a first pool for uniform `1..3` rolls under
`minecraft:entities/witch`. Glowstone Dust is one of five weight-one entries; Stick has weight two,
for total weight `7`. Each roll independently selects Dust with probability `1/7` and can select it
repeatedly.

For each selected Dust entry, base count is uniform integer `B in 0..2`. With a living attacking
entity carrying Looting level `L>0`, a fresh uniform float `U in [0,1)` adds `round(L*U)`;
absent/nonliving attacker or level zero skips that draw. Only a positive final count emits, so a
selected zero-count row is invisible and Looting can revive it. Player-kill status and the Witch's
drinking/held-item state do not select another Dust branch. The later guaranteed Redstone pool and
generic equipment/death processing are independent.

Glowstone breaking and Witch death are the only two bundled tables that directly emit Dust. No
chest, fishing, archaeology, gift, barter, raid or merchant record directly emits or consumes the
Dust identity.

### Indirect Glowstone-block acquisition

Glowstone blocks that can be broken into Dust also enter through these audited paths:

- the `glowstone_extra` feature requires air beneath exact Netherrack, Basalt or Blackstone, offers
  the origin Glowstone, then makes `1,500` triangular-offset growth attempts that place only air
  candidates with exactly one Glowstone neighbor;
- placed `glowstone` makes fixed count `10` attempts across the full build height, while placed
  `glowstone_extra` samples a bottom-biased count `0..9` and uses the range four above bottom
  through four below top;
- both placed features occur in underground-decoration step `7`, extra before normal, in all five
  Nether biomes;
- Hoglin-Stable Bastion chest pool `1` makes `3..4` equal-weight rolls across `14` entries;
  Glowstone has probability `1/14` per roll and count `3..6`;
- level-three Cleric selects amount two from exactly two offers, guaranteeing
  `4` Emeralds -> one Glowstone, with max uses `12`, XP `10` and discount `0.05`; and
- the Wandering Trader common set selects five without replacement from `76` records, so
  `2` Emeralds -> one Glowstone has inclusion `5/76`, max uses `5`, codec-default XP `1` and
  discount `0.05`.

Trade Rebalance overrides neither offer. One exact Glowstone palette entry occurs across all
`1,212` templates: Hoglin-Stable `starting_pieces/stairs_0_mirrored` places four ordinary
Glowstone cells. No template contains the exact Dust identity. Feature, Bastion, merchant,
placement and later breaking transactions retain their owners; these joins only enumerate every
bundled block-to-Dust route.

### Three crafting joins

Glowstone Dust participates in three recipe records:

- Glowstone is a movable `2x2` square of four exact Dust, available in either crafting grid, and
  emits one default Glowstone block.
- Spectral Arrow is a full `3x3` cross of four exact Dust around one center Arrow and emits two
  default Spectral Arrows.
- The always-available special Firework-Star recipe admits at most one exact Dust as its twinkle
  modifier.

The Firework matcher still requires exactly one exact Gunpowder and at least one
component-bearing live `dyes` member, permits at most one trail, shape and twinkle input, and
rejects all other identities. One Dust sets
`FIREWORK_EXPLOSION.has_twinkle=true`; a second Dust rejects the grid. Assembly consumes it once
and preserves generic row-major colors, selected shape, empty fades and optional trail on a new
default Firework Star.

Glowstone and Spectral Arrow each have a recipe advancement whose single requirement accepts
prior recipe knowledge or direct Dust possession. Firework Star has none. Thus recipes,
listeners and direct Dust unlocks count `3/2/2`. Default results do not copy Dust patches. Pattern
normalization, result capacity, atomic consumption, component construction and knowledge
publication remain generic.

Crafting four Dust into Glowstone and breaking it without Silk is intentionally lossy or neutral:
the nonexplosive return is only `2..4` before Fortune clamp, never more than the four inputs.

### Ten potion mixes

`PotionBrewing.addVanillaMixes` registers exactly ten potion-holder edges with exact Glowstone
Dust as ingredient:

| input | output |
|---|---|
| Water | Thick |
| Leaping | Strong Leaping |
| Slowness | Strong Slowness |
| Turtle Master | Strong Turtle Master |
| Swiftness | Strong Swiftness |
| Healing | Strong Healing |
| Harming | Strong Harming |
| Poison | Strong Poison |
| Regeneration | Strong Regeneration |
| Strength | Strong Strength |

The same holder transition applies independently to ordinary, Splash and Lingering Potion
containers. One Dust starts the generic `400`-tick transaction and at commit can convert every
matching bottle among slots `0..2`, in slot order, before the ingredient shrinks by one. Unmatched
bottles remain unchanged. A potion conversion preserves container identity and installs the target
potion holder rather than copying unrelated/custom potion-content details.

Long, already-Strong and every other holder has no Glowstone-Dust edge. Invalid-only bottle sets do
not start; removing the last valid bottle cancels an active transaction under `ITM-BREW-001`.
There is no recipe knowledge, advancement, XP or RNG for brewing.

**Persistence and reload boundary:**

Dust and Glowstone stacks persist identity, count and arbitrary patches. Blocks, Witches,
containers, recipe knowledge, Firework Stars, brewing stands and merchant offers persist with
their owners. Loot, recipe, advancement, tag, trade and worldgen reload changes future evaluation,
offer construction or generation only; completed drops, crafts, brews, offers and placed blocks
are not replayed or rewritten. Existing offers and in-progress brewing retain their constructed/
remembered state. Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses Dust item ID `1085`; no Glowstone-Dust-specific packet exists. The
English name is `Glowstone Dust`. It selects one untinted `item/generated` flat with texture
`item/glowstone_dust`, without a conditional model, tint, animation, explicit display transform or
special renderer.

Ingredients orders Redstone, Glowstone Dust, Gunpowder, Dragon's Breath and Fermented Spider Eye.
Dust appears once and in no other ordinary tab.

**Branches and aborts:**

Default/patched Dust; Glowstone Silk versus base/Fortune/clamp/explosion; Witch selected/
unselected, zero/positive and attacker/Looting paths; direct versus feature/chest/template/trade
block sources; three crafting records including duplicate twinkle; ten valid and all invalid
brewing holders across three bottle containers; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Dust ID `1085`, stack `64`; Glowstone block/item/state `294/395/7016`, strength/light
`0.3/15`; block count `min(4,uniform(2..4)+nextInt(L+1))`; Witch rolls `1..3`, selection `1/7`,
count `0..2+round(LU)`; feature attempts/draws after admission `1500/7500`; Bastion chest
`3..4 x 1/14`, count `3..6`; block trades inclusion `1,5/76`; recipes/listeners/direct unlocks
`3/2/2`; potion edges `10`, brew ticks/bottles `400/3`; templates exact Dust
matches/block palette matches/live cells `0/1/4`.

**Side effects:**

Dust or Glowstone loot; natural/structure/merchant Glowstone; crafted Glowstone, Spectral Arrows
or twinkling Firework Star; recipe knowledge; up to three converted potions; crafting/brewing/
merchant consumption; durable block, stack, container, Firework, brewing and offer state;
synchronization and exact client projection.

**Gates:**

Loot context/tool/Silk/Fortune/explosion; Witch death/roll/attacker/Looting; feature origin/support/
candidate/neighbor and biome placement; structure/table roll; merchant set/current cost; exact
crafting grid/dye component/modifier uniqueness/result capacity; potion container/holder/ingredient/
fuel/timer; registry/stack decode and client resources.

**State read/written:**

Reads all gates above and writes only the loot, block, crafting, advancement, brewing, offer,
durable, wire and projection state listed above.

**Failure behavior:**

Silk selects Glowstone rather than Dust. Explosion decay or zero Witch count can erase selected
Dust. Rejected feature/table/trade selection emits no convertible block. Wrong crafting grid or
duplicate Firework modifier emits no result. Invalid potion holders do not start/convert. Reload
affects future evaluation only; decode failure follows generic stack policy.

**Boundary cases and quirks:**

Glowstone Fortune is additive then capped at four, unlike ordinary ore multiplication. Silk
bypasses explosion decay. Witch can select Dust repeatedly but a selected base zero may emit
nothing. Glowstone Dust does not appear in the bundled Piglin-bartering table in this version.
Water plus Dust creates Thick rather than a Strong potion; Long and Strong variants do not accept
another Dust. The Bastion template contains four Glowstone blocks despite only one exact palette
identity string. Cleric/Wandering trades and Bastion loot name the distinct Glowstone block, not
Dust, but ordinary non-Silk breaking joins them back to this item.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.storage.loot.functions.ApplyBonusCount$UniformBonusCount`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.level.levelgen.feature.GlowstoneFeature#place`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:glowstone`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,potion,villager_trade,trade_set,worldgen/feature}`;
`reports/minecraft/components/item/{glowstone_dust,glowstone}.json`;
`data/minecraft/loot_table/{blocks/glowstone,entities/witch,chests/bastion_hoglin_stable}.json`;
`data/minecraft/recipe/{glowstone,spectral_arrow,firework_star}.json`;
`data/minecraft/advancement/recipes/{building_blocks/glowstone,combat/spectral_arrow}.json`;
`data/minecraft/{villager_trade/{cleric/3/emerald_glowstone,wandering_trader/emerald_glowstone},tags/villager_trade/{cleric/level_3,wandering_trader/common},trade_set/{cleric/level_3,wandering_trader/common}}.json`;
`data/minecraft/worldgen/{configured_feature/glowstone_extra,placed_feature/{glowstone,glowstone_extra},biome/{basalt_deltas,crimson_forest,nether_wastes,soul_sand_valley,warped_forest}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/glowstone_dust.*`;
`assets/minecraft/lang/en_us.json`;
`ITM-BREW-001`; `ITM-FIREWORK-STAR-001`; `WGEN-PIPELINE-001`;
`WGEN-JIGSAW-BASTION-001`; `EXP-ITM-088`.

**Test vectors:**

Run `EXP-ITM-088` across default/patched Dust, every Glowstone
tool/Silk/Fortune/explosion branch, Witch pool/count/Looting paths, both placed features in all
five biomes, Bastion chest/template and both block trades, all three crafts/two listeners, all ten
brewing edges and invalid holders, every template, reload domains, persisted/synchronized owners
and exact ID/name/model/tab projection.

**Limits:**

Generic block/light, breaking, explosion decay, entity death, loot, structure, crafting,
Firework-Star, brewing, merchant, worldgen, stack codec, packet and renderer control flow remains
with cited owners. Glowstone block sinks, Witches, Spectral Arrows, Firework Stars, potions and
Glowstone-consuming blocks retain their own owners. This leaf fixes exact Glowstone Dust identity,
source/sink joins, absences and projection.
