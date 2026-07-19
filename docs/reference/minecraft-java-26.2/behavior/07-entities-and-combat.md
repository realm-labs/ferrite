# 07 — Entities, Vehicles, Projectiles, and Combat

An “entity” is a dynamic object with server-owned identity and lifecycle. Concrete entity types, damage types, effects, and loot data parameterize these generic rules.

## `ENT-001` Only added, entity-ticking entities enter lifecycle tick

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.server.level.ServerLevel#addFreshEntity(net.minecraft.world.entity.Entity)`; `net.minecraft.server.level.ServerLevel#tickNonPassenger(net.minecraft.world.entity.Entity)`; `net.minecraft.world.entity.Entity#tick()`; `net.minecraft.world.entity.Entity#remove(net.minecraft.world.entity.Entity$RemovalReason)`; `COM-WIKI-ENT-001`
- **Applies when:** An entity is in a server dimension's entity manager and its position has entity-ticking activity.
- **Behavior and timing:** A new entity receives identity and joins the world before ticking. Each eligible level tick calls root non-passengers once, then recursively handles passengers. `baseTick`-class work updates shared fire, portal, fluid, and similar state before concrete AI/physics. Remove/discard sets a removal reason and excludes it from later ticking, tracking, and persistence.
- **Boundaries and quirks:** An entity spawned during iteration must not go backward and receive an extra full tick in the same phase. Death state differs from removed state. Players, multipart entities, persistent projectiles, and cross-dimension entities have special manager paths.
- **Verification owner (`ENT-LIFECYCLE-001`, `ITM-BOOKSHELF-001`, `BLK-COPPER-GOLEM-STATUE-001`; `EXP-ENT-*`, `EXP-ITM-010`, `EXP-BLK-008`):** The concrete leaves fix bookshelf item-entity callers plus statue restoration/conversion admission and already-mutated-source boundaries. Lock first/last tick for other spawn/remove-during-ticking, UUID collision, and re-add ordering after load.

## `ENT-002` Riding forms an ordered tree ticked by its root vehicle

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.Entity#startRiding(net.minecraft.world.entity.Entity,boolean,boolean)`; `net.minecraft.world.entity.Entity#stopRiding()`; `net.minecraft.world.entity.Entity#rideTick()`; `net.minecraft.world.entity.Entity#positionRider(net.minecraft.world.entity.Entity)`; `net.minecraft.server.level.ServerLevel#tickPassenger(net.minecraft.world.entity.Entity,net.minecraft.world.entity.Entity)`
- **Applies when:** An entity is a direct or indirect passenger of another entity.
- **Behavior and timing:** Successful riding creates an acyclic parent-child relation. The world ticks only the root as an ordinary entity, after which it recursively invokes passenger `rideTick` in list order and positions passengers. Dismount first breaks the relation, then selects a safe position and pose. Removal or teleport must handle the whole tree consistently.
- **Boundaries and quirks:** Player control, multiple passengers, shoulder entities, forced riding, and vehicle death have concrete branches. A passenger cannot also receive a second full root tick.
- **Verification owner (`ENT-LIFECYCLE-001`; `EXP-ENT-*`):** Lock nested-passenger order, switching mounts during tick, chunk unload, and dismount collision candidates.

## `ENT-003` Pushing and vehicle physics extend generic collision by type

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`; generic pushing and concrete vehicle trajectories remain open
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`; `net.minecraft.world.entity.Entity#push(net.minecraft.world.entity.Entity)`; `net.minecraft.world.entity.vehicle.boat.AbstractBoat#tick()`; `net.minecraft.world.entity.vehicle.minecart.AbstractMinecart#tick()`; `COM-WIKI-ENT-001`
- **Applies when:** Pushable entities overlap or a boat/minecart advances as a root vehicle.
- **Behavior and timing:** Generic movement first clips against blocks and the border. Entity push changes both velocities along horizontal separation when collision is allowed. Boats then derive water/land state, buoyancy, and paddle input; minecarts use rail shape, slopes, power/braking, and derailed branches. Passenger positions refresh after vehicle movement.
- **Boundaries and quirks:** Cramming, team collision rules, vehicle-to-vehicle contact, minecart experiments, and client interpolation alter outcomes. The default baseline must not enable experimental minecart changes.
- **Verification owner (`ENT-VEHICLE-001`; `EXP-ENT-004`):** Concrete boat/minecart constants, entity traversal order, and simultaneous multi-entity pushing need source-derived trajectory vectors, so this aggregate rule remains `Cross-checked`.

