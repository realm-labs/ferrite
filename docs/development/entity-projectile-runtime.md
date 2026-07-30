# ENT-004 Projectile Runtime

`G01-P7-S003` implements the protocol-neutral `ENT-PROJECTILES-001` transition layer. The owning
Region supplies authoritative candidates, rule and tag snapshots, and named random draws, then
commits returned effects in order. These modules do not query a world, create ambient randomness,
emit packets, or own ECS records.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_004` is divided by source family:

- `geometry` owns launch normalization and triangular deviation, source-motion inheritance,
  first-tick/spawn ordering, owner-vehicle departure, collision margins and strict nearest-hit
  selection, repeated-deflector rejection, border bounce, and entity/block callback order;
- `block` owns `impact_projectiles` plus `projectiles_can_break_blocks` admission and the exact
  chorus-flower, decorated-pot, and pointed-dripstone mutations;
- `throwable` owns gravity-before-inertia motion and snowball, egg, experience-bottle, ender-pearl,
  water-potion, splash-potion, and lingering-potion outcomes;
- `arrow` owns in-block-first tick ordering, ground release/despawn, water/air flight, stable
  position-distance sorting, piercing limits, damage and critical rolls, failed-hit reversal,
  block embedding, potion/spectral timers, and Trident loyalty;
- `hurting` owns acceleration/inertia ordering, owner/chunk removal, deflection, fireballs, Wither
  skulls, Dragon fireball clouds, and player/Breeze Wind Charge distinctions;
- `special` owns Firework Rocket, Llama Spit, Shulker Bullet, Fishing Hook, Eye of Ender, and Evoker
  Fang timers, range gates, motion, damage, and terminal transitions.

The local vector, candidate, and result records are transition vocabulary. They deliberately avoid
protocol, registry, Lattice, and world-storage types.

## Observable ordering and boundaries

Launch consumes exactly three triangular draws represented by six explicit uniform samples.
Spawning orders shoot, authoritative insertion, then projectile-spawn enchantment hooks. Ordinary
sweeps keep the clipped block endpoint as their upper bound and replace a candidate only at a
strictly smaller squared distance, preserving iteration order on ties. A redirectable target is
deflected before the projectile subtype callback; block callbacks precede the land event.

Block breaking requires server authority, `mayInteract`, live tag membership, and the live game
rule. Pointed dripstone additionally requires a thrown Trident with speed strictly greater than
`0.6`; decorated pots expose the `cracked=true` write with flags `260` before destruction.

Throwable motion applies gravity and then inertia before sweep and hit resolution. Egg and pearl
outcomes expose draw consumption separately from later placement, spawn-rule, difficulty, and
owner gates. Splash duration retains the strict `>20` result gate, while water potion distance is
strictly squared-distance `<16`.

Arrows retain their different tick order, stable explicit sort, `pierce + 1` cap, strict stop/drop
threshold, 1,200-tick despawn, 600-tick potion-content loss, and five-tick Trident dealt boundary.
Hurting-projectile and special-family helpers similarly expose all timers, radii, difficulty
branches, line-of-sight gates, and terminal removal decisions without applying side effects.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_004.rs` owns the source-specified projectile
slice. Its sixteen tests cover public launch and sweep rules, all three block callbacks, every
listed throwable and arrow family, fireball/skull/wind-charge behavior, and the six remaining
catalog families. `G01-P7-B1` remains responsible for installing these transitions into Region
entity ticking, combat/effect joins, persistence, and client projection.
