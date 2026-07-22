# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SPAWNER-001` — An ordinary spawner freezes behind its live rule, then attempts an ordered entity batch

**Parent:** `BLK-003`, `BLK-007`, `ENT-001`, `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the live gamerule and player gate, saved configuration, delay, candidate batch,
recursive entity load, mob checks/finalization/equipment, insertion/effects, spawn-egg editing and
client animation are explicit in locked server/client source.

**Applies when:**

A loaded `spawner` block entity ticks on either side, is loaded/saved/synchronized or rendered, or
a valid spawn egg is used on an ordinary or trial spawner. `BLK-TRIAL-SPAWNER-001` owns the trial
spawner state machine and its subtype-specific result of the shared egg interaction.

**Authoritative state:**

The server's live `spawner_blocks_work` rule, players, difficulty, blocks/collision/light and RNG;
the block entity's delay, weighted potentials, selected `SpawnData` and seven numeric settings;
selected entity NBT/custom light/equipment data; nearby entities and constructed entity tree; the
spawn-egg stack/player/type; block updates/events and client-side copied spawner/display/spin state.

**Transition and ordering:**

**Installation, defaults and outer gates:** `SpawnerBlock#getTicker` selects the client or server
ticker only for `mob_spawner`. A new `BaseSpawner` starts delay 20, empty potentials, no selected
data, minimum/maximum delay 200/800, spawn count 4, nearby limit 6, required-player range 16 and
spawn range 4. The `MISC` Boolean `spawner_blocks_work` defaults true.

Every server tick first asks for an alive nonspectating player strictly within required range of
the block center. A negative externally loaded range admits the first alive nonspectator at any
distance. No qualifying player returns before reading the gamerule. Otherwise false
`spawner_blocks_work` returns immediately. Both gates freeze delay, selection, RNG and entity work;
`spawn_mobs` and `spawn_monsters` are not read by this ordinary path.

**Delay and selected data:** On an admitted tick, delay exactly `-1` invokes `delay` first. A
positive delay then decrements once and returns. Any other nonpositive value enters the batch.
`delay` sets the next value to `minSpawnDelay` without RNG when `maxSpawnDelay<=minSpawnDelay`;
otherwise it consumes `nextInt(max-min)` and adds `min`, making the maximum exclusive. It then
makes one weighted-potential selection when available, replaces selected data only on a present
result, and broadcasts block event 1. The ordinary subtype's selected-data setter also sends a
same-state block update with flags 260. Normal countdown, selection and batch work do not call the
block entity's `setChanged`.

Missing selected data is obtained lazily from the weighted list through the same random selection;
an empty list installs an empty `SpawnData`. If its entity NBT has no valid type, call `delay` and
abort the whole tick. Otherwise run `spawnCount` attempts in index order. Ordinary zero/negative
spawn counts perform no attempts, leave a due delay due and therefore retry that empty batch every
admitted tick.

**Candidate position and preconstruction gates:** Each attempt creates a scoped problem reporter.
NBT `Pos` supplies the exact vector without position RNG. Otherwise consume X two doubles, Y one
`nextInt(3)`, then Z two doubles, producing
`(blockX+(d1-d2)*spawnRange+0.5, blockY+y-1,
blockZ+(d3-d4)*spawnRange+0.5)`. First require `noCollision` for the entity type's spawn AABB at
that vector.

With custom spawn rules, a nonfriendly type is rejected in Peaceful, then block light and effective
sky light at the containing block must lie in their independent inclusive configured ranges. Both
ranges decode only inside 0..15. With no custom rules, instead call the registered placement
predicate for reason `SPAWNER` before construction. These branches are exclusive.

**Recursive load and mob transaction:** Recursively load the entity and passengers from the full
NBT with reason `SPAWNER`, snapping every returned root to the candidate vector. Null load calls
`delay` and aborts the whole tick. Count existing entities of the root's **exact runtime class**,
excluding spectators, in the unit spawner-cell AABB inflated by `spawnRange`. A count at least
`maxNearbyEntities` calls `delay` and aborts the whole tick.

Snap the loaded root again at its current position with one level-RNG `nextFloat*360` yaw and pitch
zero. For a Mob with no custom rules, call its `checkSpawnRules` for `SPAWNER`; custom rules skip
that second mob check. Every Mob must pass `checkSpawnObstruction`. Entity NBT containing exactly
one key named `id` receives `finalizeSpawn` at current regional difficulty with null group data;
richer NBT skips finalization. Optional `EquipmentTable` then calls `Mob#equip`, including its
separately owned loot evaluation and drop chances. Non-Mobs bypass all mob-only steps.

`tryAddFreshEntityWithPassengers` failure calls `delay` and aborts the whole tick; construction,
finalization and equipment effects are not rolled back. Success emits level event 2004 at the
spawner, game event `ENTITY_PLACE` at the candidate block attributed to the root, and `spawnAnim`
for a Mob, then continues with the next attempt. Preconstruction collision, light/placement and
postload mob-rule/obstruction failures skip only that attempt and do not postpone retry. After the
full batch, one or more successes call `delay`; zero successes leave the due delay unchanged for an
immediate next admitted-tick retry.

**Spawn-egg edit and shared rule gate:** A spawn egg first resolves its component-selected type and
requires that type can spawn in the level. The client then predicts success without inspecting the
target. On the server, when the clicked block entity implements `Spawner`, false
`spawner_blocks_work` sends translatable `advMode.notEnabled.spawner` only to a `ServerPlayer` and
returns failure before selected-data RNG, override, block update, game event or item shrink. This
does not gate ordinary spawn-egg placement against a nonspawner target.