## `ENT-004` A projectile selects the nearest hit along this tick's motion and may deflect

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.projectile.Projectile#tick()`; `net.minecraft.world.entity.projectile.Projectile#shoot(double,double,double,float,float)`; `net.minecraft.world.entity.projectile.Projectile#hitTargetOrDeflectSelf(net.minecraft.world.phys.HitResult)`; `net.minecraft.world.entity.projectile.Projectile#deflect(net.minecraft.world.entity.projectile.ProjectileDeflection,net.minecraft.world.entity.Entity,net.minecraft.world.entity.EntityReference,boolean)`; `COM-WIKI-ENT-001`
- **Applies when:** A `Projectile` or subclass has been fired and ticks in the server entity phase.
- **Behavior and timing:** `shoot` initializes velocity from direction, speed, and inaccuracy. Each tick compares block and eligible-entity intersections from old position to candidate new position and chooses the nearest `HitResult`. It gives target/world deflection a chance, otherwise invokes block/entity hit callbacks, then applies concrete position, rotation, drag, gravity, and survival logic.
- **Boundaries and quirks:** Owner/passenger tree may be ignored before the projectile leaves its collision range. Portals, border, piercing, multiple hits, and high-speed cross-chunk travel are subclass extensions. Endpoint-only collision is incompatible tunneling.
- **Verification owner (`ENT-PROJECTILE-001`; `EXP-ENT-003`):** Lock block/entity ties, multiple targets in one tick, remaining displacement after deflection, and unloaded-chunk edges.

## `ENT-005` Damage passes through ordered defense layers before health is committed

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#hurtServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`; `net.minecraft.world.entity.LivingEntity#applyItemBlocking(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`; `net.minecraft.world.entity.LivingEntity#getDamageAfterArmorAbsorb(net.minecraft.world.damagesource.DamageSource,float)`; `net.minecraft.world.entity.LivingEntity#getDamageAfterMagicAbsorb(net.minecraft.world.damagesource.DamageSource,float)`; `net.minecraft.world.entity.LivingEntity#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`; `COM-WIKI-ENT-001`
- **Applies when:** The server submits a `DamageSource` and base amount to a living entity.
- **Behavior and timing:** `ENT-DAMAGE-001` fixes server-player/player/base immunity wrappers, difficulty and signed/nonfinite amount handling, fire/freeze/helmet transforms, 20-tick cooldown and excess-hit selection, attribution, outer events/effects/criteria and return semantics. `ENT-BLOCK-001` fixes active-use delay, incidence/type reductions, requested durability, attacker retaliation, player disable and sound ordering. `ENT-DAMAGE-REDUCE-001` fixes armor/toughness and wear, Resistance/protection, absorption, health/exhaustion/stats/combat tracking, plus every subtype hook. `ENT-KNOCKBACK-001` fixes projectile/source direction, resistance, coincident-direction RNG, velocity mutation, player indication and the Creaking, Ender Dragon and sulfur-cube overrides. Damage-type tags, item components, effects, attributes, enchantments and sulfur-cube archetype data explicitly select each layer.
- **Boundaries and quirks:** A nonplayer zero can establish cooldown and return true; a player rejects exact zero after difficulty scaling. A larger cooldown hit reduces health by only its excess while effects and criteria receive the full current remaining. A rejected weaker hit can already have woken the entity or damaged blocking/helmet items. Fully blocked fresh hits can emit blocked side effects and criteria yet return false.
- **Verification owner (`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`; `EXP-ENT-002`):** Admission/cooldown, blocking, defense/health and the complete damage-triggered velocity transaction are source-specified. Lethal protection and death remain owned by `ENT-DEATH-001`; no placeholder multiplier may cross that call boundary.

