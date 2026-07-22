# Game-rule Consumer Inventory

**Locked registry:** `reports/registries.json#minecraft:game_rule` (59 IDs)
**Scope:** the 23 IDs that were still in `unreviewed-game-rule-consumers` after the first
source-proven classification pass
**Primary evidence:** `OFF-SERVER-001`, `OFF-REPORT-001`

This inventory records every direct locked-server bytecode read of each remaining
`GameRules.<FIELD>`. For each field, every server class-file constant pool was searched, the defining
`GameRules` class was excluded, and each candidate was retained only when `javap -p -c` showed an
actual field reference. Accessor callers and the downstream transaction still require semantic
audit; a complete direct-reader list is not by itself a completion claim.

| Game rule | Exact direct reader roots | Disposition after this pass |
|---|---|---|
| `command_block_output` | `BaseCommandBlock$CloseableCommandBlockSource#shouldInformAdmins` | Keep `Unreviewed`: join command-block execution, command-result routing and administrator feedback. |
| `command_blocks_work` | `ServerLevel#isCommandBlockEnabled` | Keep `Unreviewed`: audit every accessor caller, command-block tick/chain admission and projection. |
| `entity_drops` | `VehicleEntity#destroy`, `Painting#dropItem`, `ContainerEntity#chestVehicleDestroyed`, `ItemFrame#dropItem`, `Leashable#tickLeash`, `FallingBlockEntity#tick`, `CopperGolem#turnToStatue` | Keep `Unreviewed`: seven direct readers span independent entity, vehicle, leash and falling-block transactions. |
| `immediate_respawn` | `PlayerList#placeNewPlayer`, `MinecraftServer#onGameRuleChanged` | Classify under `CLI-PLAYER-RULE-001`: the join inversion, live game event, local death-screen/request choice and authoritative-respawn boundary are explicit. |
| `locator_bar` | `ServerWaypointManager#isLocatorBarEnabledFor`, `MinecraftServer#onGameRuleChanged` | Classify under `CLI-PLAYER-RULE-001`: connection creation/removal, per-level callback, clear/rebuild and protocol delegation are explicit. |
| `log_admin_commands` | `CommandSourceStack#broadcastToAdmins` | Keep `Unreviewed`: audit command-source permissions, dedicated settings and feedback fan-out together. |
| `max_block_modifications` | `CloneCommands#clone`, `FillCommand#fillBlocks`, `FillBiomeCommand#fill` | Keep `Unreviewed`: audit volume calculations, loaded bounds, partial failure, mutation and feedback for all three commands. |
| `max_command_forks` | `Commands#executeCommandInContext` | Keep `Unreviewed`: audit execution-context accounting, fork truncation/failure and result propagation. |
| `max_command_sequence_length` | `Commands#executeCommandInContext`, `CommandBlock#executeChain` | Keep `Unreviewed`: audit shared context limits and the independently bounded command-block chain. |
| `max_entity_cramming` | `LivingEntity#pushEntities`, `OozingMobEffect#onMobRemoved` | Classify under `ENT-LIFECYCLE-001`, `ENT-DAMAGE-001` and `ENT-EFFECT-001`: both complete transactions are now explicit. |
| `projectiles_can_break_blocks` | `Projectile#mayBreak`; its only locked callers are `ChorusFlowerBlock#onProjectileHit`, `DecoratedPotBlock#onProjectileHit` and `SpeleothemBlock#onProjectileHit` | Classify under `ENT-PROJECTILE-001`: the complete gate and all three effects are now explicit there. |
| `raids` | `Raids#tick`, `Raids#createOrExtendRaid` | Keep `Unreviewed`: raid ticking alone is ordered by `SIM-PIPELINE-001`, but creation/extension and persistence are not yet closed. |
| `reduced_debug_info` | `PlayerList#placeNewPlayer`, `MinecraftServer#onGameRuleChanged` | Classify under `CLI-PLAYER-RULE-001`: the join snapshot, live entity-event values and local presentation state are explicit. |
| `send_command_feedback` | `CommandSourceStack#broadcastToAdmins`, `ServerPlayer$3#acceptsSuccess`, `GameModeCommand#logGamemodeChange`, `BaseCommandBlock$CloseableCommandBlockSource#acceptsSuccess`, `CommandBlock#setPlacedBy` | Keep `Unreviewed`: five reader roots cover different sources, recipients and placement defaults. |
| `spawn_monsters` | `ServerLevel#isSpawningMonsters`, `MinecraftServer#onGameRuleChanged` | Classify under `MOB-HOSTILE-GATE-001`: compound live reads, cache propagation, every cache consumer and all four direct special-spawn transactions are explicit. |
| `spawn_patrols` | `PatrolSpawner#tick` | Classify under `MOB-PATROL-001`: Overworld installation, timer/cadence, all admission RNG and the complete leader/follower pillager transaction are explicit. |
| `spawn_phantoms` | `PhantomSpawner#tick` | Classify under `MOB-PHANTOM-SPAWN-001`: Overworld installation, pausable cadence, ordered sky/difficulty/insomnia trials and the complete group transaction are explicit. |
| `spawn_wandering_traders` | `WanderingTraderSpawner#tick` | Classify under `MOB-WANDERING-TRADER-001`: both timer layers, persisted escalating chance, player/meeting selection and the complete trader/two-llama transaction are explicit. |
| `spawn_wardens` | `SculkShriekerBlockEntity#canRespond` | Classify under `MOB-WARDEN-SPAWN-001`: shrieker ingress, shared warning persistence/cooldown, delayed response, exact warden attempt and darkness are explicit. |
| `spawner_blocks_work` | `ServerLevel#isSpawnerBlockEnabled`, `TrialSpawner#canSpawnInLevel` | Keep `Unreviewed`: `BLK-TRIAL-SPAWNER-001` closes the trial branch, but ordinary spawner accessor callers remain. |
| `spectators_generate_chunks` | `ChunkMap#skipPlayer` | Keep `Unreviewed`: audit player-distance tracking, ticket changes, mode transitions and unload/projection order. |
| `spread_vines` | `VineBlock#randomTick` | Classify under `BLK-VINE-001`: the sole reader, complete directional growth walk, support, density and branch-local RNG cursor are explicit. |
| `universal_anger` | `NeutralMob#isAngryAtAllPlayers`, `PiglinAi#maybeRetaliate`, `PiglinAi#setAngerTarget`, `PiglinAi#lambda$angerNearbyPiglins$1`, `ResetUniversalAngerTargetGoal#canUse`, `HurtByTargetGoal#canUse` | Keep `Unreviewed`: container leaves cover piglin ingress only; neutral-mob targets/goals remain broader. |

