# Minecraft 26.2 world-border runtime

Ferrite's `WGEN-BORDER-001` owner is `ferrite-world::generation::border`. The module separates
authoritative extent state, geometry, collision and ray clipping, damage and presentation formulas,
and command conversion. It returns deterministic mutations and projections; Phase 8 Region
integration owns durable application, player audiences, packet encoding, and damage admission.

## State and tick order

`state` owns the level center, absolute coordinate limit, damage and warning settings, listeners,
and either a static or moving extent. A moving extent retains source endpoints, original double
duration, signed remaining ticks, previous/current samples, and begin/end game-time metadata.
Normal updates decrement first, copy current to previous, calculate the new sample, dirty the level,
and install a static target at completion. A false normal-run gate changes nothing, covering server
freeze, overload deferral, and independently frozen client presentation.

Immediate size, lerp, center, warning, damage, safe-zone, and absolute-limit mutations dirty before
delivering to a copied listener list. Deliveries identify the five events that broadcast to the
current dimension; damage, safe-zone, and absolute-limit changes have no client packet. Save state
contains calculated size, target, and remaining ticks but deliberately excludes the transient
absolute limit. Reload only resumes a positive remaining interpolation and resets lag history.
Reconnect snapshots include the calculated old size, target, remaining ticks, absolute limit, and
warnings; applying one creates an independent client extent with the same reset-history behavior.

## Geometry and collision

`geometry` derives partial edges from `lerp(partial,previous,current)` and clamps each edge to the
absolute limit. No-argument methods use partial zero, preserving the intermediate one-sample lag.
Point, radius, block, wrapping chunk-origin, and AABB predicates retain minimum-inclusive and
maximum-exclusive comparisons, including the exact maximum-face epsilon. Distance preserves the
source `Math.min` evaluation order and vector clamp preserves Y before Java-compatible flooring.

`collision` exposes the integer-rounded complemented interior wall and admits it only for entities
near the expanded border. Border-aware ray clipping preserves an ordinary hit unchanged unless the
ray starts inside and ends outside. A replacement derives `Direction.getApproximateNearest` from
the unclamped travel vector using its float narrowing, enum order, positive-minimum seed, and strict
tie comparison, then clamps the location and marks it as a world-border hit.

## Damage, warning, and rendering

`effects` models the alive and in-wall precedence gates before player-only AABB admission. Outside
damage uses center distance plus safe zone and submits the exact floored, minimum-one float amount
with the locked `minecraft:outside_border` metadata and tags. The model returns a decision because
the owning gameplay damage pipeline may still reject the submission.

HUD warning narrows previous-edge distance to float while projecting from calculated current size.
Force-field geometry uses the frame partial tick, while alpha intentionally reads previous-edge
distance. Both retain source clamping and the fourth-power render falloff.

## Command and integration boundary

`command` validates the closed size interval and converts none/`t`, `s`, and `d` suffixes through
float multiplication followed by Java `Math.round`. Set-zero is immediate; add starts at calculated
current size and adds the existing remaining duration. Direct nonpositive lerps remain available to
preserve their locked construction and first-update quirks even though normal command parsing does
not create every such input.

The module intentionally does not own Region state or network sessions. `BorderMutation`,
`SavedBorder`, `BorderSnapshot`, collision results, damage decisions, and presentation frames are
the stable inputs to `G01-P8-B1`, which will fence level ownership, journal durable mutations, route
dimension audiences, and apply the independent server/client tick gates.
