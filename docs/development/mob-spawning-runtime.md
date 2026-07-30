# MOB-001 Spawning Runtime

`G01-P7-S007` implements six source-specified spawning slices as protocol-neutral transitions. The
owning Region supplies entity/player snapshots, level and persisted state, placement observations
and named-stream draws. The runtime returns ordered timer, admission, construction, accounting and
presentation decisions without owning a parallel world or ambient RNG.

## Responsibility split

`ferrite-gameplay::mob::runtime::mob_001` has six leaf-aligned owners:

- `hostile` owns the live `spawn_mobs && spawn_monsters` conjunction, chunk-cache projection,
  natural/custom filtering, and the Endermite, reinforcement, Creaking-heart and portal-piglin
  direct consumers;
- `natural` owns category order and caps, snapshot/local/potential accounting, candidate and pack
  walks, list selection, construction failure boundaries, placement defaults and chunk generation;
- `patrol` owns the pausable countdown, one-player attempt, signed offsets, group walk, member
  predicates, leader entity-RNG target and unchecked insertion;
- `phantom` owns the pausable countdown, sky gate, encounter-order player trials, difficulty then
  insomnia draws, shared group position and member finalization;
- `trader` owns the 1200/24000 two-level timer, persisted escalating chance, alive-player and
  encounter-first meeting selection, exact trader/llama placement and nonrollback attachment;
- `warden` owns shrieker attribution/listening, shared warning trackers, delayed response,
  warning-four search/finalization and Darkness audience refresh.

## Hostile policy and natural spawning

`spawn_monsters` defaults true. Refresh replaces every level's chunk-source cache with the live
conjunction; false removes only `MONSTER` from natural categories and pauses Patrol/Phantom while
leaving friendly categories, cats and wandering traders reachable. Pearl and reinforcement chance
draws precede the live rule read. Reinforcement is Hard-only, makes at most fifty six-draw candidate
attempts, checks position/rule/player/AABB/collision/liquid in order and does not roll back
attribute charges after the insertion call. Creaking ticker RNG precedes its live gate; portal
policy precedes `nextInt(2000)`.

Natural category order and maxima are fixed to the seven vanilla non-`MISC` categories. Global cap
is `baseMax*spawnableChunkCount/289` with strict comparison; the union count is not the shuffled
candidate count. Persistent/custom-persistent mobs are absent from initial accounting, while
non-mobs can consume global count and biome potential but not local player count. A local attempt
needs at least one nearby nonspectator player below its base maximum. Potential equality passes.

Each admitted position consumes chunk-local X/Z and inclusive minY-through-surface-plus-one Y,
rejects `minY` and conductors, then runs three fixed-Y walks. Provisional attempts are
`ceil(nextFloat*4)`, including zero. Player and respawn exclusion are inclusive at 24 blocks;
reduced-water suppression is strict below `0.98`. Construction failure/non-Mob ends the position
routine, while later mob-gate failure rejects only the candidate. Successful finalization accounts
potential/global/local state after the insertion call regardless of retention. The 83 registered
placements and unregistered fallback remain content-owned inputs; chunk generation uses CREATURE
only, four attempts/member and continues after construction exceptions/nulls.

## Custom spawners

Patrol and Phantom are Overworld-only, post-block/random-tick custom spawners. Their hostile and
own gamerules pause rather than reset nonpersisted countdowns. A fresh Patrol due call stores
11999..13198, while later expiry stores 12000..13199; all later brightness, chance, selected-player,
village, loaded-square and environment failures preserve that schedule. Only the leader failure
ends its group. Leader target draws use the Pillager stream before placement; followers are
nonterminal and insertion is unchecked.

Phantom's corresponding ranges are 1199..2379 and 1200..2380. Scheduling precedes sky/player
failure. Spectators consume no RNG; skylight dimensions enforce darkness five, sea-level equality
and sky visibility. Difficulty uses strict `effective > nextFloat*3` before the clamped-rest
`nextInt(rest) >= 72000` trial. Each successful player samples one position, one group count, and
stacks every member there; null construction skips only that member.

Wandering Trader ignores hostile policy but is paused by `spawn_mobs` at its caller and by its own
rule. Every due outer tick subtracts 1200 from persisted delay. Expiry stores 24000, stores clamped
old-chance-plus-25 before drawing, and admits `nextInt(100) <= oldChance`; failure keeps the
elevated chance. No alive player is a successful return that resets chance without later draws.
Otherwise level RNG selects any alive player, including spectators, and the spawner stream performs
the one-in-ten trial and encounter-first meeting fallback. Trader offsets span `-48..47`, require
twelve empty collision cells and exclude the tagged biome. Two independent llama searches use the
trader placement contract at radius four. All entity insertion results are ignored before sound,
leash, home, target and despawn state.

## Warden warnings and response

Direct players, controlling passengers, projectile owners and item owners can attribute ingress.
The radius-eight vibration listener additionally requires ticking adjacent chunks, the listen tag
and non-shrieking state. A response-capable warning rejects a Warden in the 48-wide AABB, builds the
strict-radius-16 player set and force-adds the attributed player. Any cooldown rejects the entire
set; otherwise the maximum warning increments with Java wrapping then clamp `0..4`, resets to
cooldown 200 and copies all fields to every tracker. Tracker decay occurs when a tick starts at
12000, so ordinary decay is on tick 12001.

A committed shriek uses flags two, delay 90, level event 3007 and a player-attributed game event.
The delayed tick clears with flags three before rechecking the gates. Levels one through three
reply; level four makes twenty Warden attempts. Failed/no spawn consumes three reply offsets;
success consumes none. Constructed candidates establish dig/emerging memory and play the aggravated
sound before later obstruction failure and discard. Successful insertion is unchecked. Either a
reply or success applies a copied 260-tick Darkness instance to Survival/Adventure players strictly
inside forty blocks, refreshing an existing finite effect only through duration 199.

## Validation

`crates/ferrite-gameplay/tests/slices/mobs/mob_001.rs` owns all six source-specified slices. Its
twenty-five tests lock both hostile-rule projections, every direct consumer, category/cap/counting
boundaries, pack and chunk-generation formulas, all three pausable custom-spawner timers, their
distinct RNG streams and nonrollback behavior, persisted trader chance quirks, warning
synchronization/decay, Warden response and Darkness boundaries. `G01-P7-B1` remains responsible for
composing these decisions with Region entity storage, placement, persistence, effects and protocol
projection.
