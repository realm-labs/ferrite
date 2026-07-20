# Environment mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENV-WEATHER-001` — Weather timers and exposed local effects are separate layers

**Parent:** `ENV-004`, `ENV-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server and client jars fix the shared saved state, every timer/ramp
transition, command and sleep mutation, weather synchronization, local precipitation predicates and
writes, cauldron hooks, chunk-phase placement, lightning probability/target/trap branches, RNG sites
and failure behavior. Rendering, sound and particle policy after synchronized weather/entity state
is explicitly owned by `CLI-EFFECT-001`; subsequent lightning/trap-entity behavior is
entity/mob-owned. `EXP-ENV-002` is a conformance trace, not an implementation blocker.

**Applies when:**

A normally running `ServerLevel` reaches its early weather phase, an eligible chunk reaches thunder
or block-ticking work, a weather command mutates the server record, a sleep transition clears
weather, or a client receives weather synchronization.

**Authoritative state:**

The server owns one persisted `WeatherData` record—`clearWeatherTime`, `rainTime`, `thunderTime`,
`raining`, and `thundering`—shared by every level. Each level separately owns current/previous rain
and thunder strengths. Other inputs are the level key/type, game rules, shuffled spawning chunks and
block-ticking chunks, the level gameplay RNG plus the independent `randValue` position stream,
heightmaps, biome precipitation/temperature, block light/state/fluid, POIs/entities, difficulty and
entity admission.

**Transition and ordering:**

In each normally running level tick, weather follows the world border and precedes sleep, clock,
scheduled ticks and chunk work. It mutates the shared target record, advances that level's
strengths, publishes changed strengths and threshold crossings, then later chunk-source work runs
all shuffled spawning-chunk thunder/natural-spawn calls before all block-ticking
precipitation/random-tick calls. The detailed timer, local-effect and entity transactions below
execute in the stated order.

**Weather capability, timers, and ramps:**

A level can have weather iff its dimension type has sky light, lacks a ceiling, and its key is not
`minecraft:the_end`. Its early weather phase runs after the world-border update and only while the
tick-rate manager runs normally. If `advance_weather` (ordinary default `true`) is enabled, positive
clear time is decremented, forces both targets false, and rewrites each other timer to `0` when that
target was true or `1` when it was false. Otherwise thunder is processed before rain: a positive
timer decrements and toggles its target exactly when it becomes zero; a nonpositive timer samples a
new inclusive duration without toggling—thunder uses `3,600..15,600` while true and delay
`12,000..180,000` while false, rain uses `12,000..24,000` while true and delay `12,000..180,000`
while false. The updated five fields are persisted in that order. Independently of
`advance_weather`, each level copies current strength to previous, adds or subtracts `0.01F`
according to the shared target, and clamps to `[0,1]`, thunder first and rain second. `isRaining()`
is capability plus `rainLevel > 0.2`; `isThundering()` is capability plus
`(thunderLevel * rainLevel) > 0.9` because the public thunder getter multiplies by interpolated
rain. With multiple capable custom dimensions, every level invocation advances the same timers, in
server level-tick order, while its own strengths may diverge.

**Commands, sleep, persistence, and clients:**

`/weather clear [duration]` writes `(clear=D,rain=0,thunder=0,false,false)`; rain writes
`(0,D,D,true,false)`; thunder writes `(0,D,D,true,true)` to the shared record. An omitted duration
samples the command-source level RNG from rain delay, rain duration, or thunder duration
respectively; the setter changes neither strengths nor packets, so the next level weather phase
exposes it. A successful deep-sleep transition runs after that level's early weather phase: after
optional clock advance and waking players, it resets all timers to zero and both targets false only
when `advance_weather` is true and that level currently satisfies `isRaining()`; its strengths begin
falling on its next weather phase. Save/load preserves the five shared fields; construction of a
capable level initializes current rain to `1` for a saved rain target and current thunder to `1`
only for saved rain+thunder. Each strength change emits a dimension-scoped level-change packet.
Crossing the `isRaining()` threshold then emits global `START_RAINING`/`STOP_RAINING` followed by
global rain and thunder strengths; clients set both previous and current strength to each clamped
packet value (`START` first sets rain to `0`, `STOP` to `1`, before the following strength packet).
Joining/level-changing clients receive start plus both strengths only when the destination already
satisfies `isRaining()`. Clients do not autonomously ramp these fields.

**Chunk order and precipitation sample:**

`ServerChunkCache` collects and shuffles spawning chunks with the level RNG. In each one,
entity-ticking-range thunder runs before natural spawning. After all spawning chunks, block-ticking
chunks run `tickChunk`; before any section random ticks it performs exactly `random_tick_speed`
(default `3`, minimum `0`) `nextInt(48)` draws. Each zero calls
`getBlockRandomPos(chunkMinX,0,chunkMinZ,15)`, which advances only the separate integer recurrence
`randValue = randValue * 3 + 1013904223`, shifts it right by two, and takes X bits `0..3`, Z bits
`8..11` and Y bits `16..19` (thus Y `0..15`). That position is converted to the `MOTION_BLOCKING`
surface, with `below = surface.below()`, and the biome is read at `surface`. Therefore speed zero
disables this entire branch and misses consume gameplay RNG without advancing the position stream.

**Ice, snow, and precipitation receivers:**

Freezing is tested first even when the level is not raining. The biome's height-adjusted temperature
starts with its data-defined modifier/base; above `seaLevel+17`, subtract
`(temperatureNoise(x/8,z/8)*8 + y - (seaLevel+17))*0.05F/40`. Temperature at least `0.15F` is
rain-warm; colder is snow-cold. `below` becomes ice only when cold, inside build height, block light
is below `10`, its fluid is source water represented by a `LiquidBlock`, and at least one horizontal
neighbor is not water. If `isRaining()` is false, processing ends after this freeze attempt.
Otherwise `max_snow_accumulation_height` (default `1`, admitted range `0..8`) gates snow: a
snow-cold `surface` inside build height with block light below `10`, air-or-snow state, and
survivable default snow receives one layer; existing snow grows only while its layer count is below
`min(rule,8)`. The push-entities-up return and every block-write result are ignored. Finally the
biome precipitation at `below` is computed; if non-`NONE`, the block at `below` receives it. Only
empty, water, and powder-snow cauldrons override this hook: every eligible receiver first draws
`nextFloat()` (`<0.05F` for rain, `<0.1F` for snow); an empty cauldron becomes default water/powder
snow and emits `BLOCK_CHANGE`, while a nonfull layered cauldron increments only when its stored
precipitation type matches. Layer/type/full rejection happens after the chance draw. Lava cauldrons
and all other blocks use the no-op base hook.

**Lightning admission and target selection:**

Thunder is attempted once per shuffled spawning chunk that is in entity-ticking range, before that
chunk's natural spawn call, even when `spawn_mobs` is false. It requires current `isRaining()`,
current `isThundering()`, and `nextInt(100000) == 0`; a miss consumes no target-position advance. A
hit advances the same `randValue` stream for an X/Z column, takes its `MOTION_BLOCKING` surface,
then asks the POI manager for the closest lightning-rod POI within 128 whose block Y equals
`WORLD_SURFACE(x,z)-1`; a match targets the block above that rod. Without a rod, it builds the
full-block AABB from the surface through `maxY+1`, inflates it by `3`, collects alive sky-visible
living entities, and if nonempty selects one block position with `nextInt(size)`. Otherwise it uses
the surface, except a `minY-1` surface is raised by two. The final target must have local
precipitation exactly `RAIN`: active level rain, sky visibility, no `MOTION_BLOCKING` surface above
it, and a rain-warm precipitation biome. Snow precipitation therefore rejects a strike.

**Skeleton-trap and entity commit:**

After a valid target, `spawn_mobs` gates the only trap draw. When enabled,
`nextDouble() < effectiveDifficulty * 0.01` and the target's below block is not in
`#minecraft:lightning_rods` selects a trap. The server creates an event-spawn skeleton horse, marks
it trap, sets age zero, places it at the target's integer corner and ignores entity-admission
failure. It then independently creates an event lightning bolt at the target bottom center, marks
the bolt `visualOnly = trap`, and ignores admission failure. The trap flag remains true even if
horse creation/admission failed; a trap bolt is therefore visual-only, while an ordinary weather
bolt is not. Entity-type creation returning null simply skips that entity, with no rollback.
Subsequent bolt strikes, transformations, damage, fire, sounds, criteria, and trap-horse AI belong
to `ENT-LIFECYCLE-001`/`MOB-SPAWN-001`, after this weather transaction has admitted the entities.

