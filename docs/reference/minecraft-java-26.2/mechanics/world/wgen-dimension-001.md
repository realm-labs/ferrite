# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-DIMENSION-001` — Dimension type gates time scale, environment, coordinates, and spawn semantics

**Parent:** `WGEN-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server/client control flow and four dimension records fix validation,
height boundaries, layer resolution, clocks/timelines, light/weather/spawn gates, coordinate
scaling, respawn selection and client presentation inputs; `EXP-WGEN-002` is a conformance probe.

**Applies when:**

A dimension type decodes, a server or client level is constructed, an environment attribute is
sampled, a clock/timeline advances, a caller tests vertical/light/environment/spawn properties, or a
command/portal converts a position between levels. Dimension type, dimension key and level
stem/generator are independent identities.

**Authoritative state:**

The record owns `has_fixed_time`, `has_skylight`, `has_ceiling`, `has_ender_dragon_fight`, positive
`coordinate_scale`, `min_y`, storage `height`, `logical_height`, an `infiniburn` block set,
`ambient_light`, monster block-light limit and spawn-light provider, skybox, cardinal-light type, a
typed environment-attribute map, a timeline set and optional default clock. The codec admits
coordinate scale in `[0.000009999999747378752,30,000,000]`, height at least `16`, codec-bounded
min/height/logical height, and the constructor additionally requires height and min Y to be
multiples of `16`, `logical_height<=height`, and `min_y+height<=MAX_Y+1`. Omitted `has_fixed_time`
is false; skybox/cardinal light default to `overworld`/`default`; attributes/timelines default
empty; default clock is absent.

The four locked records are:

#### `overworld`

**Boolean/scale fields:**

fixed false; skylight true; ceiling false; dragon fight false; scale `1`

**Vertical, ambient and monster fields:**

min `-64`; height/logical `384`; ambient `0`; block-light limit `0`; spawn-light uniform inclusive
`0..7`

**Render, clock and tag fields:**

overworld skybox/default cardinal light; `#infiniburn_overworld`; `#in_overworld`; default clock
`overworld`

#### `overworld_caves`

**Boolean/scale fields:**

exactly Overworld except ceiling true

**Vertical, ambient and monster fields:**

exactly Overworld

**Render, clock and tag fields:**

exactly Overworld

#### `the_end`

**Boolean/scale fields:**

fixed true; skylight true; ceiling false; dragon fight true; scale `1`

**Vertical, ambient and monster fields:**

min `0`; height/logical `256`; ambient `0.25`; block-light limit `0`; spawn-light constant `15`

**Render, clock and tag fields:**

End skybox/default cardinal light; `#infiniburn_end`; `#in_end`; default clock `the_end`

#### `the_nether`

**Boolean/scale fields:**

fixed true; skylight false; ceiling true; dragon fight false; scale `8`

**Vertical, ambient and monster fields:**

min `0`; height `256`; logical `128`; ambient `0.1`; block-light limit `15`; spawn-light constant
`7`

**Render, clock and tag fields:**

no skybox/Nether cardinal light; `#infiniburn_nether`; `#in_nether`; no default clock

The dimension-owned attribute overrides are also locked data. Overworld and Overworld Caves share
cave mood `(extent 8, offset 2, delay 6000)`, creative/default music delayed `12000..24000`, the
default bed rule, portal piglin true, anchor false, and ambient/cloud/fog/sky colors
`#0a0a0a/#ccffffff/#c0d8ff/#78a7ff` with cloud height `192.33`. The End has the same cave mood,
replacing End music delayed `6000..24000`, bed `(never,never,explodes)`, anchor false,
ambient/fog/sky colors `#3f473f/#181318/#000000`, and sky light color/factor `#ac60cd/0`. Nether has
bed `(never,never,explodes)`, raid false, fast lava true, piglins-zombify false, anchor true,
sky-light level `4`, snow-golem-melts and water-evaporates true, ambient color `#302821`, lava
dripstone particles, fog start/end `10/96`, and sky-light color/factor `#7a7aff/0`; every unlisted
attribute begins from its registered default before later layers.

`#in_overworld` expands `#universal`, `day`, `moon`, and `early_game`; the End and Nether sets
contain only `#universal`; `#universal` is `villager_schedule`. Consequently Nether has no
implicit/default clock but its universal villager timeline still samples the Overworld clock named
by that timeline. The End owns a separate default clock while its timeline set is only universal.

