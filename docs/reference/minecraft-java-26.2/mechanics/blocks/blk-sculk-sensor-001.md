# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-SCULK-SENSOR-001` — Vibration selection becomes a distance-delayed, frequency-bearing redstone pulse

**Parent:** `SIM-003`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`,
`PLY-002`, `RED-001`, `RED-003`, `ITM-007`, `ENV-002`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server classes, block and block-entity reports, bundled game-event and
block tags, and generic client effect handlers fix the ordinary and calibrated sculk-sensor
transaction. This rule owns sensor-specific vibration admission and selection, travel, activation,
directional redstone, resonance, scheduled phase changes, persistence and visible effects. Generic
game-event publication, block updates, scheduled queues, advancement evaluation and packet codecs
retain their existing owners.

**Applies when:**

A listenable game event is published within a sensor's range, an entity steps on the sensor, a
candidate vibration is selected, travels, reloads or arrives, the sensor activates/deactivates,
redstone or comparator output is queried, a neighboring resonator reacts, the sensor is placed,
waterlogged, removed, saved or loaded, or either client-visible particle path runs.

**Authoritative state:**

An ordinary sensor has `power=0..15`, `sculk_sensor_phase={inactive,active,cooldown}` and
`waterlogged={true,false}`. Its 96 locked states are IDs `27163..27258`:

`27163 + 6*power + 2*phaseIndex + waterIndex`, where phase indices are inactive `0`, active `1`,
cooldown `2`, and water indices are true `0`, false `1`. The default is ID `27164`.

A calibrated sensor adds horizontal `facing={north,south,west,east}` in that order. Its 384 states
are IDs `27259..27642`:

`27259 + 96*facingIndex + 6*power + 2*phaseIndex + waterIndex`. The default is north, power zero,
inactive and dry, ID `27260`. Placement uses the placing context's horizontal direction directly;
rotation and mirror transform that property.

Both blocks default to power zero and inactive, expose the same 16×8×16-pixel outline, are not
pathfindable, use their shape for light occlusion, and emit five experience through the separately
owned break/experience transaction when its caller Boolean admits XP. Their block-entity protocol
IDs are ordinary `34` and calibrated `35`.

Each entity starts with last vibration frequency `0`, no current vibration, travel delay `0`, an
empty same-tick selector and no particle-reload request. Its listener position is the block
position. Ordinary listener radius is `8`; calibrated radius is `16`.

**Transition and ordering:**

Published events first pass tag/entity/state, subtype and occlusion gates and join a same-tick
selector. On a later game time the winner becomes current, sends its traveling particle, and starts
a floor-distance countdown. Arrival waits for the surrounding 3×3 chunks, records frequency,
computes distance power, writes active state, schedules phase work, notifies redstone, resonates
adjacent amethyst, emits the tendril event and optionally plays a dry click. Scheduled work changes
active to cooldown and power zero, then cooldown to inactive.

**Listenable events and frequencies:**

The listener accepts only the locked `minecraft:vibrations` game-event tag. Frequency zero is
always rejected. Each row also includes `resonate_N` at its row number:

| Frequency | Other locked events |
|---:|---|
| 1 | `step`, `swim`, `flap` |
| 2 | `projectile_land`, `hit_ground`, `splash`, `bounce` |
| 3 | `item_interact_finish`, `projectile_shoot`, `instrument_play` |
| 4 | `entity_action`, `elytra_glide`, `unequip` |
| 5 | `entity_dismount`, `equip` |
| 6 | `entity_interact`, `shear`, `entity_mount` |
| 7 | `entity_damage` |
| 8 | `drink`, `eat` |
| 9 | `container_close`, `block_close`, `block_deactivate`, `block_detach` |
| 10 | `container_open`, `block_open`, `block_activate`, `block_attach`, `prime_fuse`, `note_block_play` |
| 11 | `block_change` |
| 12 | `block_destroy`, `fluid_pickup` |
| 13 | `block_place`, `fluid_place` |
| 14 | `entity_place`, `lightning_strike`, `teleport` |
| 15 | `entity_die`, `explode` |

The frequency map is code-locked rather than inferred from tag order. Events absent from the map
resolve to zero even if custom data adds them to the listenable tag.

**Generic event admission:**

The ordinary listener rejects in this order:

1. any event while `currentVibration` is nonnull;
2. an event outside `minecraft:vibrations`;
3. a spectator source entity;
4. a carefully stepping source plus an event in `ignore_vibrations_sneaking`;
5. a source entity whose type/state dampens vibrations;
6. an affected block state in `dampens_vibrations`;
7. an unavailable listener position;
8. the sensor subtype gates below;
9. a fully occluded route.

The careful-step rejection triggers `AVOID_VIBRATION` first when the source is a server player.
The locked ignore tag contains `hit_ground`, `projectile_shoot`, `step`, `swim`,
`item_interact_start` and `item_interact_finish`; `item_interact_start` is already absent from the
listenable tag. Wool and wool carpets dampen affected states, while only wool belongs to the
occlusion tag.

Subtype admission rejects same-position `block_destroy` and `block_place`, frequency zero, and any
phase other than inactive. A calibrated sensor additionally reads ordinary redstone from the block
one step opposite its `facing`, queried in that opposite direction. Input zero is an unfiltered
wildcard; input `1..15` admits only the equal frequency. This input is sampled at event admission,
not again at selection or arrival.

Occlusion casts six source-block-center-to-listener-block-center rays, each source offset
`9.999999747378752e-6` toward one direction. The event is occluded only when all six rays hit an
`occludes_vibration_signals` block; one clear ray admits it.

**Direct step path:**

Server-side `stepOn` runs only while the supplied state is inactive and the entity is not a Warden.
With a matching sensor entity, it calls the subtype `canReceiveVibration` check for frequency-one
`STEP`, then force-schedules from the entity's exact position. It deliberately bypasses the generic
listenable-tag, spectator, careful-step, dampening, current-vibration and occlusion checks. The
generic movement owner controls whether an entity reaches this callback at all.

Because force scheduling does not inspect `currentVibration`, a step can enter the selector while
an earlier vibration is traveling if the block phase is still inactive. Only one waiting candidate
survives the selector rules below.

**Candidate selection and travel:**

Normal admitted events create a candidate from the event holder, exact source position, exact
Euclidean source-to-listener distance, direct source UUID/reference and optional projectile-owner
UUID. Candidates sharing one game time compete by strictly shorter distance, then strictly higher
frequency; an exact tie retains the first. A candidate from a different game time cannot replace
the retained one.

Selection is not eligible until `candidateTick < currentGameTime`. The winner becomes current,
travel time becomes `floor(exactDistance)`, one `VIBRATION` particle is sent at the source with the
block position as destination and that travel time, the entity is dirtied, and the selector is
cleared. The same block-entity tick immediately decrements the delay with floor zero, so distances
zero and one can arrive on that selection tick.

Each later server block-entity tick attempts reload-particle recovery, decrements a positive delay,
and tries delivery at zero. Sensor users require every chunk in the listener chunk's 3×3 square to
both pass `shouldTickBlocksAt` and exist as a loaded chunk. Failure retains the current vibration at
delay zero and retries every tick. Success calls the sensor user and then clears current state;
source and projectile-owner UUIDs resolve in the current server level, with null on absence.

Arrival uses the Euclidean distance between the containing source block position and listener block
position, not the stored exact-position distance. Power is
`max(1, 15-floor(15*distance/radius))`. If the live state is no longer an inactive
`SculkSensorBlock`, the arrival produces no activation but is still consumed.

**Activation, redstone and scheduled phases:**

A successful arrival first stores the frequency, then `activate` performs:

1. flags-3 write to `active` with computed power;
2. schedule this block after its active duration;
3. update neighbors at the sensor and the block below;
4. visit all six adjacent positions for resonance;
5. emit `SCULK_SENSOR_TENDRILS_CLICKING` with the direct source entity;
6. if dry, play `SCULK_CLICKING` in `BLOCKS`, volume `1`, pitch `0.8+0.2*nextFloat`.

The ordinary active duration is `30` ticks; calibrated is `10`. Write success is ignored, so
schedule, updates, resonance, event and dry sound still run after a rejected activation write.
Water suppresses only the sensor click/stop sounds, not particles, redstone, resonance or events.

When an active scheduled tick fires, it flags-3 writes phase cooldown and power zero, schedules the
same block after `10` ticks, and updates neighbors at the sensor and below. When a cooldown tick
fires, it flags-3 writes inactive and, if dry, plays `SCULK_CLICKING_STOP` at volume `1` and pitch
`0.8+0.2*nextFloat`. Inactive scheduled ticks do nothing. These writes are also unchecked;
deactivation still schedules/notifies after failure, and the final tick can still play its sound.

Both blocks are signal sources. Ordinary weak output is `power` on every queried face. Calibrated
weak output is zero only when the query direction equals its `facing`, and `power` otherwise. Direct
output is `power` only upward and zero in the other five directions. Comparator output is the
stored last frequency only while the state is active and a sensor entity is present; otherwise it
is zero. The stored frequency is not cleared during cooldown.

Replacing another block with a sensor state runs a server placement repair: when authored power is
positive and this block has no scheduled tick, it attempts a flags-18 power-zero write without
changing phase. Replacing the same block skips this repair. Removing an active state updates
neighbors at the removed position and below; other phases do not use this removal hook.

**Resonance and local presentation:**

Activation scans the locked direction iteration order. Every adjacent
`minecraft:vibration_resonators` block—only amethyst block in bundled data—receives `resonate_N`
for the arriving frequency with the sensor's direct source and resonator state, then plays
`AMETHYST_BLOCK_RESONATE` at volume `1`. Its deterministic pitch uses note indices
`[0,2,4,6,7,9,10,12,14,15,18,19,21,22,24]` for frequencies `1..15`. Each resonance event can be
heard by other sensors through the ordinary event path.

On each client animation tick, a nonactive state consumes no sensor RNG. An active state draws one
of six directions uniformly; up/down stop immediately. A horizontal choice emits one
`SCULK_TO_REDSTONE` particle at Y `+0.25`, `0.1` outside the chosen face and uniformly across the
other horizontal axis. Its velocity is `(0, nextFloat*0.04, 0)`. The horizontal path therefore
consumes the direction draw, one double and one float.

All authoritative phase/power changes project as ordinary block-state updates using the locked
state IDs. Traveling vibration particles, dry clicks, stop clicks and resonance sounds use their
already specified particle/sound packet families. The sensor entity overrides neither update
packet nor update tag and has no special client renderer: its persisted listener/frequency data is
not sent as block-entity data.

**Waterlogging and block shape:**

Placement sets waterlogged exactly when the current fluid is water. A waterlogged state exposes a
water source fluid state and every shape update schedules water with the level's water tick delay
before delegating the ordinary shape update. Sensor phase changes retain waterlogged and facing.

**Persistence and reload:**

Full block-entity save always writes integer `last_vibration_frequency` and listener data. Listener
data contains optional current `event`, selector state and nonnegative `event_delay`; vibration
records carry `game_event`, nonnegative `distance`, exact `pos`, optional source UUID and optional
projectile-owner UUID. Missing frequency defaults to zero; absent/invalid listener data creates a
fresh empty listener.

Decoded listener data always requests a particle reload. On the first and subsequent server ticks,
the entity interpolates the particle's current point from source toward the current listener
position and sends a one-particle `VIBRATION` with the remaining delay. The request clears only
after at least one recipient accepts the particle; absent current data clears it immediately.
Block phase, power, water and facing persist separately as block state. Consequently malformed or
authored state/listener combinations are not normalized on load: an inactive block can resume an
in-flight vibration, while an active block with no scheduled tick can remain active.

Selection, every positive travel tick and successful delivery dirty the block entity through the
user callback. No subtype update packet exposes these changes to clients.

**Branches and aborts:**

Both types; all state IDs; all phases/powers/water states/facings; every tagged and untagged event;
frequency zero and 1..15; source absent/spectator/careful/dampening; affected-state damping;
same-position placement/destruction; back input 0..15; six-ray occlusion masks; ordinary/forced
step; Warden/non-Warden; empty/current selector; same/different tick, distance and frequency ties;
travel distance below/at integer boundaries; every 3×3 chunk gate; live/replaced/phase-changed
arrival; write success/failure; dry/waterlogged; six resonator directions; save/load/default and
particle-recipient outcomes; signal/comparator directions and entity presence.

**Constants and randomness:**

Radii ordinary `8`, calibrated `16`; active durations ordinary `30`, calibrated `10`; cooldown
`10`; power formula above; phase writes flags `3`, placement repair flags `18`; XP `5`; shape
height `8/16`; occlusion epsilon `9.999999747378752e-6`; click pitch `0.8+0.2*nextFloat`;
ambient rise `<0.04`; resonance note indices above. Selection and travel consume no RNG. Each dry
activation/final cooldown consumes one level float; each active client animation uses the described
client RNG branches.

**Side effects:**

Selector/current vibration and dirty state; vibration and ambient particles; advancement trigger;
flags-3/18 block writes; scheduled ticks; neighbor updates; weak/direct/comparator output; game
events and resonance chains; block and amethyst sounds; water tick scheduling; XP through the
generic break owner; full-save listener continuity.

**Gates:**

Logical side and block-entity type; event/tag/frequency/source/affected-state predicates; current
vibration; phase; calibrated back input; route occlusion; step caller and Warden exclusion; listener
position/radius; delayed 3×3 chunk activity; live block/phase at arrival; water for sound only;
particle recipient for reload completion. Difficulty and game rules do not directly gate sensing.

**Boundary cases and quirks:**

Same-tick equal candidates retain the first, but higher frequency wins only after exact distance
equality. A forced step can wait behind a traveling vibration. Calibrated input is sampled before
travel. Arrival power uses block-position distance rather than travel's exact distance. Missing
adjacent chunks stall indefinitely at delay zero. Water silences only sensor clicks. Failed state
writes do not roll back schedules, updates, resonance, events or sounds. Frequency survives
cooldown and save, but comparator output hides it outside active phase. No block-entity packet
reveals an in-flight event.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.SculkSensorBlock`,
`net.minecraft.world.level.block.CalibratedSculkSensorBlock`,
`net.minecraft.world.level.block.entity.SculkSensorBlockEntity`,
`net.minecraft.world.level.block.entity.CalibratedSculkSensorBlockEntity`,
`net.minecraft.world.level.gameevent.vibrations.VibrationSystem`,
`net.minecraft.world.level.gameevent.vibrations.VibrationSelector`,
`net.minecraft.world.level.gameevent.vibrations.VibrationInfo`; locked block/block-entity reports;
bundled vibration, ignore-sneaking, dampening, occlusion and resonator tags; `EXP-BLK-020`.

**Test vectors:**

Exhaust both blocks and the formula endpoints of all 480 states; all 56 bundled vibration-tag
entries plus absent/zero-frequency events; every source/state/subtype/occlusion gate; ordinary and
forced ingress; same-tick candidate permutations and a queued forced step; exact/block distance
boundaries; every travel and reload delay; each 3×3 chunk failure; live-state and write failures;
activation/cooldown schedules and all redstone faces; every resonance frequency/direction; dry and
waterlogged sound/RNG branches; full save/load and zero/one particle recipients. Run
`EXP-BLK-020` as the executable matrix.
