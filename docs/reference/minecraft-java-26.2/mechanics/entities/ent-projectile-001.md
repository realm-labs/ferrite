# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-PROJECTILE-001` — Projectile families sweep, deflect and commit hits in subtype-defined order

**Parent:** `ENT-004`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — common launch/owner/deflection geometry and every registered projectile-family
tick/hit subclass are explicit in locked source; item payloads, enchantments and potion/firework
components are locked DataOnly inputs.

**Applies when:**

A projectile-like entity launches, ticks, collides, deflects, expires, returns or is retrieved.

**Authoritative state:**

Subtype, owner reference, `leftOwner`/shot flags, position/velocity/rotation, collision margin,
payload/components, subtype timers/state, random source and hit candidates.

**Transition and ordering:**

Launch normalizes direction, consumes three triangular draws with deviation
`0.0172275*uncertainty`, scales by power, derives rotation, and `shootFromRotation` then adds source
known X/Z movement plus Y only while airborne. Spawn invokes shoot, adds the entity, then runs
projectile-spawn enchantment hooks. The first base tick emits `PROJECTILE_SHOOT`; each tick lazily
checks whether its swept AABB expanded by movement and `1` no longer intersects any pickable member
of the owner's root vehicle.

Ordinary swept collision clips blocks/world border from old position to old+movement, shortens the
segment to a block hit, then scans entity AABBs inflated by a margin
`clamp((tickCount-2)/20,0,0.3)`. The single-hit routine keeps strictly smaller squared distance, so
an equal-distance candidate does not replace the iteration-order winner. A projectile rejects an
unhittable entity and, until it has left, any entity sharing the owner's vehicle. Entity deflection
precedes normal hit: the same deflector cannot reapply until another does. World-border bounce is
subtype-opt-in, reverses, then scales velocity `0.2`. Normal entity hit redirects a redirectable
projectile target before its own subtype callback, then emits projectile-land; block hit invokes the
block callback before that event.

`Projectile#mayBreak` returns true exactly when the projectile type is in
`minecraft:impact_projectiles` and `projectiles_can_break_blocks` is true. Exactly three locked
block callbacks invoke it, always after `mayInteract(level,pos)`: chorus flower destroys itself
with drops and the projectile as breaker; decorated pot writes `cracked=true` with flags `260` then
destroys itself with drops; pointed dripstone additionally requires a thrown trident with speed
strictly greater than `0.6`, then destroys itself with drops. A client-level hit, failed interaction,
missing tag, disabled rule, non-trident dripstone hit or speed equality performs none of those
mutations.

Throwable items apply gravity, then inertia (`0.8` water with four bubbles, otherwise `0.99`), sweep
using the resulting movement, place/rotate/apply blocks/base tick, and only then resolve a still-alive
hit. Gravity is `0.03`, or `0.05` for potions and `0.07` for XP bottles. Snowball deals `3` only to
Blazes and otherwise zero, then event/discard. Egg deals zero, rolls `nextInt(8)` then conditionally
`nextInt(32)` for one/four baby chickens and stops creation when size placement fails. XP bottle
awards `3+nextInt(5)+nextInt(5)` with block normal or reverse-flight direction. Ender pearl sends 32
portal particles, validates the owner, conditionally rolls `0.05` for an endermite, teleports to its
old position with reset velocity/rotation, resets fall/impulse, deals player `5`, and discards. The
endermite branch consumes the `<0.05` draw before requiring the live `spawn_mobs && spawn_monsters`
gate and non-Peaceful difficulty; its exact creation transaction is owned by
`MOB-HOSTILE-GATE-001`. The pearl maintains player chunk tickets and obeys
vanish-on-death/portal gates.

Water potion affects fire/water-sensitive entities inside squared distance `<16` and rehydrates
axolotls; it also dowses the impact-adjacent/opposite/four horizontal blocks. Splash potion moves the
projectile AABB to the hit, inflates `4,2,4`, computes distance to target AABB inflated by the dynamic
margin, and scale `1-sqrt(distanceSq)/4`; instant effects use that scale, duration effects round
`scale*duration*componentScale+0.5` and are added only above `20` ticks. Lingering creates radius `3`,
radius-on-use `-0.5`, duration `600`, wait `10`, and radius-per-tick `-radius/duration`.

Arrows handle in-block detection first. In ground they despawn after `1200` server ticks; a changed
block plus collision-free radius `0.06` releases them with three independent `nextFloat*0.2`
velocity multipliers. In flight water inertia is `0.6`, air `0.99`, gravity `0.05`. They gather all
entity intersections up to the block endpoint, sort by squared distance to entity position, move to
the first location, and process in that order; piercing continues the sweep until block/removal/
deflection and admits at most `pierce+1` IDs. Damage is
`ceil(clamp(speed*modifiedBase,0,Integer.MAX_VALUE))`; critical adds
`nextInt(damage/2+2)`. Successful nonpiercing hit discards; failed damage restores fire, reverses and
scales `0.2`, dropping an allowed arrow and discarding below squared speed `1e-7`. Block hit backs up
`0.05` by movement signs, zeros velocity, sets in-ground/shake `7`, clears crit/pierce and resets hit
sets. Potion arrows lose contents at in-ground time `600`; spectral applies glowing `200`; trident
deals base `8`, permits one target, marks dealt after five in-ground ticks and loyalty returns with
vertical `0.015*level`, velocity `old*0.95 + normalizedDelta*(0.05*level)`, water inertia `0.99`.