**Environment-attribute declaration audit:**

All 48 registry IDs have a typed default. The 24 `visual/*` defaults, in registry order, are fog
color `0`, fog start `0`, fog end `1024`, sky-fog end `512`, cloud-fog end `2048`, water-fog color
`-16448205`, water-fog start `-8`, water-fog end `96`, sky color `0`, sunrise/sunset color `0`,
cloud color `0`, cloud height `192.33`, sun/moon/star angles `0`, full-moon phase, star brightness
`0`, block-light tint `-10100`, sky-light color `-1`, sky-light factor `1`, night-vision color
`-6710887`, ambient-light color `-16777216`, `dripping_dripstone_water`, and an empty
ambient-particle list. The four `audio/*` defaults are empty background music, volume `1`, empty
ambient sounds and false firefly-bush sounds. The 20 `gameplay/*` defaults are sky-light level `15`,
can-start-raid true, water-evaporates false, bed rule
`(sleep when dark, set spawn always, no explosion, no-sleep message)`, respawn-anchor-works false,
portal-spawns-piglin false, fast-lava false, increased-fire-burnout false, eyeblossom `default`,
turtle-egg chance `0.002`, piglins-zombify true, snow-golem-melts false, creaking-active false,
surface-slime chance `0`, cat-gift chance `0`, bees-stay-in-hive false, monsters-burn false,
patrol-spawn true, and adult/baby villager activity `idle`. Fog/sky/cloud/water-fog nonnegative
endpoints clamp to `[0,+infinity]`; star brightness, sky-light factor, music volume and the three
probability families clamp to `[0,1]`; sky-light level clamps to `[0,15]`. Data decode rejects
out-of-range values; the completed runtime layer result is sanitized again.

Every visual/audio attribute is positional and network-syncable. All visual values except moon
phase, default particle and ambient-particle list are spatially interpolated; audio values are not.
The only nonpositional gameplay attributes are sky-light level and fast lava. Those two plus water
evaporation, piglin zombification and creaking activity are syncable; the other gameplay attributes
remain server-only in the dimension network map. A primitive map value is an override entry; a full
entry applies its declared modifier to the preceding value.

**Transition and ordering:**

Level construction adds layers in this exact order: dimension constants, one biome positional layer
for every attribute present in any loaded biome, each dimension-selected timeline in holder-set
order, then builtin weather layers only when `canHaveWeather()` is true. Leading constants are
folded into the registered default; later constant, time and positional layers apply sequentially
and the final value is sanitized. A missing sampler returns the registered default. `BlockPos`
lookup samples its center. A dimension-value lookup applies constant/time layers but skips
positional layers (and rejects a positional attribute only in an IDE assertion build). The server
invalidates every cached nonpositional value at the start of every level tick, even when that level
is frozen; direct clock mutation also invalidates all levels after broadcasting the new clock state.

All four locked timelines read the Overworld clock. `day` has period `24000`, six markers (day
`1000`, noon `6000`, night `13000`, midnight and siege `18000`, wake `0`) and 18
audio/gameplay/visual tracks; `moon` has period `192000` and constant eight-phase moon/surface-slime
tracks at `24000`-tick intervals; nonperiodic `early_game` changes patrol admission from false at
`0` to true at `120000`; periodic `villager_schedule` has adult and baby activity tracks over
`24000`. A period must be positive; marker ticks are in `[0,period)`, track keyframes are
nonnegative and ordered, a repeated-tick run normally admits three entries (the degenerate all-same
list admits only two because validation seeds its comparison with the last tick), and periodic track
ticks may include the period endpoint. Sampling uses `floorMod(totalTicks,period)`, chooses the
first segment whose end is strictly greater than the sample, returns endpoints outside a segment,
otherwise applies the track easing to the Java-float fraction before the attribute lerp. Periodic
tracks add wrap segments from the last keyframe at `last-period` to the first and from the last to
the first at `first+period`; one-keyframe tracks are constant. Network timeline data omits every
nonsyncable attribute track.

A direct positional lookup chooses the noise biome at the supplied coordinate and applies its
attribute modifier. The client camera probe first scales position by `0.25`, subtracts half a cell,
visits a `6×6×6` cube with separable interpolated kernel `[0,1,4,6,4,1,0]`, combines weights by
identical biome attribute-map identity, and folds those maps with the attribute type's spatial lerp.
It retains previous/current samples and applies the type's partial-tick lerp for rendering. This
interpolation changes only attributes declared spatially interpolated; discrete values use their
type-specific selection.

