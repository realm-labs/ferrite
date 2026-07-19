# Redstone leaf rules

## Leaf rule `RED-EXPLOSION-001` — Explosion calculation, entity effects, block effects, and fire are separate phases

**Parent:** `RED-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — sampling vectors, traversal order, drop merging, and exposure-point arithmetic remain unexpanded.  <br>
**Applies when:** Gameplay creates an explosion with a level, source, center, radius, damage source, block-interaction mode, and optional fire flag.  
**Authoritative state:** Explosion parameters, source/owner, affected-block candidate set/order, entities and exposure, block interaction mode, gamerules, loot contexts, fire flag and RNG.  
**Transition and ordering:** Construct the explosion; sample outward rays through block/fluid resistance to collect unique affected positions; find entities in the radius AABB and for each eligible entity derive normalized distance and line-of-sight exposure, then apply damage and knockback; if block interaction is enabled, randomize/process affected positions, invoke block explosion hooks and drop merging through explosion loot context; finally attempt fire placement only at eligible affected air positions. Anchor: `net.minecraft.world.level.ServerExplosion#explode()` and `net.minecraft.world.level.ServerExplosion#interactWithBlocks(java.util.List)`.  
**Branches and aborts:** Radius/noninteraction produces no affected blocks; source immunity; entity outside normalized radius or zero exposure; block/fluid resistance exhausts ray power; block interaction mode keeps/destroys/triggers; drops disabled; fire false or support/roll fails. Entity effects and block effects must not be skipped merely because the other phase has no targets.  
**Constants and randomness:** Radius and damage/knockback calculations use float/double source arithmetic; ray grid, resistance attenuation, affected-list shuffle, drop survival/merging and fire placement consume explosion RNG in their phase order. Exact numeric and RNG sequence are owned by `EXP-RED-004`.  
**Side effects:** Entity damage/knockback/velocity notification, block callbacks/removal/transformation, item drops, fire states, game events, sounds, particles and source-specific criteria.  
**Gates:** Block interaction mode, `mobGriefing` or explosion-decay gamerules selected by caller, damage immunity/tags, exposure/collision, block resistance/hooks, drops/fire flags and chunk writability.  
**Boundary cases and quirks:** Affected block collection is ray sampled, not all blocks inside a sphere. Exposure uses collision geometry. Multiple destroyed stacks may merge with an explosion-specific cap/order. Optional fire is post-destruction and therefore tests the resulting world.  
**Evidence:** `Confirmed` phase structure; numeric/RNG parity `Implementation blocker`; `OFF-SERVER-001`; locators above; `EXP-RED-004`.  
**Test vectors:** Radius zero; entity with 0/partial/full exposure; high-resistance fluid/block; each interaction mode/gamerule; overlapping drops; fire enabled with valid/invalid support; fixed RNG trace of ray, shuffle, drops and fire.

## Leaf rule `RED-UPDATE-001` — Power changes propagate through component callbacks, not a global circuit solve

**Parent:** `RED-001`, `RED-002`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — dust direction ordering and component-specific signal dispatch remain unexpanded.  <br>
**Applies when:** A source, conductor, wire, or component changes a state that can alter redstone signal.  
**Authoritative state:** Installed block states, directional weak/direct signal functions, conductor status, component internal state, scheduled ticks, neighbor-update orientation and block-event queue.  
**Transition and ordering:** Commit the initiating state; notify the defined neighbors; each receiver recomputes only through its own callback and may immediately write state or schedule a delayed tick; secondary writes recursively notify according to their flags. Query signal directionally at the instant of each callback. Vanilla does not first solve a stable graph and atomically commit it. Anchors: `net.minecraft.world.level.Level#updateNeighborsAt(net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation)`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getSignal(net.minecraft.world.level.BlockGetter,net.minecraft.core.BlockPos,net.minecraft.core.Direction)`, and `net.minecraft.world.level.redstone.ExperimentalRedstoneUtils#initialOrientation(net.minecraft.world.level.Level,net.minecraft.core.Direction,net.minecraft.core.Direction)`.  
**Branches and aborts:** Non-signal source; face not powered; conductor relays direct signal; receiver already in desired state; scheduled transition already pending; update budget or component lock suppresses a state change. Experimental redstone data pack behavior is outside default 26.2 unless explicitly enabled.  
**Constants and randomness:** Signal strength is integer 0–15. Generic propagation uses no RNG; direction/orientation and callback stack define order. Component delays are integer ticks and live in their rules.  
**Side effects:** State writes, further neighbor and comparator updates, scheduled ticks, piston/block events, block-entity mutations, sounds, particles and client block-state updates.  
**Gates:** Chunk availability/ticking for queued work, component direction, conductor rules, update flags, feature/data-pack selection and component-specific lock/power predicates.  
**Boundary cases and quirks:** Quasi-connectivity-like behavior arises from the receiver's checked positions and update paths; do not introduce a generic distance rule. Transient intermediate signals can be observable and power other components.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; exact direction trace `EXP-RED-001`.  
**Test vectors:** Branching wire with two receivers; power through a conductor from each face; state change that is reverted within one server tick; compare default and redstone-experiments-disabled worlds.

