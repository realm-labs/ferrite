# Minecraft 26.2 dimension runtime

Ferrite's `WGEN-DIMENSION-001` owner is
`ferrite-world::generation::dimension`. The runtime keeps four identities separate:

- a level's dimension key, which owns weather exclusions, storage paths, and later portal routing;
- its dimension-type record, which owns height, light, scale, rendering, and dragon-fight gates;
- named global clocks, which may advance independently of fixed-time presentation;
- position-sampled environment attributes, which can differ inside one dimension.

This separation is observable. A custom key using the End type can have weather and an End dragon
fight simultaneously, the literal End key remains weather-ineligible even with an Overworld type,
and the Nether has no default clock while its universal villager timeline still reads the named
Overworld clock.

## Record and environment boundaries

`DimensionType` validates the codec scale and vertical ranges plus the constructor's 16-block
alignment, logical-height, and top-bound invariants. The four locked records include their exact
attribute overrides and expanded timeline holder order. Build height is inclusive; section lookup
is relative to `min_y`; logical height never truncates storage.

`environment` declares all 48 typed attributes in locked order. Each declaration owns positional,
network, spatial-interpolation, and sanitizer metadata. `EnvironmentLayers` preserves construction
order as dimension, biome, selected timelines, and eligible weather. Dimension-only reads skip
positional layers; final reads sanitize again. Camera interpolation exposes the 6×6×6 separable
kernel probe, weighted type-aware folding, previous/current samples, and render partial-tick lerp.
The nonpositional cache exposes separate every-level-tick and direct-clock invalidation events.

Dimension JSON remains part of the locked content bundle. The runtime constructors are the typed
post-decode boundary: malformed scale, height, logical height, light limit, or light provider data
cannot construct an active record. Registry import continues to own raw JSON/schema disposition.

## Clock and timeline boundary

`clock` stores signed total ticks, Java-float partial ticks and rate, and pause state. The manager
advances all named clocks only under `advance_time`; network rate becomes zero while paused or
gamerule-frozen. Explicit mutations increment a broadcast/cache generation. Default-clock
operations fail when absent, while explicit named operations remain valid. Sleep advancement,
weather reset, and village-siege marker selection are independent decisions.

`timeline` validates periods, markers, keyframe order, repeated-tick runs, and the permitted period
endpoint. Sampling uses signed `rem_euclid`, strictly-greater segment ends, periodic wrap segments,
Java-float fractions, constant/linear/cubic-bezier easing, and each attribute's typed lerp. The
locked `day`, `moon`, `early_game`, and `villager_schedule` records carry all official markers and
tracks; network projection removes nonsyncable tracks.

## Spawn and caller contracts

`spawn` owns only dimension-derived decisions. Monster darkness preserves the initial `nextInt(32)`
abort, conditional block-light gate, and constant-versus-uniform provider draw count. Initial spawn
plans apply border radius reduction, the radius-one quirk, the 1,024 candidate cap, source step
selection, one random offset, radius-zero `spawn_search` tickets, and the original-suggestion
fallback. The caller retains generator heightmaps, chunk loading, surface/liquid/collision checks,
and the final height fix.

Bed evaluation checks explosion before independent spawn and sleep decisions. It exposes the exact
power-five bad-respawn transaction, per-tick sleep recheck, retained-bed gate, and the separate
position-local respawn-anchor boolean. Portal search/creation and border clamping are intentionally
owned by the following WGEN-005 and WGEN-006 batches; this module supplies their scale and identity
inputs without pre-rounding coordinates.

All collections that affect resolution use stable vector or ordered-map iteration. No environment,
height, time, or coordinate query consumes randomness; only the documented monster and initial
spawn branches receive caller-owned random draws.