**Height, light and rendering consumers:**

Build height is inclusive `min_y..min_y+height-1`; section indices are floor-divided section
coordinates relative to `min_y`, and outside-height tests reject both adjacent endpoints. Logical
height is returned separately and does not truncate Nether storage/build height. `has_skylight`
configures sky-light storage/serialization and gates daylight-detector ticking and phantom spawning.
Weather additionally requires no ceiling and a dimension key other than literal `minecraft:the_end`;
therefore a custom key reusing the End type is weather-capable, while the actual End key is not.
Ambient-light brightness is `lerp(ambient, 1/(4-3*(raw/15)), 1)`. Skybox and cardinal-light type
select client sky and directional-light presentation; the environment attributes then feed fog, sky,
cloud, lightmap, particles, music and ambient-sound consumers at their respective camera/tick/render
query points.

**Time semantics:**

`has_fixed_time` does not store a time and does not pause a clock. It only forces both
`isBrightOutside()` and `isDarkOutside()` false; otherwise bright is `skyDarken<4` and dark is its
negation. The global clock manager ticks every clock only while `advance_time` is true. An unpaused
clock performs Java-float `partial += rate`, adds `floor(partial)` to its signed long total, then
subtracts that integer from the partial; the initial rate is `1`, and saved state preserves total,
partial, rate and paused. Its network state sends rate `0` while paused or while `advance_time` is
false, otherwise the actual rate. Level game time remains a separate per-level tick counter.
`getDefaultClockTime()` returns the optional default clock's ticks or `0`. Implicit `/time`
operations fail when it is absent, while explicit clock operations remain possible. When enough
players are deeply sleeping, the server moves the optional default clock to its wake marker only if
`advance_time` is true and the clock exists, then wakes everyone; raining weather resets
independently under `advance_weather`. Village siege rolls only at the default clock's siege marker,
so absence of a default clock suppresses that roll.

**Coordinates and identity:**

`getTeleportationScale(source,destination)` is the Java-double quotient
`source.coordinate_scale/destination.coordinate_scale`. `CommandSourceStack#withLevel` multiplies
X/Z by it, preserves Y/rotation and performs no clamp or rounding. Nether-portal routing chooses
Nether versus Overworld by dimension **key**, multiplies entity X/Z, preserves Y as a double input,
then asks the destination border to clamp and floor a `BlockPos`; portal search/creation owns all
later rounding. Dimension storage folders likewise derive from the dimension key. By contrast,
dragon-fight construction follows `has_ender_dragon_fight`, so a custom level reusing the End type
initializes a fight even under a non-End key.

**Spawn and respawn consumers:**

Monster darkness first compares sky light with `nextInt(32)` and may abort; when the type's
block-light limit is below `15`, block light above that inclusive limit aborts; final local raw
brightness (with thunder sky darkening `10` where weather applies) must be at most the type's
sampled spawn-light provider. The Overworld provider consumes a second uniform draw; the End/Nether
constants do not. `has_ceiling` makes natural spawning descend from the selected height through air
before placement, makes maps halve their sample radius and draw deterministic dirt/stone noise
instead of terrain, and makes initial player-spawn columns start at generator spawn height instead
of `MOTION_BLOCKING`.

Initial non-adventure spawn search uses radius `max(0,respawn_radius)`, reduces it to floored border
distance when nearer, but forces radius `1` when that distance is at most one. It checks at most
`min(1024,(2r+1)^2)` candidates in a permutation with one thread-local-random offset and step
`count-1` for counts at most `16`, otherwise `17`, loading each candidate chunk with a radius-zero
`spawn_search` ticket; after exhaustion it height-fixes the original suggestion. Adventure skips the
permutation and only height-fixes the suggestion. A ceiling column starts at generator spawn height;
another starts at `MOTION_BLOCKING`; candidates below min Y, submerged/invalid surface stacks,
non-full support, liquid or player collision fail. Bed interaction samples the local `bed_rule`:
explosion is checked first and removes both halves before a power-`5`, fiery block-interaction
bad-respawn explosion; otherwise `can_set_spawn` and `can_sleep` are independent, allowing spawn to
be recorded before a sleep-denial result. A sleeping player rechecks `can_sleep` at its current
position every server tick. Death respawn accepts a retained bed only when its local rule still
allows setting spawn. Respawn anchors use their separate local boolean; a charged anchor in a false
location follows its bad-respawn explosion path instead of setting spawn.