## Leaf rule `RED-DAYLIGHT-DETECTOR-001` — Daylight detectors sample effective sky light and a positional sun-angle attribute every 20 server ticks

**Parent:** `RED-001`, `BLK-003`, `BLK-007`, `PLY-006`, `ENV-003`, `WGEN-004`  <br>
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceSpecified` — the locked block/report, empty block-entity subtype, generic lighting/dimension owners and `DaylightDetectorBlock` bytecode fix the signal formula, ticker admission, interaction order, flags and outputs; no unresolved ordering or numeric behavior remains in this component.  <br>
**Applies when:** A loaded daylight-detector block entity receives its server ticker in a skylit dimension, or a build-capable player uses the block with an empty hand. Placement/state storage and downstream neighbor propagation retain their generic owners.  <br>
**Authoritative state:** The block state owns `inverted` and integer `power` `0..15`; the subtype block entity adds no fields, save/update data, packet or renderer. The level supplies canonical sky-light brightness, integer `skyDarken`, positional `visual/sun_angle` in degrees, dimension `has_skylight`, game time and state-write outcomes. The current queried/captured state, not hidden block-entity data, determines every result.  <br>
**Transition and ordering:** Only the server in a dimension type with `has_skylight=true` installs this subtype ticker. On each admitted block-entity phase it acts exactly when `gameTime mod 20 == 0`; freeze, chunk activity, border and compatibility gates remain those of `BLK-UPDATE-001`. Let `b = SKY_BRIGHTNESS(pos)-skyDarken` and let float `a = float(sunAngleDegrees) * float(π/180)`. If inverted, target is `15-b` and `a` is not otherwise used. If not inverted and `b>0`, choose float endpoint `e=0` when `a<π`, else `e=2π`; replace `a` with `a+(e-a)*0.2f`, then target is Java `Math.round(float(b)*cos(a))`. Otherwise target remains `b`. Clamp target to `0..15`; only when it differs from the captured state's power offer `state.with(power,target)` using flags `3`. The write result is ignored.  <br>

Empty-hand use first checks `player.mayBuild`. False delegates to base and returns `PASS`. True returns `SUCCESS` on both sides. The client predicts no state. The server cycles captured `inverted`, offers that captured state with only the inverted change using flags `2`, ignores the result, emits `BLOCK_CHANGE` with player and that new state, then immediately evaluates the formula using that same captured new state and live level inputs. If its power differs, a second flags-`3` offer follows even when the first offer failed or callbacks replaced the position. In a no-skylight dimension no periodic ticker exists, but manual use still evaluates: ordinary sky brightness normally yields zero, so inverted use can offer power 15 while noninverted use offers zero.  <br>
**Signal and shape:** The block is a signal source. Ordinary signal on every queried face equals stored power; direct signal remains base zero. It has a full `16×16` footprint from Y `0` through `6/16` and uses that shape for light occlusion. Default state is noninverted power zero. No analog-output interface or RNG participates.  <br>
**Branches and aborts:** Client/server; player build permission; dimension skylight ticker installation; global time modulus; inverted; effective brightness positive/zero/negative; sun angle below/equal/above π; unchanged/changed target; first/second write rejection or callback replacement.  <br>
**Constants and randomness:** Period `20`; endpoints `0/2π`; smoothing factor float `0.2`; degree conversion float `π/180`; power clamp `0..15`; height `6/16`; flags `2` then optional `3`. Java float conversion, `Mth.cos(float)` and `Math.round(float)` define exact boundaries. No RNG is consumed.  <br>
**Side effects:** Optional inverted and power state writes, flags-selected client/neighbor/shape/light work, one game event on permitted server use, and ordinary signal changes. The empty block entity contributes only ticker identity/lifecycle.  <br>
**Gates:** Compatible loaded subtype; normal block-entity ticking; server and `has_skylight` for periodic work; time modulus; build permission for manual use; current state equality and state-write admission. Weather is read only insofar as it has already changed `skyDarken`; difficulty and gamerules are not read.  <br>
**Boundary cases and quirks:** Manual inversion uses flags `2`, so when recomputed power is unchanged it emits no ordinary neighbor update. Its event and second formula call survive a rejected first write. The second call uses the intended inverted state rather than rereading live state. No-skylight dimensions disable only the ticker, not manual recomputation. Normal mode smooths the angle before cosine; inverted mode ignores angle entirely.  <br>
**Evidence:** `Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors: `net.minecraft.world.level.block.DaylightDetectorBlock#updateSignalStrength`, `#useWithoutItem`, `#getTicker`, `#tickEntity`, `#ownSignal`, `net.minecraft.world.level.block.entity.DaylightDetectorBlockEntity`, `net.minecraft.world.level.LevelReader#getEffectiveSkyBrightness`, `net.minecraft.world.level.block.state.BlockBehaviour#getSignal` and `#getDirectSignal`.  <br>
**Test vectors:** Cross skylight/no-skylight and client/server; times modulo 20 at 19/0/1; effective brightness `0..15` plus custom negative boundary; sun angles immediately below/equal/above π and rounding thresholds; inverted/noninverted; current power equal/different; build permission; first/second write failure and callback replacement. Assert flags, event, formula float trace, face signals and no BE data/packet/renderer. `EXP-RED-005` is the conformance matrix.