When enabled, ordinary `setEntityId` obtains selected data if absent, possibly selecting a weighted
potential, writes its `id` and marks the block entity changed. The item path then sends a same-state
update with flags 3, emits `BLOCK_CHANGE` at the spawner with the player as source, shrinks the stack
by one and returns success. It neither resets delay nor clears other entity NBT, custom rules or
equipment. The supplied RNG is ignored when selected data already exists. On a trial spawner, the
same enabled transaction instead resets encounter data, wraps both normal/ominous configs with the
new entity type, preserves cooldown/range config, writes inactive and marks changed as specified by
`BLK-TRIAL-SPAWNER-001`.

**Persistence and client projection:** Save writes delay and the six settings as signed shorts,
plus nullable selected data and the weighted potential list. Load defaults to the constructor
values, accepts decoded numeric values without cross-field validation, derives a singleton
potential from selected data when potentials are absent, and clears the transient display entity.
The update tag contains these fields except `SpawnPotentials`.

The client never reads `spawner_blocks_work`. With any alive nonspectating client-level player strictly in
range and a display entity already constructed by render extraction, each client tick consumes
three doubles for one shared smoke/flame position, decrements positive local delay, copies current
spin to old spin and advances spin modulo 360 by `1000/(delay+200)`. Without a qualifying player it
only copies spin to old spin; without a display entity it does nothing. Block event 1 is accepted on
both sides but resets delay to `minSpawnDelay` only client-side, so presentation is not the server's
randomized countdown and can continue while the server rule is false.

Render extraction lazily loads the selected NBT tree as a nonworld display entity with root ID -1,
then extracts it at partial tick. Spin is `10*lerp(oldSpin,spin)`. Scale starts `0.53125` and divides
by `max(width,height)` only when that maximum exceeds one. Submission translates to
`(0.5,0.4,0.5)`, rotates Y by spin, translates Y `-0.2`, rotates X `-30` degrees and applies that
uniform scale.

**Branches and aborts:**

Wrong/missing ticker type; no qualifying player; false rule; positive delay; invalid entity type;
each collision/custom/generic/load/exact-class-cap/mob-rule/obstruction/insertion result; empty or
zero attempt batch; absent display entity; spawn-egg type/level failure, nonspawner target or false
rule; and client range/display/render availability.

**Constants and randomness:**

Defaults 20, 200/800, 4 attempts, limit 6, player range 16 and spawn range 4; Y draw `-1..1`;
level event 2004; block event 1; client smoke/flame three doubles and spin numerator 1000; display
scale 0.53125. Level RNG owns delay/potential selection, candidate coordinates, yaw, finalization,
equipment and reached entity effects in the exact order above. Explicit NBT `Pos`, rich versus
one-key entity data, and each abort change the cursor.

**Side effects:**

In-memory/saved spawner fields, block update/event packets, client particles/spin/display render,
level RNG, recursive entity/passenger construction, mob finalization/equipment, insertion attempts,
level/game/entity effects, spawn-egg stack mutation, dirty state and failure message.

**Gates:**

Loaded/compatible ticker, alive nonspectator player, live `spawner_blocks_work`, delay and attempt
count, selected entity data, candidate collision and custom/generic position rules, exact-class cap,
mob rules/obstruction, insertion, spawn-egg target/type and client-local presentation gates.

**Boundary cases and quirks:**

The rule freezes server state but not client animation. An inactive distant spawner does not even
read it. Failed ordinary candidates retry every active tick unless invalid type, null load, nearby
cap or insertion explicitly calls `delay`; those late resets preserve already consumed RNG and
entity side effects. One success does not stop the batch. Nearby limits compare exact loaded class,
not entity type, and rich NBT bypasses finalization. Spawn eggs edit both spawner kinds only when the
rule is true, but the rule does not disable using the egg to place an entity elsewhere.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.server.level.ServerLevel#isSpawnerBlockEnabled`;
`net.minecraft.world.level.block.SpawnerBlock#getTicker`;
`net.minecraft.world.level.block.entity.SpawnerBlockEntity`;
`net.minecraft.world.level.BaseSpawner#clientTick`, `#serverTick`, `#load`, `#save`,
`#getOrCreateDisplayEntity`, `#onEventTriggered`;
`net.minecraft.world.level.SpawnData`, `SpawnData$CustomSpawnRules#isValidPosition`;
`net.minecraft.world.item.SpawnEggItem#useOn`;
`net.minecraft.client.renderer.blockentity.SpawnerRenderer#extractRenderState`,
`#submitEntityInSpawner`; `BLK-TRIAL-SPAWNER-001`; `CLI-EFFECT-001`; `EXP-BLK-016`.

**Test vectors:**

Cross live rule, no/dead/spectator/alive players and strict/negative range; delay
`-2/-1/0/1`, min/max order and signed save/load; empty/one/many weighted data and counts
`-1/0/1/4`. For every attempt force explicit/generated positions and each gate, non-Mob/Mob,
custom/ordinary rules, Peaceful, exact-class cap, one-key/rich NBT, passengers, equipment and
insertion failure/success; assert retry mode, block/entity state, packets/effects and exact RNG.
Use every spawn egg on ordinary/trial/nonspawner targets on both logical sides with rule true/false,
existing/absent selected data and creative/survival stacks. Finally cross update/load/event order,
client range/display cache/delay endpoints, rule-disabled animation and display dimensions/partial
ticks through the exact renderer transforms.
