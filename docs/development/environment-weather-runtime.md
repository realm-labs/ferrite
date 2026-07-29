# Environment Weather Runtime

`G01-P5-S011` implements the `SourceSpecified` `ENV-WEATHER-001` slice. The protocol-neutral
decision kernel lives in `ferrite-gameplay::environment::weather`; world control and Region
adapters provide ordered observations and commit the resulting state, packet, block, game-event
and entity effects.

## Ownership and ordering

The five persisted weather fields are world-owned input. A world-control activation advances that
record once for every capable level in stable server level-tick order and emits each level's
ordered logical-tick snapshot; the final record is the durable result. Each level separately owns
its previous/current Rain and Thunder strengths. This preserves Minecraft's shared timers and
potentially divergent level ramps without introducing cluster-wide mutable Region state.

The fixed level order is world border, weather, sleep, clock, scheduled Block ticks, scheduled
Fluid ticks and chunk work. Spawning chunks are shuffled with the level gameplay stream; every
eligible chunk attempts Thunder before natural spawning. Only after all spawning chunks do
block-ticking chunks run precipitation before random Block/Fluid ticks. Custom spawners remain
last.

## Weather targets, ramps and synchronization

- Weather capability requires Sky light, no ceiling and a level key other than The End.
- Clear override, Thunder-before-Rain timer processing, inclusive duration ranges, zero/one
  toggles and the `advance_weather` gate are exact. Disabling that rule freezes targets but not
  level-local `0.01F` ramps.
- Commands replace only the five shared fields. Deep sleep clears them only when
  `advance_weather` is enabled and that level currently passes the strict Rain predicate.
- Strength changes produce dimension-scoped projections. Rain threshold crossings additionally
  produce global Start/Stop, Rain strength and Thunder strength in that order. Client imports snap
  previous/current values; they do not autonomously ramp.
- Save construction initializes Rain to one for a saved Rain target and Thunder to one only for a
  saved Rain-plus-Thunder target.

## Region-local environment work

The gameplay RNG and the wrapping `randValue = randValue * 3 + 1013904223` position stream remain
separate. Each block-ticking chunk performs exactly `random_tick_speed` bounded draws; only a zero
advances the position stream. Freezing is evaluated before active-Rain admission, so dry weather
can freeze exposed source Water. Snow and precipitation receivers run only afterward.

Temperature adjustment, the strict `0.15F` Rain/Snow split, build/light/Water-edge gates, Snow
layer limits and ignored write results are explicit. Empty, Water and Powder Snow cauldrons own
their exact strict chance comparisons; wrong-type and full layered cauldrons consume the chance
draw before rejection.

Thunder requires active Rain and Thunder plus the exact `nextInt(100000) == 0` hit. A hit alone
advances the position stream. The adapter supplies the POI manager's nearest eligible rod and the
ordered alive, sky-visible entity query; the kernel fixes rod, entity and raised-fallback selection,
local Rain validation, strict trap probability, lightning-rod exclusion and failure-independent
horse/bolt commits. Trap lightning remains visual-only even if horse creation or admission fails.

## Boundaries

Biome, heightmap, visibility, POI, entity, difficulty, content-tag and block-write observations
remain authoritative in their existing owners. Bolt strike behavior, transformations, damage,
fire, criteria and trap-horse AI remain entity/mob responsibilities. Client Rain rendering,
particles and sound remain `CLI-EFFECT-001`.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/environment/env_004.rs`. Its 15 tests cover phase order,
capability, clear/timer boundaries, inclusive samples, disabled advancement, multi-level shared
state, strict ramps and packet ordering, commands/sleep/load/join behavior, random stream
separation, `0/1/3` precipitation budgets, temperature/freezing/Snow, cauldron equality and
post-draw rejection, lightning targets/local Rain, trap equality and factory/admission failures.