**Branches and aborts:**

Decode success/failure; dimension key versus reused type; build/logical/storage bounds; sky/no-sky,
ceiling/no-ceiling, literal End key, fixed/outside-light test; optional/default/explicit clocks;
layer absent/present/positional/interpolated/syncable; custom modifier; constant/uniform spawn
light; bed set/sleep/explode combinations; anchor true/false; border-limited spawn; missing portal
destination.

**Side effects:**

Light-engine selection, weather layers, clock packets/cache invalidation, sky/audio/render inputs,
dragon-fight state, map pixels, spawn tickets/entities, bed spawn/sleep/explosion and anchor
spawn/explosion.

**Gates:**

Dimension fields, dimension and clock keys, attribute declarations/layers, timeline membership,
biome and weather state, the three owned gamerules, world border, collision and downstream caller
eligibility. Fire, fluid, weather, lighting, portals and each downstream mob/block transaction
retain their dedicated leaf rules; this leaf owns the dimension/type/attribute value and gate
supplied to them.

**Constants and randomness:**

Structural and registry defaults are listed above; moon brightness by phase index is
`[1,0.75,0.5,0.25,0,0.25,0.5,0.75]`; spawn search caps at `1024`; bed/anchor explosion power is `5`.
Coordinate and Gaussian arithmetic use Java doubles; environment numeric values are Java floats
after decode. Attribute resolution, height tests, clock selection and scaling consume no RNG.
Monster admission and initial-spawn permutation consume RNG exactly where described; downstream
attribute consumers own their own draws.

**Boundary cases and quirks:**

`overworld_caves` differs structurally from Overworld only by ceiling but that changes maps, natural
spawn positioning and initial spawn height. Nether logical height `128` is not a build ceiling.
Fixed-time types can have advancing clocks, and a level with no default clock can still run
timelines tied to another named clock. Weather depends on both type fields and the literal End key.
Attribute values can vary by position even inside one dimension, so bed/anchor and other gameplay
gates must sample their action position rather than cache a dimension boolean. Coordinate scale
affects X/Z only and can produce infinities only outside codec-valid data.

**Evidence:**

`Confirmed`; `OFF-DATA-001`, `OFF-SERVER-001`, `OFF-CLIENT-001`. Anchors: `DimensionType`,
`LevelHeightAccessor`, `EnvironmentAttributes`, `EnvironmentAttributeMap`,
`EnvironmentAttributeSystem`, `EnvironmentAttributeProbe`, `SpatialAttributeInterpolator`,
`GaussianSampler`, `ServerClockManager`, `Timeline`, `AttributeTrackSampler`,
`KeyframeTrackSampler`, `Level#canHaveWeather`, `Level#isBrightOutside`,
`CommandSourceStack#withLevel`, `NetherPortalBlock#getPortalDestination`, `PlayerSpawnFinder`,
`Monster#isDarkEnoughToSpawn`, `BedBlock#useItemOn`, `ServerPlayer#startSleepInBed`, and
`RespawnAnchorBlock#canSetSpawn`.

**Test vectors:**

(1) Decode legal endpoints and every constructor-invalid height combination; assert inclusive build
endpoints and independent logical height. (2) Query all four records and all 48 defaults, then layer
dimension/biome/timeline/weather modifiers at exact biome and partial-tick boundaries. (3) Reuse
each vanilla type under custom keys, including the actual/nonactual End weather and dragon-fight
cross-product. (4) Advance/freeze/pause/rate-change both clocks with fixed true/false, absent
default clocks and universal timelines; test sleep skip and implicit/explicit `/time`. (5) Scale
positive, negative and fractional X/Z through command and portal callers at border/floor boundaries
while varying only Y. (6) Sweep monster sky/block/local/provider thresholds and draw counts. (7)
Test ceiling/non-ceiling maps, natural spawn and the complete initial-spawn
radius/border/candidate/fallback matrix. (8) Exercise every bed `(set,sleep,explode)` combination
and anchor boolean with a position-varying biome layer. Run `EXP-WGEN-002` only as the executable
conformance matrix.