## Closed rules

`Projectile#mayBreak` returns true exactly when the projectile entity type belongs to
`minecraft:impact_projectiles` and `projectiles_can_break_blocks` is true. The locked server has
exactly three callers. `ENT-PROJECTILE-001` now fixes their interaction gate, the chorus-flower and
pointed-dripstone destruction branches, the decorated-pot cracked transition, and corresponding
boundary vectors. That pass left 22 rules in the recoverable fallback.

`max_entity_cramming` has exactly two direct readers. `ENT-LIFECYCLE-001` now fixes the server-only
one-in-four cramming-damage admission without conflating it with unconditional pushing;
`ENT-EFFECT-001` fixes the distinct oozing-removal cap, including the nonpositive-limit behavior.
`immediate_respawn`, `reduced_debug_info` and `locator_bar` each have only the two direct reader
roots shown above. `CLI-PLAYER-RULE-001` now fixes their defaults, join snapshot, live callback and
unmodified-client result while retaining server respawn admission and waypoint wire semantics under
their existing owners.

`spread_vines` has one direct reader. `BLK-VINE-001` now fixes the rule default, zero-RNG disabled
path, one-in-four admission, local-density scan, all six directional branches, support/placement
state and exact RNG cursor.

`spawn_monsters` has two direct reader roots. `MOB-HOSTILE-GATE-001` now fixes its true default,
compound `spawn_mobs` accessor, startup/live per-level cache refresh, natural and all five custom
spawner consumers, and the ender-pearl, zombie, creaking-heart and Nether-portal direct branches.

`spawn_patrols` has one direct reader. `MOB-PATROL-001` now fixes its true default, Overworld-only
installation, pausable/nonpersisted timer, branch-local RNG, player/village/chunk/timeline/biome
gates and exact pillager leader/follower transaction.

`spawn_phantoms` has one direct reader. `MOB-PHANTOM-SPAWN-001` now fixes its true default,
Overworld-only installation, pausable/nonpersisted timer, global and per-player sky gates, strict
difficulty/rest RNG and exact shared-position group transaction.

`spawn_wandering_traders` has one direct reader. `MOB-WANDERING-TRADER-001` now fixes its true
default, Overworld-only installation, pausable two-level cadence, persisted delay/chance mutation,
inclusive chance quirk, player/meeting/candidate selection and exact trader/two-llama transaction.

`spawn_wardens` has one direct reader. `MOB-WARDEN-SPAWN-001` now fixes its true default,
summoning-capable shrieker provenance, player attribution, shared persisted warning/cooldown state,
90-tick nonrollback response, exact triggered-warden candidate transaction and darkness audience.
The other 12 rules remain in the recoverable fallback.

## Reproduction

1. Expand the locked named server JAR beneath ignored `target/mc-reference/26.2/` storage.
2. For a registry ID, map its snake-case name to the corresponding `GameRules` static field.
3. Search all class files for the field-name constant, then retain only candidates whose
   `javap -p -c` output contains a `GameRules.<FIELD>` field reference.
4. Inspect every direct reader and every accessor/callback caller; compare the complete transaction
   with a source-specified leaf before moving the ID out of `Unreviewed`.
5. Run `mc-ref query game_rule minecraft:<id>`, catalog coverage/readiness and full offline
   verification after each classification batch.
