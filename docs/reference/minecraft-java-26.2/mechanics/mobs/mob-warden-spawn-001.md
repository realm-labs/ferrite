# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-WARDEN-SPAWN-001` — Shared warning trackers admit one triggered warden and darkness response

**Parent:** `MOB-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — shrieker ingress and state, per-player warning persistence and synchronization,
the delayed response, candidate search, warden finalization, sound and darkness effects are explicit
in locked source and bundled data.

**Applies when:**

A server-side sculk shrieker attributes a step or admitted vibration to a server player and is not
already shrieking.

**Authoritative state:**

The shrieker's `can_summon`/`shrieking`/`waterlogged` state, block-entity warning and vibration
records, scheduled block ticks, live difficulty and `spawn_wardens` rule, level players and wardens,
each player's persisted warning tracker, blocks/collision/world border, level RNG, constructed
warden state, and nearby players' effects and game modes.

**Transition and ordering:**

**Summoning-capable state and ingress:** The registered default shrieker has `can_summon=false`,
`shrieking=false`, `waterlogged=false`; ordinary placement never enables `can_summon`. The locked
world-generation sculk spread sets it to true on a selected shrieker, and the ancient-city feature's
rare-growth postpass explicitly offers `can_summon=true`. Runtime sculk spread supplies false.
Those generation transactions remain owned by `WGEN-PIPELINE-001`.

The block entity persists its vibration listener data and a raw integer `warning_level`, default
zero; loading does not clamp that local value. Listener-state changes mark the block entity dirty.

Stepping calls `tryGetPlayer`: accept a direct `ServerPlayer`, otherwise its controlling passenger,
otherwise a projectile owner, otherwise an item-entity owner, only when that object is a
`ServerPlayer`. A server step with an attributed player invokes the block entity and then the block
super implementation. Its vibration user has radius 8, requires adjacent chunks ticking, and
listens only to `minecraft:shrieker_can_listen`; locked data contains only
`minecraft:sculk_sensor_tendrils_clicking`. It rejects while the block is already shrieking or when
the event source cannot be attributed by the same helper. Delivery prefers the vibration's
projectile owner argument when present, otherwise its source entity, then attributes again. Generic
vibration travel, occlusion and scheduling remain owned by the shared vibration system.

**Immediate warning transaction:** A null attributed player or `shrieking=true` returns. Otherwise
set the block entity's local warning level to zero. `canRespond` then short-circuits in this order:
`can_summon=true`, difficulty not Peaceful, live `SPAWNING` Boolean `spawn_wardens=true`; that rule
defaults true. When the gate is false, no player tracker changes, but the shriek still commits.
When true, warning admission must succeed or the entire shriek is suppressed.

Warning admission first rejects any warden in `AABB.ofSize(center,48,48,48)`, whose half-extents are
24. It collects level players in encounter order that are alive, nonspectating and strictly within
16 blocks of the shrieker center, then appends the attributed player if absent even if that player
would fail those filters. Any collected tracker's positive cooldown rejects the whole warning with
no changes. Otherwise choose a maximum warning-level tracker, increment it, then copy all three
fields to every collected tracker. The increment resets ticks-since-warning to zero, sets cooldown
to 200 and clamps warning level after `+1` to `[0,4]`. The selected new level is copied into the
shrieker block entity.

Every `ServerPlayer` owns a tracker. Its codec persists nonnegative `ticks_since_last_warning`,
`warning_level` and `cooldown_ticks`, each default zero; decoded warning levels are not clamped.
Respawn restoration retains the same tracker object. Each player tick first updates warning decay:
a starting ticks-since value below 12000 increments, while a starting value at least 12000
decrements warning through the clamped setter and resets the counter. It then decrements a positive
cooldown. Thus ordinary zero state decays/reset on tracker tick 12001, and the interval resets even
at warning zero.

**Shriek and delayed response:** A committed shriek offers `shrieking=true` with flags 2, schedules
a block tick after 90 ticks, emits level event 3007, and emits game event `SHRIEK` attributed to the
player. Mutation results are ignored. On the scheduled tick, only an incoming
`shrieking=true` state is cleared with flags 3 before `tryRespond`. Removing a still-shrieking block
also invokes `tryRespond` through its pre-removal side effect.

Response rechecks the three `canRespond` gates and requires the block entity's local warning above
zero. Failure does nothing. At warning 1, 2 or 3, the warden spawn helper returns false immediately;
warning 4 attempts a spawn. Any failed/no spawn selects reply sounds `WARDEN_NEARBY_CLOSE`,
`WARDEN_NEARBY_CLOSER`, `WARDEN_NEARBY_CLOSEST`, or `WARDEN_LISTENING_ANGRY` for levels 1..4.
It consumes three independent inclusive level-RNG offsets `-10..10` in X, Y, Z order and plays at
the resulting integer block position with source null, `HOSTILE`, volume 5 and pitch 1. A successful
spawn suppresses that sound and those three draws. After either a successful spawn or a reply,
darkness is applied; neither outcome resets any tracker or the block entity's warning level.

**Warning-four warden search:** Call `SpawnUtil.trySpawnMob` for `WARDEN`, reason `TRIGGERED`, at
the shrieker position with 20 attempts, horizontal range 5, vertical range 6,
`ON_TOP_OF_COLLIDER`, and the precreation collision flag false. Each attempt consumes X then Z
offsets independently and inclusively from `-5..5`, starts six blocks above the shrieker, and first
requires the probe inside the world border. The downward search tests 13 ground cells from
`centerY+5` through `centerY-7`: the cell above must have empty collision shape and the ground cell
must have a full upward collision face. The resulting entity position spans `centerY+6` through
`centerY-6` and uses block-center X/Z.

The skipped precreation flag means no type-AABB collision check occurs before construction.
Creation consumes level RNG for yaw and ordinary Mob finalization, including follow-range bonus and
left-handed trial. Warden finalization first installs brain memory `DIG_COOLDOWN` for 1200 ticks;
for `TRIGGERED` it sets pose `EMERGING`, installs `IS_EMERGING` for 134 ticks and plays
`WARDEN_AGITATED` at volume 5/pitch 1 before superclass finalization. The generic spawn-rule check
is true; obstruction then requires no liquid, an unobstructed entity and no collision against the
warden type's default-dimension box. A postconstruction failure discards the object and continues,
so its RNG and aggravated sound are not rolled back. The first passing object is inserted with
passengers without checking retention, then plays its ambient sound and counts as success.

**Darkness response:** From the shrieker block center, apply Darkness for 260 ticks, amplifier zero,
nonambient and without particles, to Survival or Adventure players strictly within 40 blocks.
Creative and Spectator players are excluded. With null source there is no ally exclusion. A player
without Darkness is admitted; an existing same-or-higher amplifier is refreshed only when its
finite duration is at most 199 ticks. Each admitted player receives a copied instance with null
source; the return value is ignored.

**Branches and aborts:**

Unattributed step/vibration; adjacent chunks unavailable; listener tag/occlusion/travel rejection;
already shrieking; incapable block, Peaceful or false rule; nearby warden; any collected tracker on
cooldown; delayed gate disabled; local warning zero; warning below four; every world-border/ground/
construction/obstruction failure; reply-sound missing for an out-of-range decoded local warning;
and darkness game-mode/range/effect-retention rejection.

**Constants and randomness:**

Listener radius 8; nearby-player radius strict 16; warden suppression box 48 wide; tracker cooldown
200, decay interval boundary 12000 and warning clamp 0..4; shriek delay 90; reply offsets three
inclusive `-10..10`; 20 spawn attempts, inclusive horizontal `-5..5`, vertical search 13 ground
cells around range 6; emerge 134 and dig cooldown 1200; darkness radius strict 40, duration 260 and
refresh endpoint 199. All reply, spawn construction/finalization and reached sound work uses level
RNG in the order above.

**Side effects:**

Tracker persistence/synchronization and cooldown, block-entity dirty/persisted warning/vibration
state, block state and scheduled tick, level/game events, presentation sounds/particles, RNG,
discarded or inserted warden objects and brain/pose state, and per-player Darkness effects. The leaf
does not own later warden AI, anger, digging, combat or despawn.

**Gates:**

Attribution, vibration admission, not already shrieking, `can_summon`, difficulty,
`spawn_wardens`, nearby-warden and shared-player cooldown checks, delayed recheck, warning level,
candidate ground/world-border/construction/obstruction, and darkness audience/effect checks.

**Boundary cases and quirks:**

False `spawn_wardens`, Peaceful, or `can_summon=false` still allows the visible 90-tick shriek but
cannot advance warnings. Disabling the rule after warning admission preserves the already
synchronized tracker/cooldown while suppressing the delayed response; enabling it after an
initially gated shriek cannot respond because local warning is zero. `spawn_mobs` and
`spawn_monsters` are not read. A warden suppresses the initial warning rather than merely the spawn.
One nearby player's cooldown suppresses everyone, and the attributed player is forcibly joined.
Failed constructed candidates can be audible. Successful insertion is not verified before darkness.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.level.gamerules.GameRules`;
`net.minecraft.world.level.block.SculkShriekerBlock#stepOn`, `#tick`;
`net.minecraft.world.level.block.entity.SculkShriekerBlockEntity#tryGetPlayer`, `#tryShriek`,
`#tryRespond`, `#preRemoveSideEffects`;
`net.minecraft.world.level.block.entity.SculkShriekerBlockEntity$VibrationUser`;
`net.minecraft.world.entity.monster.warden.WardenSpawnTracker#tick`, `#tryWarn`;
`net.minecraft.util.SpawnUtil#trySpawnMob`;
`net.minecraft.world.entity.monster.warden.Warden#checkSpawnObstruction`, `#finalizeSpawn`,
`#applyDarknessAround`; `net.minecraft.world.effect.MobEffectUtil#addEffectToPlayersAround`;
`net.minecraft.world.level.block.SculkBlock#getRandomGrowthState`;
`net.minecraft.world.level.levelgen.feature.SculkPatchFeature#place`;
`data/minecraft/tags/game_event/shrieker_can_listen.json`; `WGEN-PIPELINE-001`;
`CLI-EFFECT-001`; `EXP-MOB-009`.

**Test vectors:**

Cross default/placed/runtime/worldgen `can_summon`, Peaceful and live rule transitions before
warning and response. Cover direct, controlling-passenger, projectile-owner and item-owner
attribution; step and vibration ingress; tag/radius/adjacent-chunk/occlusion/already-shrieking
boundaries. Exercise nearby wardens on every box face, player radius equality, dead/spectator/
forced trigger players, tracker ties, arbitrary decoded values, any-member cooldown, tick 12000/
12001, persistence and respawn copy. At warning levels 0..4, test scheduled and removal response,
rule toggles and local-warning persistence. Force every attempt offset, vertical endpoint, border,
shape, liquid/obstruction/default-box, null construction, failed insertion and successful spawn;
assert both RNG cursor and nonrollback sounds/finalization. Finally cross all darkness modes, strict
radius, absent/infinite/finite durations 199/200, amplifiers and ignored application results.
