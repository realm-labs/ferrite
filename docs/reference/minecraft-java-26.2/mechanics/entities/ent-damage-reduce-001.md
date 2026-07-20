# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-DAMAGE-REDUCE-001` — Defense, absorption and health consume the selected cooldown amount

**Parent:** `ENT-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — armor durability and effectiveness, resistance, protection enchantments, witch
modifiers, absorption, health/exhaustion/stats/combat tracking, and every `actuallyHurt`/`hurtArmor`
override in the locked jar are specified below. Generic effective durability loss remains the shared
item transaction owned by `ITM-USE-001`; this rule fixes every request, slot and call position.

**Applies when:**

`ENT-DAMAGE-001` accepts a fresh amount or cooldown excess and calls the victim's virtual
`actuallyHurt(level,source,selectedAmount)`. The selected amount is the finite nonnegative value
produced after blocking/freeze/helmet/nonfinite sanitization, or the finite nonnegative excess over
`lastHurt`.

**Authoritative state:**

Source/type/tags/weapon and exhaustion; victim subtype, attributes,
equipment/components/enchantments/effects, absorption/maximum absorption, health/maximum health,
food exhaustion, stats and combat tracker; animal/camel/armadillo/copper-golem state; wolf armor
damage/crack level; entity tick/fall state and RNG streams used by shared enchantment
durability/value effects.

**Transition and ordering:**

Dispatch the most-derived override, with the common transaction below as its named base call.

1. A player and the common living implementation first recheck virtual source invulnerability and
   return if true. A camel first stands instantly: set pose `STANDING`, emit `ENTITY_ACTION`, and
   set its last pose-change tick to `max(0,gameTime-53)`; it then reaches the animal path. Every
   `Animal` base call sets `inLove=0` before common defense, including a selected zero or a hit
   later absorbed completely. Copper golems and armadillos run their post-defense hooks in steps
   12-13. Wolf armor may replace the common transaction entirely in step 11.
2. Armor is first. When the source has `bypasses_armor`, skip both armor durability and the armor
   formula. Otherwise call the victim's virtual `hurtArmor(source,selectedAmount)` before computing
   reduction. Base living entities do nothing. Players traverse `FEET, LEGS, CHEST, HEAD`; horses
   traverse only `BODY`; wolves traverse only `BODY`. Compute one request
   `q=(int)max(1.0f,selectedAmount/4.0f)`. For every selected slot in that order, request `q`
   durability only when the stack has `equippable.damage_on_hurt`, is damageable, and its optional
   `damage_resistant` type set does not contain the source. Each stack receives the full `q`; it is
   not divided among equipped pieces.
3. Read armor as `floor(armorAttributeDouble)` and toughness as a double-to-float conversion. Let
   `d=2.0f+toughness/4.0f`, `e=clamp(armor-selectedAmount/d,armor*0.2f,20.0f)`, and initial
   effectiveness `r=e/25.0f`. If the source has a weapon stack on a server level, iterate that
   stack's enchantments in stored order and apply every matching `armor_effectiveness` conditional
   value effect sequentially to `r`, then clamp the result to `[0,1]`; otherwise retain the initial
   `r`. Replace the amount by `selectedAmount*(1.0f-r)`. In locked data only `minecraft:breach`
   supplies this component, adding `-0.15f*level` before the final clamp.
4. Magic processing returns immediately with the armor result when the source has
   `bypasses_effects`; this skips both Resistance and protection enchantments. Otherwise, when
   Resistance is present and the source lacks `bypasses_resistance`, set `i=(amplifier+1)*5`,
   `j=25-i`, and replace amount by `max(amount*(float)j/25.0f,0.0f)`. Let `resisted=before-after`.
   When `0<resisted<Float.MAX_VALUE/10`, a server-player victim gains `round(resisted*10)`
   `DAMAGE_RESISTED`; otherwise, and only otherwise, a causing server player gains the same
   `DAMAGE_DEALT_RESISTED` statistic for a nonplayer victim.
5. If the post-Resistance amount does not pass the JVM positive comparison, return `0.0f` from magic
   processing. If the source has `bypasses_enchantments`, return it unchanged. Otherwise iterate
   equipment slots in enum order `MAINHAND, OFFHAND, FEET, LEGS, CHEST, HEAD, BODY, SADDLE`; for
   each nonempty stack, iterate stored enchantments whose slot definition matches that slot, then
   each matching `damage_protection` conditional effect in list order, mutating one float
   accumulator. Locked protection data adds `level` for Protection; `2*level` for the matching fire,
   blast or projectile tag; and `3*level` for Feather Falling's fall tag. Clamp the accumulated
   protection to `[0,20]` and replace amount by `amount*(1-protection/25)`.
6. A witch applies its override after the entire common magic helper: set the amount to zero when
   `source.getEntity()` is that same witch, then multiply by `0.15f` when the source is in
   `witch_resistant_to`. Armor processing and durability have already occurred before either witch
   branch.
7. Snapshot the defended amount as `D` and current absorption as `A`. Compute health damage
   `H=max(D-A,0.0f)`. Set absorption to `A-(D-H)`, clamped by `setAbsorptionAmount` to
   `[0,max_absorption]`; let absorbed `B=D-H`. When `0<B<Float.MAX_VALUE/10`, a player victim gains
   `round(B*10)` `DAMAGE_ABSORBED`. For a nonplayer victim, only a causing server player gains the
   corresponding `DAMAGE_DEALT_ABSORBED`. If `H` is exactly float zero, return now: do not add
   exhaustion, record combat, change health, or emit `ENTITY_DAMAGE`.
8. A player with positive `H` next calls `causeFoodExhaustion(source.exhaustion)`: ability
   invulnerability suppresses it; otherwise the server adds the value and caps exhaustion at
   `40.0f`. Then record combat damage, set health to `clamp(oldHealth-H,0,maxHealth)`, award
   `round(H*10)` `DAMAGE_TAKEN` only when `H<Float.MAX_VALUE/10`, and emit `ENTITY_DAMAGE`.
9. A nonplayer with positive `H` records combat damage, sets the same clamped health, then calls
   `setAbsorptionAmount(currentAbsorption-H)` before emitting `ENTITY_DAMAGE`. For ordinary finite
   inputs this second absorption write is already clamped at zero and changes nothing; its ordering
   is still normative.
10. `CombatTracker#recordDamage` first rechecks status: while taking damage it clears entries and
    leaves combat when dead or when `tickCount-lastDamageTime` is strictly greater than `300` in
    combat or `100` otherwise. It then appends `(source,H,currentFallLocation,(float)fallDistance)`,
    sets `lastDamageTime=tickCount` and `takingDamage=true`. If not already in combat, alive, and
    the source's causing entity is living, it enters combat, sets both start and end to the current
    tick, and calls `onEnterCombat`.
