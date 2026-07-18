# Mob spawning and AI leaf rules

## Leaf rule `MOB-SPAWN-001` — Natural spawning is a category-cap, chunk, position, and mob-rule pipeline

**Parent:** `MOB-001`, `MOB-002`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — cap arithmetic, candidate selection, pack termination, structure overrides, and per-species spawn rules remain unexpanded.  <br>
**Applies when:** The server chunk source runs natural spawning for eligible chunks and mob categories.  
**Authoritative state:** Eligible/ticking chunks, non-spectator players, per-category counts/caps, spawn potentials/biome structure data, difficulty, gamerules, local light/fluid/block state and RNG.  
**Transition and ordering:** Build the spawn state/counts; for each eligible chunk and category present in the caller-supplied category list and below its scaled cap, choose candidate positions and biome/structure spawn entries; perform pack attempts; validate distance from players/world spawn, category placement, collision, light and entity-specific spawn rules; create/finalize/add accepted mobs; update counts so later attempts see them. Anchor: `net.minecraft.world.level.NaturalSpawner#spawnForChunk(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.chunk.LevelChunk,net.minecraft.world.level.NaturalSpawner$SpawnState,java.util.List)`.  
**Branches and aborts:** Category cap met; no eligible players/chunk; gamerule false; peaceful; invalid Y/block/fluid/light/collision; too near/far; weighted entry fails; pack budget reached; entity finalization/addition rejected. Failed attempts consume only the RNG reached by that branch.  
**Constants and randomness:** Category base caps, eligible chunk scaling, attempt/pack limits and distance thresholds are source constants. Weighted selection and candidate offsets consume the level RNG in control-flow order. Exact constants/RNG trace: `EXP-MOB-001`.  
**Side effects:** Entity creation/finalization/equipment, group data, jockey/passenger entities, category counts, game events and later tracking. Failed attempts do not reserve cap.  
**Gates:** `doMobSpawning`, difficulty, chunk block/entity ticking eligibility, player spectator status/distance, category cap, biome/structure spawn data, placement rules, local conditions and entity feature flags.  
**Boundary cases and quirks:** Caps are global-per-level scaled by eligible chunks, not per chunk. Spawn chunks and simulation distance affect eligibility differently. Some mobs use special spawners outside this pipeline.  
**Evidence:** `Confirmed` pipeline; exact iteration/RNG `Cross-checked`; `OFF-SERVER-001`, `OFF-DATA-001`; locator above; `EXP-MOB-001`.  
**Test vectors:** One/two players with overlapping and separate eligible chunks; cap boundary; spectator only; peaceful; biome/structure override; fail collision then succeed; fixed seed trace of pack attempts.

## Leaf rule `MOB-AI-001` — Mob AI arbitrates goals, navigation, controls, senses, and memory on entity ticks

