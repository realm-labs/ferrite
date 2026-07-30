# ITM-007 Progression Runtime

`G01-P6-S008` implements the audited hunger, experience, and advancement rules owned by
`ITM-PROGRESSION-001`.

## Runtime boundary

`item::runtime::progression` is split by state owner:

- `hunger` owns food and saturation bounds, exhaustion spending, natural regeneration,
  starvation, and the shared branch timer;
- `experience` owns piecewise level costs, positive and negative point normalization, direct
  level changes, level-up sound gating, enchantment seed refresh, and death rewards;
- `advancement` owns requirement matrices, timestamped criterion progress, trigger listeners,
  completion transitions, rewards, visibility updates, selected tabs, packets, and persisted
  progress.

The modules expose codec-neutral state transitions. Protocol packet encoding, registry decoding,
loot generation, inventory insertion, command execution, player damage, and sound publication
remain with their existing owners. Advancement reward delivery receives those effects as
callbacks and records their required order.

## Exact transition rules

Hunger spends one exhaustion quantum only when exhaustion is strictly greater than four.
Saturation is spent before food. Saturated regeneration runs every 10 eligible ticks and slow
regeneration every 80; starvation uses the same timer and applies the Easy, Normal, and Hard
health floors.

Experience costs use the three vanilla level ranges. Point changes preserve score and total
side effects while normalizing against the destination level's cost. Level-up sounds require a
positive change into a multiple of five and a strictly greater than 100-tick interval.

Advancement requirements are ANDs of groups whose members are OR alternatives. Awards and
revocations update listeners immediately, while rewards and announcements occur only on the
incomplete-to-complete edge. Flushes calculate visibility before filtering dirty progress,
consume the first-packet reset flag once, and omit empty packets.

## Determinism and ownership

Mutable progression state belongs to the authoritative player owner. A transfer must move the
complete `FoodData`, `ExperienceData`, and `AdvancementTracker` state with the player's
generation-fenced snapshot; none of these transitions consult process identity.

Advancement definitions and rewards retain registry order. Reward loot and pickup pitches use a
caller-owned `GameplayRandom`, so the Region owner can bind the appropriate persisted named
stream. Saved advancement criteria retain their obtained timestamps; unknown saved definitions
are reported rather than admitted.

## Validation

`crates/ferrite-gameplay/tests/slices/items/itm_007.rs` verifies hunger bounds and thresholds,
regeneration and starvation timers, all experience cost breakpoints, bidirectional level
normalization, sound and seed side effects, advancement requirement matrices, listener
transitions, idempotence, load/save behavior, visibility and tab packets, and exact reward event
ordering.