11. A wolf wearing exactly `minecraft:wolf_armor` and hit by a source outside `bypasses_wolf_armor`
    does not call Animal/common defense at all. Request `ceil(selectedAmount)` durability from the
    body armor and absorb the entire hit with no overflow to health, absorption, stats, combat, game
    event, or love reset—even when that request breaks the armor. Compare remaining-durability crack
    levels before and after: remaining fraction `<0.32` is `HIGH`, `<0.69` is `MEDIUM`, `<0.95` is
    `LOW`, otherwise `NONE`; a broken/non-damageable post-stack is `NONE`. On a level change, play
    `wolf_armor_crack` and send 20 armadillo-scute item particles at `(x,y+1,z)` with spreads
    `(0.2,0.1,0.2)` and speed `0.1`.
12. After an armadillo's Animal/common call, return when no-AI or dead/dying. A causing living
    entity sets brain memory `DANGER_DETECTED_RECENTLY=true` with expiry `80`; if the armadillo is
    not panicking, in liquid, leashed, a passenger, or a vehicle, call `rollUp`. That call itself
    does nothing if already scared; otherwise it stops motion, resets love again, emits
    `ENTITY_ACTION`, plays `armadillo_roll`, and enters `ROLLING`. With no causing living entity, a
    source in `panic_environmental_causes` calls `rollOut`, which only acts while scared and emits
    `ENTITY_ACTION`, plays `armadillo_unroll_finish`, and enters `IDLE`.
13. After its common defense call, a copper golem always sets its synchronized state to `IDLE`.
    These post-hooks run after a fully absorbed base hit; the armadillo dead/no-AI gates still
    apply. Camel stand and Animal love reset run before common defense.

**Branches and aborts:**

Virtual invulnerability recheck; player/common and subtype override; wolf armor
present/bypassed/broken; armor bypass, slot-specific durability and resistant items; absent/present
source weapon and Breach; effects/resistance/enchantment bypass tags; Resistance amplifier/stat
ownership; every conditional protection match; witch self/resistant source; absorption
full/partial/none and stat ownership; player ability/exhaustion/stats versus nonplayer; combat
entry/reset; camel/animal/armadillo/copper-golem pre/post hooks.

**Constants and randomness:**

