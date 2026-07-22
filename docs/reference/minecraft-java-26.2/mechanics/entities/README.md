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
