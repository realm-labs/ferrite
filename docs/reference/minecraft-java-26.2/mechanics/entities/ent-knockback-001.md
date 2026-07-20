# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-KNOCKBACK-001` — Damage direction, resistance and subtype rules commit velocity

**Parent:** `ENT-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the fresh-damage call gate, every direction provider and five/six-argument
override in the locked jar, resistance and coincident-direction arithmetic, sulfur-cube archetype
transform, velocity dirtying and server-player indication are specified below. Player attack
selection remains owned by its combat slice; this rule fixes the shared velocity transaction that it
calls.

**Applies when:**

`ENT-DAMAGE-001` reaches its fresh-hit postprocessing and the damage type lacks `no_knockback`, or
another specified caller invokes either shared `LivingEntity#knockback` entry point.

**Authoritative state:**

Damage source/type/direct and causing entity/explicit source position; victim subtype, position,
bounding box, current velocity, on-ground state, yaw, `knockback_resistance`, RNG and velocity-sync
flag; direct projectile position/velocity; blocked flag and full post-blocking amount; sulfur-cube
body item, matching archetype order, knockback/sound settings and attacker's eye/position/look
vector; server-player connection and hurt direction.

**Transition and ordering:**

The damage-owned path runs after the fresh hit's block event/damage event and optional `markHurt`,
and before lethal protection, sounds and criteria.

1. `ENT-DAMAGE-001` skips this entire rule when the damage type is in `no_knockback`; otherwise it
   calls `dealDefaultKnockback(source,remaining,blocked)` even when `remaining` is zero after a full
   block. A cooldown-excess hit is not fresh and therefore does not call it.
2. Initialize horizontal direction `(x,z)=(0,0)`. If `source.directEntity` is a `Projectile`, call
   that projectile's virtual direction provider and negate both returned components. The base
   projectile returns its current `(velocity.x,velocity.z)`, so the later subtraction pushes along
   its travel direction. `FireworkRocketEntity` and every `AbstractThrownPotion` instead return
   `(victim.x-projectile.x,victim.z-projectile.z)`, so negation plus later subtraction pushes away
   from the projectile's current position. These are the only locked overrides.
3. With no direct projectile, obtain `source.getSourcePosition()`: an explicit damage-source
   position wins, otherwise the current direct-entity position is returned, otherwise it is absent.
   When present set `(x,z)=(sourceX-victimX,sourceZ-victimZ)`; when absent retain zero.
4. Dispatch the virtual five-argument `knockback` with strength double equal to widened float
   `0.4f`, `(x,z)`, the source and full `remaining`. A Creaking returns without velocity mutation
   when `canMove()` is false. The Ender Dragon returns while its current phase is sitting. Otherwise
   their overrides and the inherited implementation delegate to the virtual six-argument entry with
   its final boolean fixed to `false`.
5. The common six-argument transaction computes `k=strength*(1-knockback_resistance)` in double. If
   `k<=0`, return without dirtying velocity or consuming RNG. Otherwise set the living entity's
   velocity-sync flag. While `x*x+z*z` is strictly less than widened float `1.0e-5f`, replace `x`
   with `(nextDouble()-nextDouble())*0.01` and then `z` with another two draws times `0.01`; test
   again and repeat. Normalize `(x,0,z)` and scale it by `k`. From snapshotted old velocity `V`,
   commit `(V.x/2-push.x, min(0.4,V.y/2+k), V.z/2-push.z)` while on ground, or
   `(V.x/2-push.x,V.y,V.z/2-push.z)` while airborne. The common implementation does not read its
   source, amount or final boolean parameters.
6. A sulfur cube replaces step 5 only when `source.getEntity()` is non-null and its body slot is
   nonempty; otherwise it invokes the common transaction. Its active settings are reset to defaults
   `(horizontal,vertical)=(0.33,0.06)` and regular hit sound whenever body equipment changes, then
   every archetype matching the new item is visited in registry iteration order; each contributes
   its attribute modifiers and the last match supplies knockback and sound settings. The locked
   last-match values are:

