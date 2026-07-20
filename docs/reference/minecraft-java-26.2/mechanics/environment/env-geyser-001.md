# Environment mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENV-GEYSER-001` — Potent sulfur derives five fluid states and runs a positional geyser clock

**Parent:** `ENV-001`, `ENV-002`, `SIM-005`, `BLK-003`, `BLK-005`, `BLK-007`, `ENT-001`, `ENT-006`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source and data fix all five states, neighbor derivation,
bounded water-column scan, deterministic countdown, block-event epoch, nausea/visibility tests,
entity-launch transaction, persistence, particles/sounds/game events and block loot. Generic effect
merge, entity movement synchronization and particle-engine internals remain under their generic leaf
owners rather than unknown here.

**Applies when:**

`minecraft:potent_sulfur` is placed or receives any shape update, its state-specific block-entity
ticker runs, its eruption block event executes, the client display tick runs, or its countdown is
saved/loaded.

**Authoritative state:**

The block property is exactly `dry`, `wet`, `dormant`, `erupting`, or `continuous`, default `dry`,
for five states. The block entity separately owns signed `waitingCountdown` default `-1` and
transient signed-long `eruptionTick` default `-1`; the latter initializes to the level game time on
first level attachment and is never persisted. Other inputs are the immediate-above source-water
test, below block/tag/fluid, up-to-five-block source scan, current game time, world seed/position,
collision shapes, entities/effects, client RNG and write/event admission. The block copies sulfur's
base-drum instrument, correct-tool requirement and strength `1.5/6`, then uses gold map color and
potent-sulfur sound.

**Transition and ordering:**

Placement and every neighbor-shape callback rederive state. If the immediate-above fluid is not
source water, result is `DRY`. Otherwise a below source lava (the sole continuous tag member;
empty/source fluid required) gives `CONTINUOUS`; a below magma block (the sole periodic member)
gives `DORMANT`, except captured `ERUPTING` remains erupting. Entering the periodic family from any
other state resets the block-entity countdown to `-1` before returning dormant; already
dormant/erupting does not reset. Other supports give `WET`. Flowing lava fails the source-fluid gate
and gives wet. Every placed transition into erupting/continuous queues block event `(0,0)`,
broadcasts the corresponding start sound at volume/pitch 1, and emits `BLOCK_ACTIVATE`; the event
sets that side's `eruptionTick` to its current game time. Entering dormant emits no placement
effect; the countdown transition additionally emits `BLOCK_DEACTIVATE` with the captured erupting
state.

**Source-column scan:**

Starting one block above sulfur, inspect through Y `origin+5`. Source-water cells continue only when
the state is water or has an empty collision shape under the sulfur-Y position context; a
source-waterlogged colliding block aborts. The first non-source-water air or passable state is the
gas/plume source. A nonair colliding state aborts, and five continuing water cells return no source.
Therefore an ordinary active column admits `0..4` water blocks, with source at the first passable
nonwater cell above them. All countdown, gas, plume and launch work aborts when this scan returns
null, without changing counters.

**Periodic countdown:**

Dormant and erupting server countdown work runs only when global game time is divisible by 20. When
`countdown<=0`, recreate a positional Xoroshiro stream from `worldSeed XOR -904011478` and block
position. Dormant sets `10*(waterBlocks-1)+inclusive(15,30)`; erupting first consumes/discards one
unbounded integer, then sets `waterBlocks-1+inclusive(1,2)`, aligning the second positional draw.
The newly initialized positive value is decremented in the same callback. At zero, dormant writes
erupting or erupting writes dormant with flags 3. Dormant's composed ticker performs this before its
ten-tick nausea ticker; erupting launches entities before its countdown. Recreating the stream makes
each cycle at one seed/position/water height repeat the same durations. Load accepts any persisted
integer; save always writes it. Reset neither dirties nor persists by itself.

**Noxious gas:**

Wet server state runs nausea; dormant runs it after countdown. On game times divisible by 10, a
valid source selects alive nonspectator living entities whose boxes intersect the source block AABB
inflated `2.5` only in X/Z. Admission then requires the eye's containing block to be
air/water/empty-collision, squared distance from source center at most `9`, source water one block
below the eye, and a collider-only/fluid-none clip from the center below the source to that
below-eye point whose result is not a block hit. Each admitted entity receives nausea duration 80,
amplifier 0, ambient true and particles/icon visible; generic same-effect merge owns how that
refresh combines. Client wet/dormant state instead emits one `NOXIOUS_GAS_CLOUD` at source center on
game times divisible by 20.

**Geyser launch:**