## `ENT-006` Status effects merge by type and expire on server ticks

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.entity.LivingEntity#addEffect(net.minecraft.world.effect.MobEffectInstance,net.minecraft.world.entity.Entity)`; `net.minecraft.world.entity.LivingEntity#tickEffects()`; `net.minecraft.world.effect.MobEffectInstance#update(net.minecraft.world.effect.MobEffectInstance)`; `net.minecraft.world.effect.MobEffectInstance#tickServer(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,java.lang.Runnable)`; `COM-WIKI-ENT-001`
- **Applies when:** A living entity gains, refreshes, removes, or is currently affected by a mob effect.
- **Behavior and timing:** Two instances of one effect type do not run as fully independent public entries. `update` selects current and hidden effects from amplifier, duration, ambient, and visibility rules. Server tick invokes effect logic at its permitted interval and decrements finite duration. Expiry restores a hidden effect or removes the entry, updating attribute modifiers and client-visible state.
- **Boundaries and quirks:** Instant effects, infinite duration, immunity, milk/command removal, death, and dimension transfer have separate callbacks. Particle/icon visibility does not decide whether the server effect applies.
- **Verification owner (`ENT-EFFECT-001`, `ENV-GEYSER-001`; `EXP-ENT-005`, `EXP-ENV-005`):** The geyser leaf fixes its exact 80-tick ambient nausea ingress and geometric gates. Build generic merge fixtures for strong-short over weak-long, hidden chains, infinite effects, and multiple additions in one tick.

## `ENT-007` Lethal damage checks death protection before death and drop lifecycle

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.entity.LivingEntity#checkTotemDeathProtection(net.minecraft.world.damagesource.DamageSource)`; `net.minecraft.world.entity.LivingEntity#die(net.minecraft.world.damagesource.DamageSource)`; `net.minecraft.world.entity.LivingEntity#tickDeath()`; `net.minecraft.world.entity.LivingEntity#dropAllDeathLoot(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`; `net.minecraft.world.entity.LivingEntity#dropExperience(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.Entity)`
- **Applies when:** Committed damage leaves health in the lethal range.
- **Behavior and timing:** `ENT-DEATH-001` fixes the main-before-offhand protection selection, copied-stack criteria/effects and event `35`; ordinary killer score/callback, death event, loot/custom/equipment/XP/wither-rose, event `3` and pose order; the distinct server-player transaction; and common, Creaking and Ender Dragon death timers. Loot-pool algorithms remain owned by `ITM-LOOT-001`, but their death context and emitted-stack consumer are fixed here.
- **Boundaries and quirks:** `mob_drops` does not gate fox-held drops, player inventory drops or player XP; `keep_inventory` zeroes player XP as well as preserving inventory. Successful zombie-villager conversion suppresses the victim death event/drop block while still allowing byte `3` and the dying pose. Common removal occurs on death tick 20, heart teardown after tick 45 and dragon removal at tick 200. Unload is never death.
- **Verification owner (`ENT-DEATH-001`; `EXP-ENT-002`):** Source-specified for protection, killer callbacks, every registered override, loot context, equipment/inventory/XP spawning, server-player effects and all three removal timelines. Black-box work is regression coverage, not an unresolved specification dependency. `ENT-DAMAGE-001` fixes the lethal call position.

## `ENT-008` Teleport is a state transition with target dimension, pose, velocity, and passenger policy

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.entity.Entity#teleport(net.minecraft.world.level.portal.TeleportTransition)`; `net.minecraft.world.entity.Entity#teleportSameDimension(net.minecraft.server.level.ServerLevel,net.minecraft.world.level.portal.TeleportTransition)`; `net.minecraft.world.entity.Entity#teleportCrossDimension(net.minecraft.server.level.ServerLevel,net.minecraft.server.level.ServerLevel,net.minecraft.world.level.portal.TeleportTransition)`; `net.minecraft.world.entity.Entity#teleportPassengers()`
- **Applies when:** A command, portal, ender pearl, or mechanic submits a `TeleportTransition`.
- **Behavior and timing:** Same-dimension teleport updates authoritative position/rotation/relative components and synchronizes. Cross-dimension conversion creates/transfers entity state in the destination, removes the source-dimension instance, and invokes a post-transition callback. Velocity, passengers, and portal cooldown follow transition/entity policy; they are not universally zeroed or preserved.
- **Boundaries and quirks:** Players have a confirmation path; non-player cross-dimension transfer may yield a destination instance. Sleeping, leash, passenger tree, unavailable target chunks, and world border need atomic handling.
- **Verification owner (`ENT-LIFECYCLE-001`; `EXP-ENT-*`):** Build a black-box matrix for every relative flag, cross-dimension identity, passenger policy, and target-load failure. This remains `Cross-checked`.