**Branches and aborts:**

Frozen tick-rate manager; incapable dimension; `advance_weather` disabled (timer targets freeze but
strengths still approach them); clear override; chunk absent from the applicable
spawning/block-ticking set; entity-ticking-range denial; speed/chance miss;
warm/light/interior-water/snow-survival denial; inactive rain; no local rain at lightning target;
absent POI/entity; `spawn_mobs` denial or trap miss; entity factory/admission failure. No branch
catches up elapsed unloaded or frozen time.

**Constants and randomness:**

All timer ranges are inclusive integer samples. Strength delta is exactly `0.01F`; rain/thunder
predicates are strict `>0.2` and `>0.9`; precipitation admission is `nextInt(48)==0`; cauldron
comparisons are strict `<0.05F`/`<0.1F`; lightning is `nextInt(100000)==0`; rod radius is 128,
entity-target inflation is 3, and trap comparison is strict `< effectiveDifficulty*0.01`. The level
gameplay RNG and wrapping 32-bit `randValue` recurrence are distinct; every server draw/advance site
and post-draw abort is specified above. Client presentation randomness belongs to `CLI-EFFECT-001`.

**Side effects:**

Shared saved weather fields; per-level strengths and sky brightness; dimension-scoped and
threshold-triggered global game-event packets; ice/snow writes and entity displacement; cauldron
states and `BLOCK_CHANGE`; skeleton-horse/lightning admissions; later client
rendering/particles/sounds and entity behavior. Fire's rain checks consume the same local weather
state in `ENV-FIRE-001`; precipitation ticking does not directly extinguish fire.

