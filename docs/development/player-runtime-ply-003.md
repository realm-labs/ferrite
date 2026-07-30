# PLY-003 Special Player Movement Runtime

`G01-P6-S014` implements `PLY-MOVEMENT-SPECIAL-001` without merging special modes into ordinary
ground/air travel. The dispatcher, fluid dynamics, swimming steering, fall flight, glider
maintenance, and ability-flight wrapper remain explicit transitions over the shared collision
runtime.

## Runtime boundary

`ferrite-gameplay::player::special_travel` owns:

- water-before-lava dispatcher selection with affected/standable-fluid gates and fall-flight
  fallback;
- Water slowdown, movement-efficiency interpolation, Dolphin's Grace, sprint gravity suppression,
  climbable and fluid-exit impulses, and ridden floating;
- shallow/deep Lava damping, the shallow sixteenth-gravity adjustment, unconditional
  quarter-gravity step, and shared fluid-exit test;
- nonpassenger swimming look-vector steering with the strict `-0.2` multiplier boundary and
  upward surface gate;
- source-ordered fall-flight lift, dive conversion, pitch lift, horizontal steering, drag,
  collision, positive wall-damage threshold, validity, and 10/20-tick maintenance;
- ordered multi-slot glider damage-before-event effects using an explicitly supplied bounded
  durability choice;
- the ability-flight wrapper's pre-travel vertical control, sprint-dependent airborne speed,
  ordinary-or-fluid superclass dispatch, and post-travel entry-Y-times-`0.6` overwrite.

`player::travel` now exposes the shared relative-input transform and an explicit airborne
acceleration override. Ordinary ground/air behavior remains unchanged when that override is absent.
Generic shape clipping remains owned by `player::collision`.

## Ordering and boundaries

Water halves `water_movement_efficiency` while airborne before interpolating slowdown toward
`0.54600006f` and acceleration toward current movement speed. Dolphin's Grace replaces the resulting
slowdown with `0.96f`. Nonsprinting, nonzero-gravity fluid adjustment occurs after Water damping;
sprinting Water receives no replacement gravity.

Shallow Lava applies `(0.5,0.8,0.5)`, conditionally applies the same sixteenth-gravity adjustment,
and then always applies gravity divided by four. Deep Lava applies all-axis `0.5` and only the
quarter-gravity step. Equality with the fluid-jump threshold selects the shallow branch.

Fall flight applies all four velocity stages before `(0.99,0.98,0.99)` drag and collision. A
climbable runs one ordinary-air tick and clears fall flight. Server validity requires a usable
glider in any matching equipment slot whose next durability point would not break it. At tick
multiples of 20, the selected glider damage effect precedes the shared 10-tick glide event.

Ability flight wraps the selected superclass path. It retains the superclass position and
horizontal velocity but discards its final vertical velocity, restoring the wrapper-entry Y
multiplied by `0.6`.

## Region ownership and determinism

The authoritative Region supplies medium state, ordered collision geometry, player attributes,
equipment slots, and the bounded glider-choice draw. The runtime consumes no ambient RNG. Glider
selection is the only random branch and its chosen valid-slot index is explicit, so replay and
local/in-process/multi-process execution observe the same effect order.

## Validation

`crates/ferrite-gameplay/tests/slices/player/ply_003.rs` verifies dispatcher priority, Water
efficiency/Grace/sprint branches, shallow/deep and sprinting Lava gravity, swimming steering,
fall-flight collision damage and validity, 10/20-tick glider ordering, climb termination, and both
ordinary and fluid ability-flight wrappers.
