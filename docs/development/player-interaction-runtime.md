# Player Target Interaction Runtime

`G01-P6-S015` implements `PLY-TARGET-INTERACTION-001` as protocol-neutral selection and transaction
plans. Concrete block, entity, and item callbacks remain with their catalog owners; this runtime
fixes which callback is reached, its ordering, and how its result changes the outer operation.

## Runtime boundary

`ferrite-gameplay::player::interaction` is split by responsibility:

- `targeting` selects the nearest entity only when strictly closer than the block hit, keeps exact
  ties as blocks, applies target-specific strict ranges, and runs the active item's custom
  attack-range attempt before ordinary fallback;
- `attack` owns client aborts, spectator/piercing/ordinary block/entity/miss paths, main-hand swing
  and instant-break decisions, plus independent server current-target, border, `+3` range,
  invalid-target, feature, and item-attack admission;
- `use_action` owns main/offhand order, right-click suppression/delay, entity-to-air fallthrough,
  block callback/empty-hand/use-on order, air prediction, interaction-result algebra, swing and
  criterion selection, cumulative prediction sequences, server entity/block/air admission, and
  hand-stack convergence.

`InteractionResult::Success` is the only consuming result. `Fail`, `Pass`, and
`TryEmptyHandInteraction` remain distinct; the empty-hand marker has special meaning only after the
main-hand block item callback. `ItemContext` carries item participation and an optional transformed
stack without forcing unrelated callbacks to emulate item behavior.

## Selection and dispatch order

Ordinary pick searches with the larger configured reach, shortens entity selection to the exact
block hit, and replaces the block only for a strictly nearer entity. Final block/entity results
must also be strictly inside their own range. A custom attack-range block outside ordinary block
reach falls back to ordinary pick rather than widening block interaction.

Client attack uses main hand. The early delay, null hit, busy hands, disabled feature, and
`cannotAttackWithItem(...,0)` gates precede spectator and piercing paths. An admitted custom-range
entity rejection still swings. Block-air becomes the miss path; miss delay is exactly ten when the
mode owns miss time.

Client use loads delay four, then visits main and off hand. A disabled held stack, border failure,
entity/block success, or final block failure terminates the entire operation at its source-defined
point. Entity non-success continues to air use for the same hand. Block use always starts a
prediction and sends before local callbacks; air use sends even while cooldown makes prediction
pass. Secondary use with either hand nonempty bypasses the block and empty-hand callbacks but can
still reach held-item `useOn`.

## Server authority and stack convergence

Attack and entity-use admission use strict bounding-box range plus three. Block use cumulatively
acknowledges sequence first, then applies client-loaded, feature, strict range plus one, strict
`1.0000001` hit components, height, protection, teleport, and `mayInteract` gates. Every processed
height/protection/invoke branch sends target then hit-face-neighbor updates; geometric rejection
does not invoke callbacks.

Entity callbacks install packet secondary-action state before invocation. Server swing effects and
criteria are derived only from consuming results and their item context. Infinite-material entity
count restoration requires the identical stack object to remain held with a lower count.

Air-use convergence compares object identity, count, damage, use duration, and active-use state.
Transformed empty stacks clear the hand; other transformed stacks replace it. Full inventory
resynchronization occurs only on the source-owned changed/nonactive branch.

## Region ownership and determinism

The connection adapter acknowledges wire prediction sequences and converts admitted requests into
Region commands. The authoritative Region owns current targets, protection/permission checks,
callback state, inventory mutation, and committed correction. Plans contain ordered stable IDs and
effects only; callback-owned RNG remains in the callback's Region transaction.

## Validation

`crates/ferrite-gameplay/tests/slices/player/ply_004.rs` verifies strict selection, custom range,
spectator/piercing/miss/block attacks, invalid server targets, both-hand entity and block
fallthrough, feature/border termination, empty-hand marker behavior, prediction ACK order,
server-range boundaries, swing/criterion effects, infinite count restoration, and transformed air
stack resynchronization.
