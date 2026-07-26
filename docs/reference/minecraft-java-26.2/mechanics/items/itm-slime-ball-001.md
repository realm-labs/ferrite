# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SLIME-BALL-001` — Slime Ball joins small-Slime and Panda acquisition to sticky devices, Magma Cream, Frog food and sulfur-cube growth

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `PLY-005`, `PLY-006`,
`PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`,
`ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`BLK-SLIME-001`, `ENT-001`, `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`,
`ENT-KNOCKBACK-001`, `MOB-001`, `MOB-004`, `MOB-AI-001`, `MOB-BREED-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components/tags, Slime and Panda loot tables, exact
Slime/Frog/Sulfur-Cube consumers, all recipes/advancements and Wandering records, all `1,212`
decoded templates and client resources determine every Slime-Ball-specific branch. Generic death,
loot, crafting, breeding, growth, merchant, persistence, packet and rendering algorithms retain
their cited owners.

**Applies when:**

`minecraft:slime_ball` is emitted by a size-one Slime or Panda sneeze, obtained through a Wandering
offer or Slime-Block decompression, consumed in crafting, offered to a Frog or baby Sulfur Cube,
moved, renamed, persisted, synchronized or rendered before and after tag, recipe, loot, trade or
resource reload.

**Authoritative state:**

Slime Ball is raw item ID `1059`, a common nondamageable plain `Item` with maximum stack `64`. Its
two direct item tags are `frog_food` and `sulfur_cube_food`. It has no food component for players,
consumable, remainder, fuel, compost, equipment, durability, projectile, cooldown, inventory-tick
or intrinsic use branch.

**Transition and ordering:**

### Slime death and Panda sneeze

The Slime entity table runs its sole pool only when the dead Slime's cube-mob size is exactly `1`.
Within that pool the damage-source branches are mutually exclusive:

- when the source entity is not a Frog, emit base `0..2` Slime Balls and add living-attacker
  Looting `round(LU)`;
- when the source entity is a Frog, emit exactly one Slime Ball and do not apply Looting.

Sizes above one emit no Ball from this table; their ordinary death/splitting timeline remains the
Slime owner. The Frog branch is keyed to source entity, not direct attacker or player attribution.

`gameplay/panda_sneeze` has one roll selecting one Slime Ball at weight `1` against Empty weight
`699`, hence exact probability `1/700`. Panda sneeze scheduling and invocation remain the Panda AI
owner; the loot table owns only the output draw. No other entity, chest, fishing, barter,
archaeology or gift table directly emits Slime Ball.

### Four recipes and progression

Four exact recipes join the identity:

- shaped `3x3` Slime Block compression consumes nine Balls and emits one Block;
- shapeless decompression consumes one Slime Block and emits nine Balls;
- shapeless Magma Cream consumes one Slime Ball plus one Blaze Powder and emits one Cream; and
- shaped Sticky Piston places one Ball above one Piston and emits one Sticky Piston.

All four have advancements. Direct Slime-Ball possession is an inventory alternative for Slime
Block and Sticky Piston (`2` direct unlocks); Magma Cream uses Blaze Powder and decompression uses
Slime Block. Recipe knowledge is the other OR alternative. Outputs are default stacks; Slime-Block
physics, piston adhesion, its two block-ingredient brewing edges, Magma-Cream brewing and piston
runtime remain their named owners.

### Frog, sulfur-cube and merchant joins

`Frog#isFood` tests the live `frog_food` tag. Therefore a Ball enters generic Frog temptation,
adult breeding/love and baby age-up paths, consuming one on an admitted interaction outside
infinite-material mode. Removing the tag rejects it without changing the Slime death Frog branch,
which is an entity-type predicate.

A baby Sulfur Cube's interaction and temptation predicate tests live `sulfur_cube_food`. An
admitted Ball is consumed through the generic animal-style feeding path, advances baby age,
plays its eating response and eventually reaches the Sulfur Cube's adult size transition. Adults
do not use this food-tag branch; their swallowable-item/archetype path is independent.

Wandering Trader common selects five distinct candidates from `76`, so the four-Emerald to one
Slime-Ball offer has inclusion probability `5/76`. It has maximum uses `5`, omitted XP decoding to
`1`, reputation discount `0.05`. Trade Rebalance does not replace this record or set.