## Leaf rule `RED-DELAY-001` — Repeaters, comparators, observers, and torches schedule component-owned transitions

**Parent:** `RED-003`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — delay values, priority collisions, comparator modes, observer pulses, and torch burnout need separate slices.  <br>
**Applies when:** A delayed redstone component receives a relevant neighbor/input change.  
**Authoritative state:** Facing, powered/output/mode/delay/locked state, pending scheduled tick, input signals and component block-entity data.  
**Transition and ordering:** Neighbor callback samples the component-specific inputs; if desired output differs, enqueue a tick with component delay and priority; on execution resample inputs, apply lock/pulse/burnout/mode rules, commit state/output and notify defined outputs. Comparator calculation reads rear and side signals plus container analog output; observer emits a fixed pulse after detecting its watched-side state change. Anchors include `net.minecraft.world.level.block.RepeaterBlock`, `net.minecraft.world.level.block.ComparatorBlock`, `net.minecraft.world.level.block.ObserverBlock`, and `net.minecraft.world.level.block.RedstoneTorchBlock`.  
**Branches and aborts:** Repeater locked; stale scheduled transition; comparator subtract/compare result unchanged; observer pulse already active; torch powered from attachment or burnout. Each branch may deliberately retain a scheduled tick even if a later input changes.  
**Constants and randomness:** Signal is clamped to 0–15. Repeater player delay settings correspond to integer game-tick delays; observer pulse and comparator/repeater scheduling use source constants and tick priorities. No RNG selects output. Exact simultaneous-input waveform is `EXP-RED-002`.  
**Side effects:** Powered/output state, comparator block entity output, new scheduled ticks, neighbor/comparator notifications, click/torch sounds and particles.  
**Gates:** Facing and side-input geometry, lock predicate, chunk schedule eligibility, freeze, container analog support and experimental mode.  
**Boundary cases and quirks:** A scheduled callback must resample rather than blindly applying the state desired when queued. Pulses shorter than delay can be filtered or transformed depending on component and priority ordering.  
**Evidence:** `Confirmed` state-machine split; exact collision waveforms `Cross-checked`; `OFF-SERVER-001`; listed classes; `EXP-RED-002`.  
**Test vectors:** Repeater input pulse shorter/equal/longer than delay; lock before due tick; simultaneous side/rear comparator changes; observer watches two same-tick changes; torch rapid-toggle burnout sequence.

## Leaf rule `RED-PISTON-001` — A piston resolves a finite move plan before executing its block event

**Parent:** `RED-004`, `RED-005`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — movement/destruction ordering, sticky branches, and quasi-connectivity notification cases remain unexpanded.  <br>
**Applies when:** A piston observes a power condition that differs from its extension state.  
**Authoritative state:** Piston facing/extended state, checked power positions, queued block events, resolved push/destroy lists, block mobility reactions and moving-piston block entities.  
**Transition and ordering:** Neighbor change evaluates piston power; enqueue extend or retract block event; when the event runs, revalidate power, build the directional move plan, reject if immovable/limit/bounds fail, otherwise replace destinations/origins with moving states in the required reverse order, create moving block entities, update piston state and notify affected positions. Sticky retraction conditionally pulls the front block. Anchors: `net.minecraft.world.level.block.piston.PistonBaseBlock#triggerEvent(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,int,int)` and `net.minecraft.world.level.block.piston.PistonStructureResolver#resolve()`.  
**Branches and aborts:** Stale event after power reversal; already correct state; push limit exceeded; immovable/destroy reaction; build bounds; sticky target cannot be pulled; competing moving piston; plan resolution false. A failed extension does not partially move the resolved prefix.  
**Constants and randomness:** Maximum push chain is 12 blocks. Movement progress is deterministic per tick; generic resolution consumes no RNG. Mobility reaction and block-entity eligibility are block-state behavior.  
**Side effects:** Block event queue, moving-piston entities, temporary moving block states, entity displacement, destroyed blocks/drops, neighbor/shape/comparator updates, sounds and particles.  
**Gates:** Power geometry, facing, event revalidation, push reaction, world bounds, chunk availability, block-entity/moving restrictions and sticky semantics.  
**Boundary cases and quirks:** The move plan order is observable through updates and entity collision. Retraction can encounter the head/moving state from a previous transition. Power is not a single adjacent-face query.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; complex simultaneous piston trace `EXP-RED-003`.  
**Test vectors:** Push 12 and 13 blocks; include destroyable and immovable states; reverse power before the event; sticky pull versus non-pull; crossed pistons and entities in swept volume.