**Parent:** `MOB-004`, `MOB-005`  
**FidelityClass:** `EquivalentPlayerVisibleBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — generic schedulers are located, but per-species memories, sensors, activities, goals, navigation gates, and permitted route divergence remain unexpanded.  <br>
**Applies when:** A mob is entity-ticking and not in a state that suppresses ordinary AI.  
**Authoritative state:** Goal selectors/flags/priorities, brain activities/memories/sensors, navigation path, move/look/jump controls, target, attributes, leash and mob-specific timers.  
**Transition and ordering:** Update ambient/mob timers and sensing/brain at their scheduled cadence; stop running goals that cannot continue; evaluate eligible goals and start only those whose control flags can be acquired against higher-priority running goals; tick running goals; navigation advances path; controls translate desired movement/look/jump into entity inputs before travel. Species may use Brain behaviors, GoalSelector, or both.  
**Branches and aborts:** `NoAI`; dead/removed; inactive chunk; passenger; stunned/sleeping; sensor cadence not due; memory absent/expired; goal use/continue false; control flag conflict; path unavailable/stuck; target invalid.  
**Constants and randomness:** Goal priorities are integer and lower numeric value has precedence. Sensor/behavior intervals, memory expiry and path tolerances are species/source data. Goal reevaluation and many behaviors consume RNG at species-defined sites; exact cadence is `EXP-MOB-002`.  
**Side effects:** Target/memory/path, movement and rotation, block interaction, item use, attacks, sounds/game events, breeding/taming state and spawned entities/items.  
**Gates:** Entity ticking, `NoAI`, goals/brain predicates, senses/line of sight/range, mobGriefing where blocks change, difficulty, time/weather and species state.  
**Boundary cases and quirks:** Navigation computes intent; collision/travel still decides actual motion. Goals with disjoint control flags can run concurrently. Memory visibility is not equivalent to a live line-of-sight test every tick.  
**Evidence:** `Confirmed` arbitration structure; species cadences require per-family depth; `OFF-SERVER-001`; `GoalSelector`/`Brain` class families; `EXP-MOB-002`.  
**Test vectors:** Competing MOVE goals of different priority; concurrent LOOK goal; target disappears; path blocked after compute; `NoAI` toggle; unload/reload; Brain memory expires at exact tick.

## Leaf rule `MOB-DESPAWN-001` — Persistence and distance checks choose immediate removal, random removal, or retention

**Parent:** `MOB-003`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — exact thresholds, random cadence, and every persistence override remain unexpanded.  <br>
**Applies when:** A mob's server tick reaches despawn checking.  
**Authoritative state:** Persistence-required flag, custom persistence conditions, nearest eligible player distance, category/type despawn distances, `NoAI`/special flags, difficulty and RNG.  
**Transition and ordering:** If peaceful removal applies, discard through that path; otherwise test persistence; find nearest relevant player; if beyond hard distance and type may despawn, discard immediately; if beyond random distance and inactivity/random check succeeds, discard; if near enough, reset no-action timer.  
**Branches and aborts:** No player; persistent because named/tamed/leashed/passenger/picked-up or subtype rule; type cannot despawn; hard distance; random-distance roll; near reset; special dimension/difficulty removal.  
**Constants and randomness:** Distances and random interval/chance are mob-category/type methods, measured by squared distance. Random despawn consumes mob RNG only when that branch is reached. Exact thresholds and no-player behavior are `EXP-MOB-003`.  
**Side effects:** Removal/untracking without death loot, reset inactivity timer, passenger/leash cleanup and client removal.  
**Gates:** Entity ticking, persistence, nearest eligible non-spectator player, type despawn policy, distance, difficulty and special states.  
**Boundary cases and quirks:** Despawn is discard, not death. Horizontal and 3D distance must follow the source calculation. A loaded but non-ticking mob does not run this check.  
**Evidence:** `Confirmed` branch model; thresholds `Cross-checked`; `OFF-SERVER-001`; `Mob#checkDespawn()` family; `EXP-MOB-003`.  
**Test vectors:** Exact squared-distance boundaries, named/tamed/leashed/passenger, no players, spectator only, peaceful hostile, inactive chunk and fixed-RNG random despawn.

## Leaf rule `MOB-BREED-001` — Love, mate selection, child creation, cooldown, and ownership inheritance commit together

**Parent:** `MOB-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — species eligibility, inheritance, taming chances, ownership, and special side effects remain unexpanded.  <br>
**Applies when:** A breedable/tameable mob is fed, enters love, finds a compatible mate, or completes breeding.  
**Authoritative state:** Age/cooldown, love timer and cause player, mate compatibility, species variant/genetics, tame/owner state, child entity and gamerules.  
**Transition and ordering:** Item interaction validates food/age/state and consumes item under ability rules; set love state; AI selects compatible mate and approaches; on completion create species child, apply parent/variant/owner rules, set both parent cooldown ages and clear love, add child, award player/stat/criterion/XP according to branch. Taming uses its own item/RNG attempt and owner assignment but shares interaction authority.  
**Branches and aborts:** Baby/cooldown; not food; not compatible/same invalid entity; path/partner lost; spawn child null/rejected; `mobGriefing` for food pickup rather than direct feeding; tame roll fails/succeeds; already tamed owner interaction.  
**Constants and randomness:** Love and age durations, XP range, tame chance and variant inheritance are species/source constants. RNG is consumed at child-variant/tame/XP sites only after prerequisites. Exact per-family values are `EXP-MOB-004`.  
**Side effects:** Item decrement, hearts/smoke, love/cooldown/age/owner, child/XP entities, statistics/criteria, sounds/game events and AI memories.  
**Gates:** Species food tag, age, health/state, mate compatibility, ownership, player abilities, entity-add validity, gamerules for related environmental behavior and difficulty where species checks it.  
**Boundary cases and quirks:** Feeding into love and successful child creation are separate transitions; item can be consumed even if no mate is found. Ownership inheritance is species-specific, not universal.  
**Evidence:** `Confirmed` generic lifecycle; species inheritance/chances require family rules; `OFF-SERVER-001`; `Animal`/`TamableAnimal` families; `EXP-MOB-004`.  
**Test vectors:** Adult/baby/cooldown; compatible/incompatible variants; mate removed at completion; full entity rejection; creative item; tame fail/success fixed RNG; owner versus non-owner interaction.