An exhaustive decoded scan finds zero exact Slime-Ball identities across all `1,212` templates.
Natural Slimes/Pandas and structure entities remain spawn/AI driven rather than stored Ball stacks.

**Persistence and reload boundary:**

Stacks, mobs, knowledge, offers and crafted outputs persist with their owners. Loot, recipe,
advancement, food-tag and trade reload changes only future evaluation; completed deaths, sneezes,
crafts, feeding, growth and trades are not replayed. Existing offers retain constructed
costs/results. Resource reload independently changes projection only.

**Wire and client projection:**

Generic stack publication uses item ID `1059`; no Slime-Ball-specific packet exists. English name
is `Slimeball`. The item definition selects one untinted `item/generated` flat using
`minecraft:item/slime_ball`, without condition, animation or special renderer.

Ingredients orders Turtle Scute, Armadillo Scute, Slime Ball, Clay Ball. Slime Block and Sticky
Piston use their independent block/item projections.

**Branches and aborts:**

Default/patched Ball; Slime size one versus larger; Frog versus non-Frog source and Looting;
Panda hit/miss; four recipes/listeners/two direct unlocks; Frog live tag and adult/baby state;
Sulfur-Cube live tag and baby/adult state; Wandering selected/unselected; zero templates;
persistence/reload/wire/client branches are distinct.

**Constants and randomness:**

Item ID `1059`; stack `64`; ordinary small-Slime base `0..2` plus `round(LU)`, Frog-source output
`1`; Panda sneeze `1/700`; recipes/listeners/direct unlocks `4/4/2`; compression/decompression
`9:1/1:9`; Wandering inclusion `5/76`, exchange `4:1`, uses/XP/discount `5/1/0.05`;
templates/matches `1212/0`.

**Side effects:**

Slime-death and sneeze output; crafted Block, Cream, Sticky Piston and knowledge; Frog breeding/
growth state; Sulfur-Cube growth/eating state; merchant input/output; durable stack/entity state,
synchronization and exact client projection.

**Gates:**

Slime exact size and damage-source entity; attacker/Looting context; Panda loot selection; exact
grid/result capacity and knowledge; Frog/Sulfur-Cube age, live tag, cooldown/love/growth admission;
profession/set/current price; registry/stack decode and client resources.

**Boundary cases and quirks:**

A Frog-caused small-Slime death guarantees one Ball and deliberately bypasses the ordinary
`0..2` plus Looting branch. The same Ball can feed Frogs, but removing `frog_food` affects player
feeding rather than Frog-source death loot. Baby Sulfur Cubes use a food tag while adults use a
separate swallowable-item predicate. Panda sneeze is a true `1/700` loot draw, not a count range.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.storage.loot.functions.EnchantedCountIncreaseFunction`;
`net.minecraft.world.entity.animal.frog.Frog#isFood`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/slime_ball.json`;
`data/minecraft/tags/item/{frog_food,sulfur_cube_food}.json`;
`data/minecraft/loot_table/{entities/slime,gameplay/panda_sneeze}.json`;
`data/minecraft/recipe/{slime_block,slime_ball,magma_cream,sticky_piston}.json`;
`data/minecraft/advancement/recipes/{redstone/{slime_block,sticky_piston},misc/{slime_ball,magma_cream}}.json`;
`data/minecraft/{villager_trade/wandering_trader/emerald_slime_ball,tags/villager_trade/wandering_trader/common,trade_set/wandering_trader/common}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/slime_ball.*`;
`assets/minecraft/lang/en_us.json`;
`BLK-SLIME-001`; `ITM-RECIPE-SERIALIZER-001`; `EXP-ITM-093`.

**Test vectors:**

Run `EXP-ITM-093` across default/patched Ball, every Slime size and Frog/non-Frog/Looting source,
controlled Panda-sneeze selections, four recipes/listeners, Frog adult/baby feeding, baby/adult
Sulfur-Cube feeding and selected/unselected Wandering offers under independent tag/data reload.
Scan every template, persist/reload/synchronize owners and assert ID, name, generated model,
texture and Ingredients position.

**Limits:**

Generic death, loot, crafting, Slime Block/piston, Frog breeding, Sulfur-Cube growth, merchant,
packet and renderer control flow remains with cited owners. Slime, Panda, Frog, Slime Block, Magma
Cream, Sticky Piston and Sulfur Cube retain their dedicated owners. This leaf fixes the exact loose
item, acquisition/sink joins, absences and projection.