**Gates:**

Tick-rate admission; dimension sky-light/ceiling/key capability; `advance_weather`,
`random_tick_speed`, `max_snow_accumulation_height`, and `spawn_mobs`; spawning/block/entity-ticking
chunk membership; active rain/thunder strengths; heightmaps, sky, biome precipitation/temperature,
build height, block light/state/fluid/survival; POI/entity candidates; difficulty; entity
creation/admission; client dimension/tracking.

**Boundary cases and quirks:**

Shared targets can advance multiple times per server tick in multiple weather-capable dimensions
while strengths remain per level. Disabling `advance_weather` does not freeze ramps. Dry weather can
still freeze water. Full or wrong-type layered cauldrons consume their chance draw. Snow biomes
reject natural lightning because `isRainingAt` requires `RAIN`, not merely precipitation. Trap
lightning is visual-only, ordinary lightning is not, and failed horse admission does not change that
choice. Global threshold packets can reset weather fields on clients outside the crossing level
before the following global strength values.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; all fully qualified locators above plus
`net.minecraft.world.level.biome.Biome#shouldFreeze(net.minecraft.world.level.LevelReader,net.minecraft.core.BlockPos)`,
`Biome#shouldSnow`, `net.minecraft.world.level.block.CauldronBlock#handlePrecipitation`,
`net.minecraft.world.level.block.LayeredCauldronBlock#handlePrecipitation`,
`net.minecraft.server.commands.WeatherCommand`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleGameEvent`, and
`net.minecraft.world.level.Level#getBlockRandomPos(int,int,int,int)`. `EXP-ENV-002` replays
boundaries but owns no unknown.

**Test vectors:**

Timer `0/1` under each target and a one-/two-tick clear override; disable `advance_weather` during
ramps; two capable custom dimensions; command before/after early weather and same-tick sleep reset;
join at rain threshold; speed `0/1/3` with forced `48` misses/hits; freezing while dry; snow max
`0/1/8`; each cauldron type/fullness with forced float equality; spawning versus block-ticking chunk
sets; lightning threshold/chance, rod/entity/fallback targets, snow-biome rejection, `spawn_mobs`
denial, trap equality, and factory/admission failure.