Erupting and continuous run launch every admitted client and server block-entity tick after finding
a source. Let `waterBlocks` be sourceY minus sulfurY minus one. Scan at most `6*waterBlocks` blocks
upward from immediately above sulfur; air, water and empty-collision states pass, and the first
obstruction returns its index. Query alive nonspectator entities in the full block above sulfur
expanded toward Y by `unobstructedCount-1` (negative one when count is zero). For every candidate,
call fall-distance accumulation before any remaining gate. Movement simulation must be enabled;
flying players, passengers and the sole `#not_affected_by_geysers` member `ender_dragon` fail.
Vertical velocity must be strictly below `0.3F+waterBlocks*0.1`; an admitted entity adds exactly
`0.2F` to Y velocity and marks synchronization needed. Repeated ticks stop adding once the current
velocity reaches/floats past the strict threshold.

**Client plume and display effects:**

Erupting/continuous clients run plume work before launch. With `eruptionTime=gameTime-eruptionTick`,
multiples of 20 create one `GEYSER` controller particle at source X/Z center and exact source Y
carrying `waterBlocks`; multiples of 40 also play the corresponding active sound locally at source
center, volume/pitch 1. A block event resets the cadence epoch; chunk attachment without one uses
attachment game time. Independently, client `animateTick` for every non-dry state first requires
immediate-above source water, then always consumes six floats for two always-visible sulfur-bubble
positions inside the block above and one `nextInt(10)`; zero plays local noxious-gas ambience at the
sulfur integer corner. This display path does not require the bounded source scan.

**Branches and aborts:**

No immediate source water; continuous/periodic below fluid non-source; periodic re-entry versus
retained eruption; missing/mismatched block entity on reset/event; source scan
obstruction/five-water overflow; global modulo miss; nonpositive countdown initialization; failed
state write; effect AABB/eye/passability/range/water/clip denial; zero-height or obstructed launch
box; unsimulated/flying/passenger/tagged/high-velocity entity; client/server ticker split. A failed
transition write does not undo the already-mutated countdown.

**Constants and randomness:**

Above-water limit 4/source probe 5; nausea frequency/duration/range `10/80/3`; client cloud/plume
frequency 20; active sound frequency 40; launch base/force `0.3F/0.2F`, height multiplier 6;
deterministic salt `-904011478`; dormant random 15..30 plus ten per extra water level, eruption
random 1..2 plus water height; display sound chance 1/10. Server countdown randomness is isolated
positional state; display randomness is client level RNG.

**Side effects:**

Five-state flags-3 writes and neighbor/client consequences; block events; start/active sounds;
activate/deactivate game events; persisted countdown; nausea effects; entity fall
bookkeeping/velocity/sync; sulfur bubbles, gas-cloud and geyser controller particles. Block loot
yields one potent sulfur item only when the generic survives-explosion condition passes and copies
no countdown.

**Gates:**

Loaded block-entity ticking, exact state-specific ticker, source water and support tags/fluid,
collision/source scan, global modulo, deterministic countdown, alive/nonspectator and
movement/effect gates, client display admission, state/event/effect/entity synchronization.

**Boundary cases and quirks:**

Exactly five source-water cells suppress all gas/geyser work even while state remains active.
Countdown does not advance while the source scan is blocked. An erupting zero-water stale state can
initialize and return dormant in one 20-tick callback after launching from a downward-expanded box.
Fall-distance accumulation happens even for later-rejected candidates. Client launch runs alongside
server launch, while the block event—not persisted state—anchors visible plume cadence. Wet/dormant
ambient display can continue with a passable-above source layout that fails the deeper source scan.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.PotentSulfurBlock#getStateForPlacement`,
`net.minecraft.world.level.block.PotentSulfurBlock#updateShape`,
`net.minecraft.world.level.block.PotentSulfurBlock#onPlace`,
`net.minecraft.world.level.block.PotentSulfurBlock#animateTick`,
`net.minecraft.world.level.block.PotentSulfurBlock#triggerEvent`,
`net.minecraft.world.level.block.PotentSulfurBlock#getTicker`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#findNoxiousGasSourceBlock`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#canBeReachedByNoxiousGas`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#getUnobstructedBlockCount`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#geyserPositional`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.PotentSulfurBlockEntity#setLevel`; locked block/entity/tag
reports and `data/minecraft/loot_table/blocks/potent_sulfur.json`; `EXP-ENV-005`.

**Test vectors:**

Every five-state/above-water/below dry, source/flowing lava, magma and ordinary support transition;
0..5 water cells with air, passable, colliding and waterlogged endpoints; countdown `-1/0/1`, all
positional endpoints and failed writes; game-time residues for 10/20/40; effect AABB/range equality,
eye cell, below-eye source and clip hit; launch height obstruction 0/max, every rejection gate and
velocity equality; client attachment/block-event epochs; exact display RNG; missing/wrong block
entity; save/load arbitrary countdown; explosion-admitted/rejected loot.