Armor uses float `2`, `/4`, minimum `20%` of armor, cap `20`, and `/25`; Breach is `-0.15*level`.
Resistance uses `(25-5*(amplifier+1))/25`. Protection contributions are `1/2/3` per level as listed,
total cap `20`, hence at most `80%` protection. Equipment request is truncating
`(int)max(1,amount/4)`; wolf armor uses `ceil(amount)`. Stat scale is `10`, upper bound strict
`<3.4028235e37f`; exhaustion cap `40`; combat expiry comparisons are strict `>100/300`; crack
thresholds are strict `<0.32/0.69/0.95`; armadillo memory is `80`. Common arithmetic and locked
Breach/protection effects consume no RNG. Shared effective durability may consume enchantment RNG;
arbitrary data-defined value effects own RNG only when their matched operation requests it.

**Side effects:**

Equipment durability/criteria/break events; Resistance/absorption/damage statistics; absorption,
food exhaustion, combat entries/state, synchronized health and `ENTITY_DAMAGE`; animal love, camel
pose/event, wolf armor sound/particles, armadillo memory/motion/state/sounds/events, copper-golem
state. Health reaching zero is observed by `ENT-DAMAGE-001`, which then invokes `ENT-DEATH-001`;
this leaf does not itself run death protection or drops.

**Gates:**

Locked damage tags/data; entity subtype and invulnerability; attributes/components/equipment slots;
source weapon and enchantment conditions; effects/amplifier; absorption/health/abilities; source
causing entity; animal AI/state/environment; server-only stats, combat and particles.

**Boundary cases and quirks:**

Armor durability is based on the pre-armor selected amount and occurs before every reduction, so
armor may wear even when Resistance, enchantments, absorption, witch self-immunity, or final zero
health damage prevents `ENTITY_DAMAGE`. Equipped armor on ordinary mobs reduces damage but never
wears because base `hurtArmor` is empty; only players, horses and wolves select slots. A selected
zero can still stand a camel, clear Animal love, set a copper golem idle, drive an eligible
armadillo post-hook, or be swallowed by wolf armor, while common durability/health/event work is
skipped. A wolf-armor-breaking hit has no overflow. Full absorption updates absorption and absorbed
stats but skips health, exhaustion, combat and damage event. Resistance multiplies before dividing:
sufficiently large finite amounts—including `Float.MAX_VALUE` at amplifiers 0-3—overflow to positive
infinity; the subsequent absorption subtraction can produce NaN absorption, stats fail their finite
bound, health clamps to zero, and the combat entry retains infinite `H`. Cooldown-excess health uses
only the selected excess, while outer effects/criteria still receive the full current remaining
amount per `ENT-DAMAGE-001`.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.entity.LivingEntity#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.LivingEntity#getDamageAfterArmorAbsorb(net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.LivingEntity#doHurtEquipment(net.minecraft.world.damagesource.DamageSource,float,net.minecraft.world.entity.EquipmentSlot[])`,
`net.minecraft.world.entity.LivingEntity#getDamageAfterMagicAbsorb(net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.damagesource.CombatRules#getDamageAfterAbsorb(net.minecraft.world.entity.LivingEntity,float,net.minecraft.world.damagesource.DamageSource,float,float)`,
`net.minecraft.world.damagesource.CombatRules#getDamageAfterMagicAbsorb(float,float)`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#modifyArmorEffectiveness(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.Entity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#getDamageProtection(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.player.Player#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.player.Player#hurtArmor(net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.Animal#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.wolf.Wolf#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.wolf.Wolf#hurtArmor(net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.armadillo.Armadillo#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.camel.Camel#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.golem.CopperGolem#actuallyHurt(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.animal.equine.Horse#hurtArmor(net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.monster.Witch#getDamageAfterMagicAbsorb(net.minecraft.world.damagesource.DamageSource,float)`,
and
`net.minecraft.world.damagesource.CombatTracker#recordDamage(net.minecraft.world.damagesource.DamageSource,float)`.
Data: item component reports; six defense enchantments; damage-type tag files for all bypass,
resistance, witch and wolf-armor gates.

**Test vectors:**

`EXP-ENT-002`: armor `0/1/20/30`, fractional attribute floor, toughness `0/high`, selected amounts
around durability `/4` truncation, bypass and damage-resistant equipment, every player slot,
ordinary mob armor, horse body and wolf body; Breach levels `0-4`; each protection condition/level,
ordered combinations and cap `19/20/21`; Resistance amplifiers `0/3/4/5`, every bypass combination
and stat-owner permutation; witch self/tag combinations; absorption below/equal/above defended
damage and `Float.MAX_VALUE` overflow; player ability/exhaustion/stats versus nonplayer attacker
stats; combat `100/101/300/301`; zero/full-absorption subtype hooks; wolf armor
normal/crack-threshold/break/bypass; armadillo causing/environmental/no-AI/death/state gates; assert
durability, RNG, stats, state, health and event order.
