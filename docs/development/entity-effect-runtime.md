# ENT-006 Effect Runtime

`G01-P7-S005` implements the protocol-neutral `ENT-EFFECTS-001` transition layer. The owning Region
provides active-map state, entity observations and explicit random draws, then commits returned
callbacks, attribute changes, spawns, blocks or explosions. The runtime never owns a parallel
effect map or ambient RNG.

## Responsibility split

`ferrite-gameplay::entity::runtime::ent_006` has three owners:

- `instance` owns amplifier construction, finite/infinite duration ordering, recursive hidden
  chains, merge flags, ordinary add callbacks, forced replacement and remove-all ordering;
- `ticking` owns no-duration removal, callback cadence/result, recursive hidden decrement and
  promotion, 600-tick update cadence, attribute-refresh facts, ordinary removal and concurrent-map
  abort accounting;
- `special` owns regeneration/poison/wither cadence, Hunger/Saturation/Absorption, instant
  Heal/Harm, applicability, Bad/Raid Omen, Infested hurt, Wind Charged/Weaving/Oozing removal,
  slime construction facts and client particle chance.

The 40 effect identities, categories, colors, modifiers and potion compositions remain locked
DataOnly inputs in the imported `minecraft:mob_effect` registry. Focused tests validate its exact
entry count when the workspace-owned content bundle is present; the repository content gate owns
the bundle unconditionally.

## Instance and tick ordering

Infinite duration is `-1` and outranks every finite duration. A stronger shorter instance copies
the old visible value ahead of its previous hidden chain. Equal strength extends only when longer;
a longer weaker instance merges recursively into the hidden chain. Incoming nonambient state can
clear ambient, while particle/icon flags always copy. An unchanged accepted merge still calls
`onEffectStarted` but reports no map mutation.

A zero-duration visible instance is removed before callback execution. Otherwise cadence observes
remaining duration, or entity tick count for an infinite instance. Callback false removes before
any decrement or promotion. Hidden finite durations decrement recursively; visible duration then
decrements and promotes the hidden head exactly at zero, even when that hidden duration has already
reached zero. Promotion refreshes attributes. A still-present positive duration divisible by 600
updates without attribute refresh.

Attribute refresh removes modifiers by stable ID before adding permanent
`baseAmount*(amplifier+1)` values, then exposes health/absorption clamp, dimension and waypoint
consequences. Ordinary removal invokes one callback. Remove-all copies and clears the server map
before modifier removal. Concurrent mutation stops the remaining pass and reports deferred keys.

## Specialized behavior

Periodic intervals retain Java masked right-shift behavior and use every-tick fallback when the
shifted interval is nonpositive. Poison compares floating health strictly above one. Instant
Heal/Harm retains Java wrapped left shifts, inversion tags and `(int)(scale*amount+0.5)` conversion.

Bad Omen converts to 600-tick Raid Omen only through all player/difficulty/village/capacity gates;
Raid Omen triggers at remaining duration one and removes before decrement. Infested uses inclusive
`<=0.1`. Removal effects require exactly `KILLED`. Weaving reports 2–3 attempts of fifteen samples;
Oozing applies its max-cramming scan/cap, skips only failed constructions, consumes yaw only after
construction, creates size-two triggered slimes at Y+0.5 and never finalizes or rolls back failed
insertion.

## Validation

`crates/ferrite-gameplay/tests/slices/entities/ent_006.rs` owns the source-specified effect slice.
Its fourteen tests cover the 40-entry registry, merge and hidden-chain boundaries, unchanged and
forced add, every tick removal/update path, concurrent mutation, attribute scaling, every
specialized cadence/instant/applicability branch, omen transitions, killed-only hooks, cramming,
construction failure and RNG cardinality. `G01-P7-B1` remains responsible for composing effect
callbacks with damage, death, spawning, persistence and client projection.
