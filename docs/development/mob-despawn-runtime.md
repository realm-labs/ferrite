# MOB-003 Despawn Runtime

`G01-P7-S008` implements `MOB-DESPAWN-001` as a protocol-neutral Region transition. The owning
Region supplies root-entity state, the nearest nonspectator player observation, category, squared
3D distance, subtype persistence/removal policy and the reached mob-stream draw.

## Invocation and ordering

The server invokes the transition for non-frozen, non-removed root mobs before current-chunk
entity-ticking admission. Valid passengers are not checked independently. Peaceful incompatibility
discards first, before persistence or distance. Stored or custom persistence resets
`noActionTime` to zero and exits. With neither, nearest-player lookup has unlimited range and
excludes spectators; no player preserves both inactivity and RNG.

Hard distance is strict above squared 128 blocks, except `WATER_AMBIENT` uses squared 64. A true
subtype removal policy discards, but this is deliberately not an early return. The soft expression
still evaluates: only `noActionTime > 600` consumes `nextInt(800)`, and a zero draw then evaluates
strictly beyond squared 32 before calling the subtype policy. Consequently an already hard-
discarded mob can call that policy twice. Any failed soft expression resets inactivity only when
distance is strictly below squared 32; equality does neither. Effective AI increments inactivity
before goal/Brain work. Discard does not produce death damage, loot or XP.

## Persistence and removal catalog

Base custom persistence is passenger or leashed. Fish/axolotl add bucket origin, Nautilus adds
tame state, sulfur cube adds body item or bucket origin, Enderman adds a carried block and Raider
adds current raid. Stored persistence remains a separate serialized fact used by commands,
item pickup, age locking and subtypes.

Base removal permits distance despawn. Animal, golem, Villager, Wandering Trader, Allay and Warden
use the never policy, with exact Chicken, Cat and Ocelot overrides. Fish require neither bucket
origin nor custom name; Nautilus always permits removal after its custom-persistence gate. Raider
refuses during a raid. Patrolling mobs permit removal when not patrolling or strictly beyond
squared 128. Piglin observes the stored flag. Hoglin, camel husk and zombie horse always permit it.
Zombie Villager requires no conversion and zero villager XP.

## Validation

`crates/ferrite-gameplay/tests/slices/mobs/mob_003.rs` owns the source-specified slice. Its eleven
tests lock invocation order; Peaceful/persistence/no-player exits; all strict hard/soft/timer
boundaries; RNG and double-policy-call behavior; effective-AI increment; base/custom persistence;
and the exhaustive audited subtype policy families. `G01-P7-B1` remains responsible for composing
the result with Region entity removal and cleanup.
