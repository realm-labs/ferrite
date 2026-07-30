# ENT-005 Damage Runtime

`G01-P7-S004` implements the four protocol-neutral ENT-005 slices: damage admission, item
blocking, damage reduction, and knockback. The owning Region snapshots source/entity/equipment
state and named random draws, composes these pure transitions in the audited order, and commits
their returned effects. The runtime does not mutate ECS/world state or emit packets directly.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_005` follows the reference transaction boundaries:

- `admission` owns base/living/player/server-player immunity, PvP and difficulty wrappers, retained
  pre-abort side effects, blocking/freeze/helmet transforms, nonfinite sanitization, strict
  cooldown selection, attribution, fresh-hit callbacks, criteria/stat facts, and timer decay;
- `blocking` owns use admission and maturity, JVM float admission, incidence geometry, ordered
  component reductions, requested durability, default/Hoglin/Ravager retaliation, exact active
  weapon and Warden disable selection, cooldown/stop-use order, and sound pitch draws;
- `reduction` owns armor slot/durability selection, armor and ordered Breach arithmetic,
  Resistance/protection/witch processing, absorption/health/exhaustion/stats/combat facts, Wolf
  Armor interception/cracks, and Camel/Animal/Armadillo/Copper Golem hooks;
- `knockback` owns projectile/source direction selection, five- versus six-argument gates, resistance
  and coincident-direction retries, grounded/airborne velocity, player indication, all twelve
  sulfur-cube settings, last-match selection, and its float angle/power transform.

No broad facade re-exports these owners. Callers import the responsibility that supplies their
transaction step.

## Damage transaction

Wrapper admission records side effects only after their source position in the call chain. A player
may reset action time before a later living-immunity abort; wake-up occurs only after living
immunity/death/fire-resistance gates. Player exact zero aborts after difficulty scaling, while a
nonplayer negative value becomes positive zero and can still open a fresh cooldown.

Blocking resolves before freeze and helmet transforms. NaN uses explicit partial-order branches so
it follows JVM comparison behavior rather than an ordinary Rust positivity test. Ordered damage
reductions all see the submitted amount, their float sum is capped once, and the durability helper
runs even for a geometric zero reduction. Retaliation uses full submitted damage and precedes a
possible player disable; the later outer block sound remains outside this module.

Cooldown time is strict `>10`. A weaker/equal repeat rejects after earlier wake/item effects; a
stronger repeat reduces only the excess but retains the full remaining amount for effects,
knockback and criteria. Fresh hits alone reset `20/10/10` timers and run event, impact, knockback
and hurt-sound branches. A fully blocked fresh hit can still attribute, retaliate, knock back and
emit criteria while returning false.

## Reduction and velocity

Armor durability uses the selected cooldown amount before every defense formula. Player slots use
feet/legs/chest/head order; horses and wolves select body; ordinary living armor does not wear.
Armor/Breach, Resistance, protection, witch, absorption and health retain source float ordering,
including nonplayer's second absorption write and the overflow-to-NaN edge.

Wolf Armor consumes the full selected amount without overflow. Crack thresholds are strictly
`<0.32`, `<0.69`, and `<0.95`. Camel and Animal pre-hooks precede common defense; Armadillo and
Copper Golem hooks follow it.

Common knockback computes widened `0.4f * (1-resistance)`, consumes four draws per coincident
direction retry, halves horizontal velocity, and caps only grounded vertical velocity. Exhausted
scripted draws return an explicit incomplete result and cannot be committed. Sulfur cubes retain
their separate float geometry, power envelope, unconditional dirty flag and hit sound even at
resistance one. Player damage indication remains independent of whether velocity changed.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_005.rs` owns all four source-specified slices.
Its sixteen tests cover wrapper abort positions, zero/NaN/infinity handling, cooldown thresholds,
full blocking, shield timing/angle/durability, subtype retaliation, every reduction stage,
absorption overflow, combat expiry, subtype hooks, source direction, RNG retry cardinality,
five-argument gates, player indication, and sulfur-cube settings. `G01-P7-B1` remains responsible
for Region composition with ENT-006 effects and ENT-007 death protection.
