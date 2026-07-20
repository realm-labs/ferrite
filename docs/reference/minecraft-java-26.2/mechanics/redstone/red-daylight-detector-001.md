# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `RED-DAYLIGHT-DETECTOR-001` — Daylight detectors sample effective sky light and a positional sun-angle attribute every 20 server ticks

**Parent:** `RED-001`, `BLK-003`, `BLK-007`, `PLY-006`, `ENV-003`, `WGEN-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked block/report, empty block-entity subtype, generic lighting/dimension
owners and `DaylightDetectorBlock` bytecode fix the signal formula, ticker admission, interaction
order, flags and outputs; no unresolved ordering or numeric behavior remains in this component.

**Applies when:**

A loaded daylight-detector block entity receives its server ticker in a skylit dimension, or a
build-capable player uses the block with an empty hand. Placement/state storage and downstream
neighbor propagation retain their generic owners.

**Authoritative state:**

The block state owns `inverted` and integer `power` `0..15`; the subtype block entity adds no
fields, save/update data, packet or renderer. The level supplies canonical sky-light brightness,
integer `skyDarken`, positional `visual/sun_angle` in degrees, dimension `has_skylight`, game time
and state-write outcomes. The current queried/captured state, not hidden block-entity data,
determines every result.

**Transition and ordering:**

Only the server in a dimension type with `has_skylight=true` installs this subtype ticker. On each
admitted block-entity phase it acts exactly when `gameTime mod 20 == 0`; freeze, chunk activity,
border and compatibility gates remain those of `BLK-UPDATE-001`. Let
`b = SKY_BRIGHTNESS(pos)-skyDarken` and let float `a = float(sunAngleDegrees) * float(π/180)`. If
inverted, target is `15-b` and `a` is not otherwise used. If not inverted and `b>0`, choose float
endpoint `e=0` when `a<π`, else `e=2π`; replace `a` with `a+(e-a)*0.2f`, then target is Java
`Math.round(float(b)*cos(a))`. Otherwise target remains `b`. Clamp target to `0..15`; only when it
differs from the captured state's power offer `state.with(power,target)` using flags `3`. The write
result is ignored.

Empty-hand use first checks `player.mayBuild`. False delegates to base and returns `PASS`. True
returns `SUCCESS` on both sides. The client predicts no state. The server cycles captured
`inverted`, offers that captured state with only the inverted change using flags `2`, ignores the
result, emits `BLOCK_CHANGE` with player and that new state, then immediately evaluates the formula
using that same captured new state and live level inputs. If its power differs, a second flags-`3`
offer follows even when the first offer failed or callbacks replaced the position. In a no-skylight
dimension no periodic ticker exists, but manual use still evaluates: ordinary sky brightness
normally yields zero, so inverted use can offer power 15 while noninverted use offers zero.

**Signal and shape:**

The block is a signal source. Ordinary signal on every queried face equals stored power; direct
signal remains base zero. It has a full `16×16` footprint from Y `0` through `6/16` and uses that
shape for light occlusion. Default state is noninverted power zero. No analog-output interface or
RNG participates.

**Branches and aborts:**

Client/server; player build permission; dimension skylight ticker installation; global time modulus;
inverted; effective brightness positive/zero/negative; sun angle below/equal/above π;
unchanged/changed target; first/second write rejection or callback replacement.

**Constants and randomness:**

Period `20`; endpoints `0/2π`; smoothing factor float `0.2`; degree conversion float `π/180`; power
clamp `0..15`; height `6/16`; flags `2` then optional `3`. Java float conversion, `Mth.cos(float)`
and `Math.round(float)` define exact boundaries. No RNG is consumed.

**Side effects:**

Optional inverted and power state writes, flags-selected client/neighbor/shape/light work, one game
event on permitted server use, and ordinary signal changes. The empty block entity contributes only
ticker identity/lifecycle.

**Gates:**

Compatible loaded subtype; normal block-entity ticking; server and `has_skylight` for periodic work;
time modulus; build permission for manual use; current state equality and state-write admission.
Weather is read only insofar as it has already changed `skyDarken`; difficulty and gamerules are not
read.

**Boundary cases and quirks:**

Manual inversion uses flags `2`, so when recomputed power is unchanged it emits no ordinary neighbor
update. Its event and second formula call survive a rejected first write. The second call uses the
intended inverted state rather than rereading live state. No-skylight dimensions disable only the
ticker, not manual recomputation. Normal mode smooths the angle before cosine; inverted mode ignores
angle entirely.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.level.block.DaylightDetectorBlock#updateSignalStrength`, `#useWithoutItem`,
`#getTicker`, `#tickEntity`, `#ownSignal`,
`net.minecraft.world.level.block.entity.DaylightDetectorBlockEntity`,
`net.minecraft.world.level.LevelReader#getEffectiveSkyBrightness`,
`net.minecraft.world.level.block.state.BlockBehaviour#getSignal` and `#getDirectSignal`.

**Test vectors:**

Cross skylight/no-skylight and client/server; times modulo 20 at 19/0/1; effective brightness
`0..15` plus custom negative boundary; sun angles immediately below/equal/above π and rounding
thresholds; inverted/noninverted; current power equal/different; build permission; first/second
write failure and callback replacement. Assert flags, event, formula float trace, face signals and
no BE data/packet/renderer. `EXP-RED-005` is the conformance matrix.