| archetype | horizontal `H0` | vertical `V0` | `minecraft:entity.sulfur_cube.<suffix>` |
| --- | ---: | ---: | --- |
| `bouncy` | `0.4125` | `0.105` | `bouncy.hit` |
| `explosive` | `0.4125` | `0.09` | `explosive.hit` |
| `fast_flat` | `0.9125` | `0.09` | `fast_flat.hit` |
| `fast_sliding` | `0.6625` | `0.09` | `fast_sliding.hit` |
| `high_resistance` | `0.4125` | `0.09` | `high_resistance.hit` |
| `hot` | `0.4125` | `0.09` | `hot.hit` |
| `light` | `0.4125` | `0.18` | `light.hit` |
| `regular` | `0.4125` | `0.09` | `regular.hit` |
| `slow_bouncy` | `0.4125` | `0.24` | `slow_bouncy.hit` |
| `slow_flat` | `0.4125` | `0.105` | `slow_flat.hit` |
| `slow_sliding` | `0.4125` | `0.09` | `slow_sliding.hit` |
| `sticky` | `0.4125` | `0.09` | `sticky.hit` |

7. For that sulfur-cube special path, cast input `(x,z)` to a float pair `D`. Let `A` be the causing
   entity's eye position, `L=normalize(attackerLook)`, `C` the cube bounding-box center and
   `T=normalize(C-A)`. Compute signed horizontal angle
   `a=(float)atan2(L.x*T.z-L.z*T.x,L.x*T.x+L.z*T.z)` and set `D=rotate(D,(float)(1.6f*a))`, where
   rotation uses the locked float sine/cosine pair.
8. Transfer power for vertical aim. Let `h=0.5f*cubeHeight`, `U=normalize(C+(0,h,0)-A)` and
   `B=normalize(C-(0,h,0)-A)`. Compute `f=(float)clampedMap(L.y,U.y,B.y,-1,1)`, then `t=abs(f*0.5f)`
   and negate `t` only when `f<0`. Set `(H,V)=(H0*(1-t),V0*(1+t))`.
9. Rotate that power pair for relative elevation. With `Q=cubePosition-attackerPosition`, let
   `b=(float)atan2(-Q.y,horizontalLength(Q))`, then `(H,V)=rotate((H,V),(float)(-0.8f*b))`. Compute
   `m=max(H0>0 ? abs(H)/H0 : 0, V0>0 ? abs(V)/V0 : 0)`; if `m>1`, divide both by `m`. This preserves
   the original horizontal/vertical power envelope after rotation.
10. Let `p=sqrt(amount)*(finalBoolean ? (float)strength*0.25f : 1.0f)`. Multiply both powers by `p`,
    then by float `(1-knockback_resistance)`. Set the velocity-sync flag unconditionally. Clamp
    `H*0.4f` and `V` independently to `[-128,128]`; normalize `(D.x,0,D.y)`, returning a zero
    direction when its length is below widened float `1.0e-5f`, and commit
    `(oldX-normal.x*H, oldY+V*1.2, oldZ-normal.z*H)`. Play the active archetype's `hit_sound`. This
    path does not halve old horizontal velocity, cap grounded vertical velocity, return at
    resistance `1`, or consume coincident-direction RNG.
11. After the five-argument call returns, `dealDefaultKnockback` invokes `indicateDamage(x,z)` only
    when `blocked` is false. The base living method is a no-op. `ServerPlayer` stores float
    `hurtDir=(float)(Mth.atan2(z,x)*57.2957763671875-yaw)` and immediately sends
    `ClientboundHurtAnimationPacket` built from that player. This indication still occurs when
    common resistance returned early or a five-argument subtype gate suppressed velocity.
12. The shared six-argument entry is also called directly for positive player-attack extra
    knockback, with caller-provided strength, yaw-derived direction, source, attack amount and
    boolean. Direct dispatch bypasses the Creaking/dragon five-argument gates and damage indication
    but reaches the sulfur-cube override. The attack slice owns when those calls occur and the
    attacker's subsequent `(0.6,1,0.6)` velocity damping; implementations must not substitute the
    five-argument wrapper.

**Branches and aborts:**

Fresh versus cooldown-excess hit; `no_knockback`; direct base/firework/thrown-potion projectile
versus nonprojectile; explicit/direct/missing source position; Creaking movable and dragon phase
gates; resistance `k<=0`; grounded/airborne; each coincident-direction retry; blocked indication;
server player versus other living; sulfur cube causing entity/body-item gate, zero/multiple
archetype matches, look/hit/elevation transforms, final boolean and sound.

**Constants and randomness:**