Hurting projectiles first add normalized movement times acceleration (`0.1`) and apply inertia
`0.95` air/`0.8` liquid, then sweep/place/base-tick/hit/trail; missing/removed owner or unloaded chunk
discards server-side. Attack deflection restores acceleration `0.1`; other deflection halves it.
Large fireball deals `6` then explodes at stored power; small deals `5`, preserves prior fire on
failed damage and may place fire under mob-griefing; wither skull deals owner `8`/unowned `5`, heals
owner `5` on kill, applies amplifier-1 wither `10s` Normal/`40s` Hard, then power-1 explosion.
Dragon fireball creates its exact duration-600 expanding instant-damage cloud. Wind charges have
zero acceleration/inertia `1`, ignore each other/end crystals, deal `1`, and explode radius `1.2`
(player, first five ticks undeflectable) or `3` (breeze); block explosion is offset `0.25` along the
hit normal and above build height+30 also explodes.

Other catalog families retain their located state machines: firework lifetime is
`10*(1+flight)+nextInt(6)+nextInt(7)`, attached flight acceleration and line-of-sight radius-5 damage
`5+2*explosionCount`; llama spit gravity `0.06`, drag `0.99`, damage `1`; shulker bullet chooses
10/20/30/40/50-step homing legs, deals `4` and levitation `200`; fishing hook is
FLYING→HOOKED/BOBBING with ground expiry `1200`, range squared `1024`, synchronized bite RNG and a
single fishing-loot evaluation on retrieval; eye of ender targets at most horizontal `12` and +`8`,
expires after `>80` and survives with `4/5`; evoker fangs attack at warmup `-8`, deal `6`, and discard
after their 22-life countdown.

**Branches and aborts:**

Owner/vehicle/friendly-fire filters, deflection, world border, portal, unloaded chunk, subtype hit
immunity, piercing history, payload absence, gamerules, difficulty, pickup policy and expiry.

**Constants and randomness:**

All shared/subtype constants and draw sites are stated above; remaining component values come from
the locked item catalog. Geometry is Java double math and candidates retain level iteration order
except arrows' explicit stable distance sort.

**Side effects:**

Damage/effects/knockback/fire/explosions, blocks/clouds/entities/items/XP, projectile state/removal,
owner/stat/criterion/loot state, game events, sounds/particles and tracking updates.

**Gates:**

Entity/block predicates, owner-left state, team/friendly fire, subtype state, feature/components,
world border/portal/chunk, difficulty and gamerules including mob griefing, the impact-projectile
tag plus projectile-block-breaking rule, TNT/explosion and pearl death behavior.

**Boundary cases and quirks:**

Throwable motion applies drag/gravity before sweep, while arrow order differs. Arrow candidate sort
uses entity position distance, not intersection distance. A redirectable projectile hit receives
deflection before the hitter's callback. Splash duration exactly `20` is rejected. Eye of ender is a
projectile-family entity but not a `Projectile` subclass.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.entity.projectile.Projectile`,
`net.minecraft.world.entity.projectile.Projectile#mayBreak(net.minecraft.server.level.ServerLevel)`,
`net.minecraft.world.entity.projectile.ProjectileUtil`,
`net.minecraft.world.entity.projectile.ThrowableProjectile`,
`net.minecraft.world.entity.projectile.arrow.AbstractArrow`,
all classes in `projectile.arrow`, `projectile.throwableitemprojectile`,
`projectile.hurtingprojectile` and `projectile.hurtingprojectile.windcharge`, plus
`FireworkRocketEntity`, `FishingHook`, `ShulkerBullet`, `EyeOfEnder`, `EvokerFangs` and `LlamaSpit`;
`net.minecraft.world.level.block.ChorusFlowerBlock#onProjectileHit(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.phys.BlockHitResult,net.minecraft.world.entity.projectile.Projectile)`,
`net.minecraft.world.level.block.DecoratedPotBlock#onProjectileHit(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.phys.BlockHitResult,net.minecraft.world.entity.projectile.Projectile)`,
`net.minecraft.world.level.block.SpeleothemBlock#onProjectileHit(net.minecraft.world.level.Level,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.phys.BlockHitResult,net.minecraft.world.entity.projectile.Projectile)`;
`EXP-ENT-003`.

**Test vectors:**

Owner overlap/leaving; entity/block and equal-distance ties; repeated/multiple deflectors; every
throwable result RNG boundary; splash distance/duration `20/21`; arrow pierce/block/failed-damage/
despawn; all tag/rule/permission combinations for chorus flower and decorated pot, plus pointed
dripstone with non-trident/trident speed `0.6`/next representable greater value; trident loyalty;
fireball/wind charge gates; firework visibility; fishing retrieval once.
