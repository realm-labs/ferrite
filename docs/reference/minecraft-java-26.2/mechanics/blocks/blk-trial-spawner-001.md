# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-TRIAL-SPAWNER-001` — Trial spawners detect a cohort, run a bounded encounter, and eject one reward per registered player

**Parent:** `BLK-003`, `BLK-007`, `ENT-001`, `ENT-006`, `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server classes and all 28 locked trial-spawner configurations
determine the six-state server transition, player/omen admission, target counts, mob creation and
tracking, ominous overhead-item attempts, reward timing, persistence and client publication.
Spawn-placement/finalization and loot-table evaluation dispatch into their separately owned generic
algorithms, but every input, gate and call position supplied by this subtype is fixed here.

**Applies when:**

A loaded `trial_spawner` block entity ticks on either side, its block state is changed between
`inactive`, `waiting_for_players`, `active`, `waiting_for_reward_ejection`, `ejecting_reward` and
`cooldown`, a spawn egg overrides its entity type, or its saved/update data is loaded. Trial-chamber
template placement and the 28 exact configuration payloads remain data-owned by
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`; this leaf owns their later runtime interpretation.

**Authoritative state:**

The server owns block `state` and `ominous`; normal/ominous configuration holders; target cooldown
and required-player range; unordered registered-player and current-mob UUID sets; absolute cooldown
and next-spawn times; total spawn count; optional next `SpawnData` and fixed ejection table; level
time/RNG/difficulty/gamerules; live players/entities/geometry; loot registries and entity admission.
Persisted state contains those two UUID sets, both absolute times, count, next spawn data, ejection
table and full config. Display entity, item cache and spin are transient. The client receives only
next-spawn time while active plus optional spawn data; it derives particles, sound and spin from
synchronized block state/data.

**Transition and ordering:**

The server first copies block `ominous` into the runtime, then removes every tracked UUID whose
entity is missing, dead, in another dimension or at block-position squared distance strictly greater
than `47²=2209`; any removal postpones the next spawn to `now+ticks_between_spawn`.
`spawner_blocks_work=false` always disables encounters. Otherwise a test override bypasses the
remaining gates; ordinary operation also requires non-Peaceful difficulty and `spawn_mobs=true`.
Active normal/ominous config selects by block state. Targets are
`floor(total_mobs + added_total*additionalPlayers)` and
`floor(simultaneous_mobs + added_simultaneous*additionalPlayers)`, where additional players are
`max(0,registered-1)`. Locked defaults are range `4`, totals `6+2p`, simultaneous `2+p`, spawn
interval `40`, required range `14` and target cooldown `36,000`; per-record overrides are those
audited in the jigsaw leaf.

**Player scans and omen conversion:**

A scan is admitted exactly when `(pos.asLong()+gameTime) mod 20 == 0`. The ordinary detector selects
noncreative, nonspectator players whose block positions are strictly closer than required range. The
first cohort scan additionally requires a reverse visual ray from player eye to spawner center to
miss or terminate in the spawner cell; later additions do not require sight. During an ominous
cooldown, scanning stops. Before cohort admission, a nonominous spawner searches the sight-qualified
list in encounter order: the first Trial Omen holder wins immediately; otherwise the last Bad Omen
holder wins. Bad Omen amplifier `a` is removed and replaced by Trial Omen amplifier zero for exactly
`18,000*(a+1)` ticks; an existing Trial Omen is retained. The player receives event `3020` data `0`,
then the spawner writes `ominous=true`, emits event `3020` data `1`, and resets as below.

Every newly added UUID raises `nextMobSpawnsAt` to at least `now+40` and, unless this scan caused
conversion, emits event `3013` normal or `3019` ominous with the resulting cohort size. A cooldown
scan that did not convert returns before adding anyone. Conversion discards every currently tracked
entity after event `3012` normal-flame data and, for mobs, preserved-equipment drops; failed/missing
lookups are skipped. It clears the selected spawn data only when the ominous potential list is
nonempty, zeroes the total/current set, sets next spawn to `now+ominous interval`, marks/publishes
data, and sets the item-spawner timer to `now+160`. Thus a normal cooldown can be interrupted only
by an admitted omen conversion.

**Six-state server transition:**

After the prelude, exactly one current-state branch runs and at most one state write occurs.

1. `inactive` obtains/chooses next spawn data and tries to construct the waiting state's display
   entity. A missing or unloadable entity ID stays inactive; otherwise it becomes
   `waiting_for_players`.
2. `waiting_for_players` clears encounter statistics and stays put while disabled. With no usable
   selected ID and no potential it returns inactive. Otherwise it scans; a nonempty registered set
   becomes active.
