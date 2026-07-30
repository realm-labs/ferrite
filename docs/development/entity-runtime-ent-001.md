# ENT-001 Entity Runtime

`G01-P7-S001` establishes the first protocol-neutral entity runtime. The owning Region remains the
only authority that may mutate these records; packet handlers, render projection, registry import,
world generation, and later mob-Brain integration consume plans and values from this layer without
becoming alternate entity owners.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_001` is divided by state ownership:

- `lifecycle` owns UUID admission, section visibility, tracking/ticking transitions, passenger
  trees, cramming, removal, unload, and same-/cross-dimension teleport transactions;
- `drops` owns the seven differently placed live reads of `entity_drops`;
- `catalog` closes the 37 imported `minecraft:entity_type` identities, raw protocol IDs, families,
  and 35 subtype slice owners;
- `profiles` locks construction dimensions, eye height, tracking cadence, category, Peaceful
  admission, base attributes, and experience for every owned identity;
- `aquatic` owns schooling, air, size/variant packing, puffing, ink, moisture, and Tadpole age;
- `undead` owns Skeleton-family cadence, projectile effects, conversion, daylight, shearing, and
  Parched/Wither special attacks;
- `hostile` owns the Bat, Blaze, Breeze, Endermite, Ghast, cube, Spider, Phantom, Guardian, Shulker,
  and Vex local state machines;
- `raider` owns Evoker/Illusioner spells, Piglin-Brute conversion, Pillager crossbow state, and
  Vindicator raid behavior;
- `passive` owns Iron/Snow Golem, Villager, and Wandering Trader local transitions.

No module owns world insertion or packet emission. Effects are returned in source order so a later
Region integration batch can commit them inside the fixed tick pipeline.

## Lifecycle invariants

UUID membership is committed before section/callback publication. An ordinary duplicate has no
mutation; a player duplicate unrides and discards the previous player before adding the new one.
Always-ticking entities force effective `Ticking` visibility. Section movement publishes tracking
changes before ticking changes and moves the dynamic listener only for an accessible destination.

Root ticking rejects removed and frozen entities, repairs a stale vehicle link, performs despawn
admission, and recursively ticks eligible passenger trees. Cramming damage retains the server,
positive-limit, raw-count, one-in-four, filtered-count, and damage-admission gates while pushing
every raw neighbor independently.

Mounting validates the graph and admission gates before replacing an old ride. Players occupy the
first passenger position when no player already does. Ejection is reverse ordered and writes the
60-tick boarding cooldown.

The first removal reason is retained, but every call repeats cleanup using that call's reason.
Destroyed entities unride, all removals eject passengers, and UUID membership is released so a
later ordinary entity may reuse the identity. Chunk unload saves a serializable root once and then
removes its passenger tree. Same-dimension teleport places passengers before the root; a failed
cross-dimension destination creation intentionally leaves the old root alive after passenger
transfer, matching the source's no-rollback boundary.

## Live drop boundaries

`entity_drops` is not modeled as a global post-filter:

- vehicle destruction reads it after kill;
- container vehicles perform an independent read before slot-order destructive itemization and
  direct-player Piglin notification;
- paintings read it before both their local sound and item;
- item frames clear the displayed stack before the read and preserve the fixed/remover/map/RNG
  distinctions;
- invalid leashes always unlink/callback/send/notify, spawning a lead only when enabled;
- falling-block write failure may remain alive when either drop gate is false, unlike ineligible or
  timed-out entities;
- Copper Golem statue and equipment commitment precede the final leash-only read.

The outcome lists deliberately expose these ordering differences to the Region transaction rather
than collapsing them to a Boolean “may drop” answer.

## Determinism and content

Random values enter the pure transitions as named draws. The module never creates an ambient RNG or
depends on entity iteration order. The imported content bundle remains the source of runtime
registry material; `catalog::verify_entities` proves that the locked 26.2 bundle still carries all
37 exact persistent IDs, raw IDs, and assigned behavior families.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_001.rs` owns all 37 source-specified slices. It
checks closed identity/slice coverage, construction profiles, lifecycle ordering, all seven drop
sites, and exact boundary vectors across every responsibility module. Region ECS installation,
combat joins, AI scheduling, and spawning are intentionally composed by the remaining Phase 7
batches and `G01-P7-B1`; this batch does not create a second integration path ahead of those owners.