Default strength is widened float `0.4f`; resistance registry range is `[-2,1]`, so common effective
strength is normally `[0,1.2000000178813934]`. The retry compares horizontal length squared with
widened float `9.999999747378752e-6`, whereas vector normalization compares length with that same
number. Each candidate component scale is `0.01`, horizontal old velocity divisor is `2`, and the
ground vertical cap is double `0.4`. A rejected retry consumes exactly four victim `nextDouble`
calls in x-pair then z-pair order; accepted input consumes none. Sulfur constants are
horizontal-angle `1.6`, vertical-aim `0.5`, elevation `0.8`, direct-call boolean factor `0.25`,
horizontal output factor `0.4`, vertical factor `1.2` and clamps `±128`; that path consumes no RNG.

**Side effects:**

Victim velocity and velocity-sync state; sulfur-cube hit sound; server-player `hurtDir` and one
immediate hurt-animation packet. This transaction emits no game event and does not itself alter
health, cooldown, damage attribution or damage boolean result.

**Gates:**

Fresh damage and `no_knockback`; source geometry/direct projectile subtype; victim resistance,
ground state and subtype; sulfur body equipment, causing entity and locked archetype/tag data;
server-player connection; caller choice of five- versus six-argument entry.

**Boundary cases and quirks:**

A fully blocked fresh hit still gives an ordinary victim the same `0.4f` base knockback because
common arithmetic ignores amount and blocked state; blocking only suppresses `indicateDamage`. A
missing source or zero-horizontal-velocity ordinary projectile enters the RNG retry, whereas
firework/potion position overrides can avoid it. Resistance `1` prevents common velocity dirtying
but not an unblocked server-player hurt packet; resistance `-2` triples common strength. The sulfur
special path at resistance `1` still marks velocity dirty and plays its hit sound with zero computed
power. On the default damage path the inherited five-argument wrapper always supplies final boolean
`false`, even for a blocked hit; the boolean scaling is reachable through direct six-argument
callers. A sitting dragon or immobile Creaking can still receive direct six-argument player-attack
knockback. Sulfur direction can be zero without RNG, leaving horizontal velocity unchanged while
vertical power and sound still apply.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.entity.LivingEntity#dealDefaultKnockback(net.minecraft.world.damagesource.DamageSource,float,boolean)`,
both
`net.minecraft.world.entity.LivingEntity#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float)`
and
`net.minecraft.world.entity.LivingEntity#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float,boolean)`,
`net.minecraft.world.entity.projectile.Projectile#calculateHorizontalHurtKnockbackDirection(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.projectile.FireworkRocketEntity#calculateHorizontalHurtKnockbackDirection(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.projectile.throwableitemprojectile.AbstractThrownPotion#calculateHorizontalHurtKnockbackDirection(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.server.level.ServerPlayer#indicateDamage(double,double)`,
`net.minecraft.world.entity.monster.creaking.Creaking#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.monster.cubemob.SulfurCube#knockback(double,double,double,net.minecraft.world.damagesource.DamageSource,float,boolean)`,
its three angle/power helpers and `collectEquipmentChanges(java.util.Map)`, and
`net.minecraft.world.entity.player.Player#causeExtraKnockback(net.minecraft.world.entity.Entity,float,net.minecraft.world.phys.Vec3,net.minecraft.world.damagesource.DamageSource,float,boolean)`.
Data: `data/minecraft/tags/damage_type/no_knockback.json`, all
`data/minecraft/sulfur_cube_archetype/*.json`, and their item tags.

**Test vectors:**

`EXP-ENT-002`: fresh/cooldown-excess and no-knockback; source absent/explicit/direct entity;
ordinary projectile travel in every quadrant, zero velocity, firework and splash/lingering potion
positions; direction squared below/equal/above `1e-5` with scripted retry RNG; resistance `-2/0/1`,
ground/air and signed old vertical velocity; full block with zero remaining; server-player
packet/hurtDir; Creaking movable/frozen and dragon sitting/flying through five- and six-argument
calls; sulfur cube empty/body, null/non-null causing entity, every archetype, multiple-match order,
horizontal/vertical/elevation angles, amount `0/1/4`, both booleans, resistance `-2/1`, clamp
boundaries and zero direction. Assert RNG cursor, exact float/double velocity, dirty flag, sound and
packet order.