3. `active` performs the same disable/no-mob fallbacks. It snapshots additional-player count
   **before** this tick's scan, then scans and, when ominous, runs the overhead-item attempt below.
   If total target has already been reached, it waits for the tracked set to empty; that tick sets
   cooldown end `now+targetCooldown`, zeroes total/next-spawn time and enters
   `waiting_for_reward_ejection`. Otherwise it attempts one mob only when `now>=nextMobSpawnsAt` and
   current tracked count is below the simultaneous target. A successful admission appends the root
   UUID, increments total, sets next spawn to `now+interval`, then independently chooses/publishes
   the next potential. Failure changes none of those counters or time, so the next active tick
   retries.
4. `waiting_for_reward_ejection` stays until float-converted `now >= float(cooldownStart)+40`, plays
   open-shutter, and enters `ejecting_reward`. Ordinary world times produce the intuitive 40-tick
   delay; float rounding of large absolute game times is retained.
5. `ejecting_reward` acts only when `float(now-cooldownStart) mod 30.0f == 0`. An empty cohort then
   plays close-shutter, clears its fixed table and enters cooldown. Otherwise it chooses the active
   config's weighted ejection table once if absent, evaluates that same table on every admitted
   ejection, upward-dispenses every returned stack at speed `2` from `bottom-center+1.2Y`, emits
   `3014` only for a nonempty result, and removes one UUID after evaluation. The last player's
   ejection therefore precedes shutter close by the next 30-tick qualifying instant; corrupt or
   externally authored elapsed values beyond exact float integers retain float-modulo behavior.
6. `cooldown` scans only as described above. An omen conversion with a newly populated cohort zeroes
   total/next time and returns active. At `now>=cooldownEndsAt`, it writes `ominous=false`, clears
   current mobs/next spawn/cohort/counters/times, and returns to waiting. Otherwise it remains
   cooldown.

**Mob attempt transaction:**

Obtaining missing next data first makes one weighted-potential choice and publishes it; an empty
list supplies empty `SpawnData`. A missing/unknown entity type aborts. Unless NBT supplies `Pos`,
choose X/Z as center plus `(nextDouble-nextDouble)*spawnRange` and Y as `spawnerY+nextInt(3)-1`.
Abort, in order, for spawn-AABB collision, failed reverse visual ray to the spawner cell, generic
`TRIAL_SPAWNER` placement rejection, or custom light-rule rejection. Recursive load snaps yaw from
`nextFloat*360`; null load aborts. A mob must additionally pass obstruction. Exactly an entity tag
containing only `id` triggers ordinary `finalizeSpawn`; richer NBT does not. Every mob becomes
persistent and receives optional configured equipment before `tryAddFreshEntityWithPassengers`.
Admission failure leaves no encounter counters; success emits spawner/target events `3011/3012` with
normal or ominous flame data, then `ENTITY_PLACE`. Passenger creation and admission follow generic
entity ownership, while this leaf fixes their transaction position.

**Ominous overhead items:**

The first active ominous tick evaluates `items_to_drop_when_ominous` with the deterministic seed
`levelSeed + BlockPos(floor(x/30f),floor(y/20f),floor(z/30f)).asLong`; results are cached as a
weighted list of count-one stack copies weighted by original counts. Every later active ominous tick
still makes one weighted gameplay-RNG selection **before** testing the 160-tick timer. Empty choice
aborts. When due, live registered players are filtered to alive, noncreative, nonspectator and
within squared required range. One gameplay Boolean chooses tracked mobs or players without falling
back if the chosen list is empty; a list larger than one consumes a random index. A ray goes upward
from that entity by `height+2+nextInt(4)` and proposes one block-center below the hit; a
collision-bearing block there aborts. Otherwise an `ominous_item_spawner` carrying the chosen stack
is offered at that point. Its admission Boolean is ignored: two floats still determine pitch, begin
sound plays, and the next timer becomes `now+160`.

**Persistence, client, and quirks:**

State/config save uses lenient defaults; loading replaces packed fields/config and publishes when
already attached. The shared spawn-egg path first requires a spawnable component-selected entity
type. False `spawner_blocks_work` sends `advMode.notEnabled.spawner` to a server player and returns
failure before override, update, event or stack shrink; the client predicts success without this
gate. When true, trial override ignores the supplied RNG, resets encounter data, replaces both
normal and ominous configurations with direct copies whose spawn potentials contain only the new
entity type, preserves target cooldown and player range, sets block state inactive and marks
changed. The item path then sends a flags-3 update, emits player-attributed `BLOCK_CHANGE`, shrinks
the stack by one and succeeds. Selection changes
publish the compact update tag. The transient display entity is not invalidated when later spawn
data changes, so an already constructed renderer can continue showing its first entity. Waiting and
both reward states make one half-probability small-flame position inside center-offset `0.9`; active
emits smoke plus normal/soul flame every tick inside offset `1`; cooldown makes a one-third smoke
attempt inside `0.9` and, every 20 client ticks, emits `20+nextInt(4)` smoke at top-center. Inactive
emits none. Waiting and active additionally make a per-tick `nextFloat<=0.02` ambient branch
followed by independent volume/pitch floats. Spinning stores old spin then advances modulo 360 by
`200`-speed waiting or `1000`-speed active divided by `max(0,nextSpawnAt-clientTime)+200`. Light is
`0/4/8` for inactive-or-cooldown/waiting/active-or-reward states.

