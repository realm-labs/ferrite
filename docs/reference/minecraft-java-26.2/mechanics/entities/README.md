# Entities mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`ENT-LIFECYCLE-001`](ent-lifecycle-001.md)

Entity insertion, ticking, passenger traversal, transfer, and removal have explicit ownership

### [`ENT-DAMAGE-001`](ent-damage-001.md)

Damage is a gated pipeline from damage source to health/death transition

### [`ENT-BLOCK-001`](ent-block-001.md)

Item blocking resolves angle, blocked amount, durability and retaliation

### [`ENT-DAMAGE-REDUCE-001`](ent-damage-reduce-001.md)

Defense, absorption and health consume the selected cooldown amount

### [`ENT-KNOCKBACK-001`](ent-knockback-001.md)

Damage direction, resistance and subtype rules commit velocity

### [`ENT-DEATH-001`](ent-death-001.md)

Death protection, death entry, drops and timed removal form one transaction

### [`ENT-PROJECTILE-001`](ent-projectile-001.md)

Projectile ticks sweep from old to new position and resolve the first accepted hit

### [`ENT-VEHICLE-001`](ent-vehicle-001.md)

Vehicle control, physics, collision, and passenger placement are server-owned

### [`ENT-ENTITY-DROPS-001`](ent-entity-drops-001.md)

Entity drops gate seven differently placed itemization branches

### [`ENT-EFFECT-001`](ent-effect-001.md)

Status effects merge, tick, expire, and expose attributes in a defined lifecycle

### [`ENT-BAT-001`](ent-bat-001.md)

Bats alternate between ceiling rest and transient-target flight under exact spawn and wake gates

### [`ENT-GIANT-001`](ent-giant-001.md)

Giants are goal-free Monsters with latent oversized combat attributes and no baseline spawn selector

### [`ENT-ENDERMITE-001`](ent-endermite-001.md)

Endermites expire by persisted lifetime and enter baseline worlds through player Ender Pearls

### [`ENT-GLOW-SQUID-001`](ent-glow-squid-001.md)

Glow Squids combine ageable squid propulsion with a synchronized post-hit darkness clock

### [`ENT-BLAZE-001`](ent-blaze-001.md)

Blazes hover, charge and fire a retained-state three-projectile volley

### [`ENT-BOGGED-001`](ent-bogged-001.md)

Bogged retain one-way shearing state and fire slow poison-arrow volleys

### [`ENT-BREEZE-001`](ent-breeze-001.md)

Breezes cycle slide, shot and long-jump memories around explosive wind charges

### [`ENT-SPIDER-001`](ent-spider-001.md)

Spiders climb, abandon bright fights and finalize into shared-effect packs or skeleton jockeys

### [`ENT-COD-001`](ent-cod-001.md)

Cod school through transient leader links and become persistent after bucket release

### [`ENT-SALMON-001`](ent-salmon-001.md)

Salmon randomize three synchronized sizes before forming five-member schools

### [`ENT-TROPICAL-FISH-001`](ent-tropical-fish-001.md)

Tropical Fish share common variants in eight-member schools but rare variants spawn alone

### [`ENT-PUFFERFISH-001`](ent-pufferfish-001.md)

Pufferfish inflate around scary living entities and poison successful Mob and Player contacts

### [`ENT-TADPOLE-001`](ent-tadpole-001.md)

Tadpoles combine fish and Brain AI until age, feeding or loaded state converts them to Frogs

### [`ENT-SQUID-001`](ent-squid-001.md)

Squids pulse through water and emit thirty ink packets only after admitted Mob-attributed damage

### [`ENT-DOLPHIN-001`](ent-dolphin-001.md)

Dolphins trade Fish for treasure searches while balancing air, moisture, swimmer grace and item play

### [`ENT-ELDER-GUARDIAN-001`](ent-elder-guardian-001.md)

Elder Guardians anchor Monuments, charge a synchronized beam and pulse Mining Fatigue

### [`ENT-EVOKER-001`](ent-evoker-001.md)

Evokers arbitrate Vex, fang and Wololo spells while raids and Mansions provide baseline production

### [`ENT-GHAST-001`](ent-ghast-001.md)

Ghasts collision-sweep floating destinations, charge Large Fireballs and admit reflected kills

### [`ENT-GUARDIAN-001`](ent-guardian-001.md)

Guardians oscillate through Water, retaliate with stationary thorns and charge synchronized beams

### [`ENT-ILLUSIONER-001`](ent-illusioner-001.md)

Illusioners project client-only mirror images, blind each new target once and fire spawn-issued Bows

### [`ENT-IRON-GOLEM-001`](ent-iron-golem-001.md)

Iron Golems never despawn, split player-created targeting and crack as their health falls

### [`ENT-SLIME-FAMILY-001`](ent-slime-family-001.md)

Cube mobs derive every attribute from one synchronized size and split into two to four children

### [`ENT-PARCHED-001`](ent-parched-001.md)

Parched fire slow Weakness arrows they cannot receive themselves and ride Camel Husks

### [`ENT-PHANTOM-001`](ent-phantom-001.md)

Phantom size drives flight, swoop combat and client projection

### [`ENT-PIGLIN-BRUTE-001`](ent-piglin-brute-001.md)

Piglin Brutes bind HOME-centered brain combat to bastion production and zombification

### [`ENT-PILLAGER-001`](ent-pillager-001.md)

Pillagers couple crossbow state, patrol and raid production to a five-slot inventory

### [`ENT-SHULKER-001`](ent-shulker-001.md)

Shulkers bind shell expansion and surface attachment to teleporting Bullet duplication

### [`ENT-SKELETON-001`](ent-skeleton-001.md)

Skeletons switch Bow and melee combat and convert to Strays in Powder Snow