Client level events map server IDs exactly: `3011` makes 20 smoke-plus-selected-flame pairs around
the spawner; `3012` first uses two floats for spawn-mob pitch at the mob cell and makes the same 20
pairs; `3013/3019` use two pitch floats then make `30+5*min(data,10)` normal/ominous detection
particles; `3020` uses volume `0.3` for player data zero or `1` for spawner data one, two pitch
floats, 30 ominous detection particles, then 20 Trial-Omen plus soul-flame pairs with Gaussian
velocity; and nonempty reward event `3014` uses two pitch floats then makes 20 small-flame/smoke
pairs. Every pair shares its sampled position; become/eject pairs share their sampled Gaussian
vector, with eject small-flame Z velocity multiplied by `0.25`. Normal runtime flame data is only
`0/1`. Hash-set player removal order is not reward attribution: the table is shared and every
registered UUID contributes exactly one evaluation.

**Branches and aborts:**

Ticker/activity gate; gamerules/difficulty; selected config/data; scan phase, distance/sight/mode
and omen kind; six current states; target/simultaneous floors; tracked-entity
liveness/dimension/range; every spawn parse/geometry/rule/load/obstruction/admission result; cached
item-table empty; timer/list coin/geometry/admission; ejection table/result; saved versus transient
fields.

**Constants and randomness:**

Scan modulus `20`, initial spawn buffer `40`, tracking radius `47`, cooldown `36,000`, reward-open
delay `40`, ejection period `30`, ominous-item period `160`, Trial Omen `18,000*(a+1)`, ambient
threshold inclusive `0.02`, and all draw positions/bounds are stated above. Weighted spawn, item and
loot algorithms remain with their generic owners.

**Side effects:**

Block-state/light and block-entity dirty/publication changes; omen effects; entity discard/equipment
drops/spawn/admission; persistent UUID/counter/time/config data; loot evaluation and item entities;
level/game events; sounds/particles; client display/spin.

**Gates:**

Loaded compatible block entity and normal gameplay tick; `spawner_blocks_work`; difficulty and
`spawn_mobs`; state/config/selected entity; position-phased player distance, mode and sight; omen
effect; total/simultaneous target; tracked liveness/dimension/range; absolute spawn/item/reward
times; collision, visual ray, generic/custom spawn rules, obstruction and entity admission;
loot-table result; spawn-egg type, rule and target.

**Boundary cases and quirks:**

Failed mob attempts do not postpone a retry, but pruning any stale tracked UUID does. Rich entity
NBT bypasses ordinary mob finalization, while one-key `id` data receives it. Ominous overhead
selection consumes a weighted draw before the timer test, can choose an empty candidate side without
fallback, and advances its timer after valid geometry even when entity admission fails. An empty
reward result still removes one cohort UUID. The ejection table is fixed at its first reward
attempt. Absolute reward opening and saved corrupt elapsed values retain float rounding/modulo.
Display-entity and ominous-item caches are transient and not invalidated by later selected/config
data on the same object. UUID-set iteration is not insertion order and supplies candidate/removal
traversal, but reward count is one evaluation per registered UUID rather than per-UUID loot
attribution.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.TrialSpawnerBlock#getTicker`,
`net.minecraft.world.level.block.entity.TrialSpawnerBlockEntity`,
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawner#tickServer`, `#spawnMob`,
`#applyOminous`, `#ejectReward`,
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerState#tickAndGetNext`,
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerStateData#tryDetectPlayers`,
`#resetAfterBecomingOminous`, `#getDispensingItems`, and
`net.minecraft.world.level.block.entity.trialspawner.PlayerDetector`;
`net.minecraft.world.item.SpawnEggItem#useOn`; `BLK-SPAWNER-001`.

**Test vectors:**

Sweep all six states at exact time inequalities and both ominous values; all gamerule/difficulty
combinations; scan modulus/range/sight/mode/Bad-versus-Trial-Omen branches; zero/one/many players
and tracked mobs; all 28 config formulas; every mob-attempt abort with draw traces; tracking
distance `47` and one block beyond; total/simultaneous boundaries; item list coin including empty
chosen side and rejected entity admission; reward timings `+39/+40/+59/+60` and 30-tick periods;
empty/nonempty loot; save/reload/update-tag and stale transient caches. `EXP-BLK-006` is the
encounter conformance matrix; `EXP-BLK-016` crosses the shared spawn-egg/rule interaction.
