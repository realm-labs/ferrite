# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-BLOCK-001` — Item blocking resolves angle, blocked amount, durability and retaliation

**Parent:** `ENT-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the complete `minecraft:blocks_attacks` admission, timing, angle, reduction,
requested durability, retaliation, player disable and sound transaction is specified below. The
effective durability loss after enchantment processing is the shared item-durability transaction
owned by `ITM-USE-001`, and every call into the shared velocity transaction is owned by
`ENT-KNOCKBACK-001`; this rule fixes their exact inputs and call positions.

**Applies when:**

`ENT-DAMAGE-001` submits a strictly positive amount to a living entity that may be using a stack
with `minecraft:blocks_attacks`. The component also supplies the generic item-use start, duration
and animation for the official shield.

**Authoritative state:**

Submitted float amount; damage type/tags, source position and direct entity; victim position/head
yaw/use flag/use stack/use hand/use remaining ticks; complete `BlocksAttacks` component;
blocking-stack damage and cooldown; attacker subtype/age/weapon/active stack/attributes; ravager
timers; world and entity RNG streams.

**Transition and ordering:**

Resolve the following server transaction in order.

1. A generic item whose stack has `minecraft:blocks_attacks` starts use in the selected hand and
   returns `CONSUME`; absent a consumable override its use duration is `72000` ticks and its use
   animation is `BLOCK`. `startUsingItem` rejects an empty stack or an already-using entity;
   otherwise it stores the same hand-stack reference and duration, and on the server sets
   using/offhand flags and emits `ITEM_INTERACT_START` only when the stack's
   `use_effects.interact_vibrations` is true. Blocking eligibility is nevertheless delayed as
   described next.
2. `applyItemBlocking` returns zero for a finite negative value, either signed zero, or negative
   infinity. Its JVM `fcmpg` branch also admits NaN, as well as ordinary positive values and
   positive infinity. Fetch `getItemBlockingWith`: it returns the current use-stack reference only
   when the entity is using an item, that stack still exposes the component, and
   `useDuration(stack,entity)-useItemRemaining >= round(blockDelaySeconds*20.0f)`. Otherwise return
   zero.
3. Return zero when the component's optional `bypassedBy` holder set contains the source damage
   type. Independently, return zero when the direct entity is an `AbstractArrow` whose pierce level
   is strictly positive.
4. Compute the horizontal incidence angle. With a source position, form
   `h=normalize((sourceX-victimX,0,sourceZ-victimZ))`, form the victim view vector with pitch `0`
   and current head yaw, then use the double result `acos(h dot view)`. Without a source position
   use double `3.1415927410125732`.
5. Traverse `damageReductions` in list order and float-add each result. One reduction returns zero
   when `angle > 0.017453292f*horizontalBlockingAngle` or when its optional damage-type set does not
   contain the source type; otherwise it returns
   `clamp(base+factor*submittedAmount,0.0f,submittedAmount)`. Every entry receives the same
   submitted amount rather than a diminishing remainder. Clamp the float sum once more to
   `[0.0f,submittedAmount]`; this is `blockedAmount`.
6. Call `hurtBlockingItem` even when the reduction list produced zero. A nonplayer victim returns
   from this helper without a stat or durability mutation. A player victim first gains the blocking
   stack's `ITEM_USED` statistic on the server. Its item-damage function returns zero when
   `blockedAmount<threshold`; otherwise it returns `floor(base+factor*blockedAmount)`. A positive
   result is submitted to `ItemStack#hurtAndBreak` for the used hand's equipment slot. That shared
   transaction may reduce the request through enchantment effects, triggers the durability criterion
   before storing the new damage, and on break shrinks one stack, broadcasts the equipment-break
   event and stops location-dependent enchantments. It does not itself stop use.
7. Retaliation occurs only when `blockedAmount>0`, the source lacks `is_projectile`, and the direct
   entity is a living attacker. Invoke the victim's virtual
   `blockUsingItem(level,attacker,source,submittedAmount)`. The base path invokes the attacker's
   virtual `blockedByItem(victim,source,submittedAmount)`; it does not pass `blockedAmount`.
8. The default attacker reaction calls the shared knockback transaction on the attacker with
   strength double `0.5`, direction `(attackerX-victimX,attackerZ-victimZ)`, the same source and the
   full submitted amount. Three attacker subtypes replace that reaction: an adult hoglin, or a
   non-baby zoglin, calls `HoglinBase.throwTarget(attacker,victim)`; a baby does nothing. A ravager
   does nothing while `roarTick!=0`. Otherwise it consumes one `nextDouble`: below `0.5` it sets
   `stunnedTick=40`, plays `ravager_stunned` at volume/pitch `1`, broadcasts entity event `39`,
   calls `victim.push(ravager)`, and marks the victim's velocity dirty. At or above `0.5` it pushes
   the victim by `(4*dx/max(dx^2+dz^2,0.001),0.2,4*dz/max(dx^2+dz^2,0.001))` and marks that velocity
   dirty.
9. `HoglinBase.throwTarget` first computes `k=attacker.attack_knockback-victim.knockback_resistance`
   and returns without RNG when `k<=0`. Otherwise it consumes, in order, `nextInt(21)-10`,
   `nextFloat`, `nextFloat`; it rotates the normalized attacker-to-victim horizontal vector by the
   first float, scales it by `k*(0.2+0.5*second)`, pushes the victim with that horizontal vector and
   vertical `k*third*0.5`, then marks the victim velocity dirty.
10. A player victim performs the base reaction first and then re-fetches its blocking stack. If
    present, ask the attacker for seconds to disable blocking. The default attacker returns its
    main-hand weapon component's `disable_blocking_for_seconds` only when that exact stack reference
    is also its active item (use stack while using, otherwise main hand; spectators have an empty
    active item). A warden instead always returns `5.0f`. Continue only for a positive value and a
    still-present blocking component. Compute `ticks = round(seconds*disableCooldownScale*20.0f)`
    when the product is positive, otherwise zero. For positive ticks, add a cooldown for the
    blocking stack and stop item use. On the server that stop clears kinetic-use state/using flags,
    emits `ITEM_INTERACT_FINISH` when the snapshotted stack enables interaction vibrations, then
    clears the use stack and remaining ticks. Only afterward play the optional disabled sound.
11. Return `blockedAmount` to `ENT-DAMAGE-001`. Only if its outer hit is fresh, the amount is
    positive, and the snapshotted use stack still supplies the component does the outer transaction
    call `onBlocked`; its optional block sound therefore occurs after requested durability, attacker
    reaction and any disable sound. The later outer criterion/stat/result behavior is exactly
    `ENT-DAMAGE-001`.

**Branches and aborts:**

Nonpositive/NaN amount; no active or mature blocking stack; bypassed damage type; piercing
`AbstractArrow`; missing/coincident source geometry; per-reduction angle/type; empty/multiple
reductions and partial/capped sums; requested durability below/at threshold and item break;
projectile-tagged or nonliving direct source; default, hoglin, zoglin or ravager retaliation;
nonplayer/player victim; weapon inactive/absent, warden override and zero/positive disable duration;
fresh versus cooldown-excess/rejected outer hit.

**Constants and randomness:**

Component codec defaults are delay `0`, disable scale `1`, one reduction
`(90 degrees, any type, base 0, factor 1)`, item damage `(threshold 1, base 0, factor 1)`, and no
bypass/block/disable sound. Official `minecraft:shield` is the only 26.2 item with this component:
delay `0.25` seconds = `5` ticks, default 90-degree/full-damage reduction, bypass
`#minecraft:bypasses_shield`, item damage `(3,1,1)`, disable scale `1`, block sound
`minecraft:item.shield.block`, disable sound `minecraft:item.shield.break`, max damage `336`, and
default use effects `(canSprint=false, interactVibrations=true, speedMultiplier=0.2f)`. Thus a
blocked shield amount below `3` requests zero durability; amount `3` requests `4`; amount `x>=3`
requests `floor(1+x)`. All seven axes have weapon disable duration `5.0` seconds, producing `100`
shield-cooldown ticks. A warden produces the same value without a weapon. Resolution and rounding
consume no RNG. Each present block sound consumes one world `nextFloat` for pitch `0.8+0.4*r` at
volume `1`; each disable sound independently consumes one for the same pitch at volume `0.8`.
Ravager and hoglin-family consumption is exactly the gated order above; effective item durability
may consume the shared enchantment RNG.

**Side effects:**

Item-use state/hand flags/pose and start/finish game events; `ITEM_USED`; requested and effective
durability, criteria, break event and enchantment cleanup; attacker/victim velocity and dirty flag;
ravager stun state/sound/event; blocking-stack cooldown and use termination; disable then block
sounds; outer blocked stat, advancement criteria, damage callbacks and boolean result through
`ENT-DAMAGE-001`.

**Gates:**

Locked item component and damage-type tags; use elapsed time; source geometry/type/pierce; player
versus other living victim; direct living attacker and projectile tag; attacker
subtype/age/roar/attributes/weapon/active item; component sound optionals; outer cooldown/fresh
decision.

**Boundary cases and quirks:**

Exactly five elapsed shield ticks are eligible; four are not. Exactly 90 horizontal degrees block
because only a greater angle fails; coincident horizontal positions normalize to zero, whose dot is
zero and angle is 90 degrees. A missing source position maps to the slightly-above-pi constant and
therefore fails the official 90-degree shield reduction. A submitted NaN passes the initial gate;
with the official shield its reduction, sum and returned blocked amount remain NaN, the player still
gains `ITEM_USED`, item damage floors to zero, retaliation is skipped, and `ENT-DAMAGE-001` treats
the blocked test as false before sanitizing remaining damage to `Float.MAX_VALUE`. Positive infinity
resolves to infinite blocked damage, requests `Integer.MAX_VALUE` durability damage and can break
the shield before retaliation; subtracting it from the submitted infinity then leaves the outer
remaining value NaN, which is sanitized to `Float.MAX_VALUE`. A geometrically/type-mismatched finite
reduction likewise awards `ITEM_USED` for a player because durability handling follows resolution
unconditionally, but requests zero damage and does not retaliate. Piercing or bypass rejection
occurs earlier and awards no blocking-use stat. Nonplayers can block but never execute this helper's
stat/durability branch. A shield can break before retaliation; its same mutable stack reference
remains the outer snapshot, so subsequent component reads and stop-use ordering follow that object
rather than a copied pre-break stack. A fully blocked hit may ultimately return false from the outer
damage call after all blocking side effects.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`,
`net.minecraft.world.item.Item#getUseAnimation(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.Item#getUseDuration(net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.world.entity.LivingEntity#startUsingItem(net.minecraft.world.InteractionHand)`,
`net.minecraft.world.entity.LivingEntity#stopUsingItem()`,
`net.minecraft.world.item.ItemStack#causeUseVibration(net.minecraft.world.entity.Entity,net.minecraft.core.Holder$Reference)`,
`net.minecraft.world.entity.LivingEntity#getItemBlockingWith()`,
`net.minecraft.world.entity.LivingEntity#applyItemBlocking(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.item.component.BlocksAttacks#resolveBlockedDamage(net.minecraft.world.damagesource.DamageSource,float,double)`,
`net.minecraft.world.item.component.BlocksAttacks$DamageReduction#resolve(net.minecraft.world.damagesource.DamageSource,float,double)`,
`net.minecraft.world.item.component.BlocksAttacks#hurtBlockingItem(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity,net.minecraft.world.InteractionHand,float)`,
`net.minecraft.world.item.component.BlocksAttacks$ItemDamageFunction#apply(float)`,
`net.minecraft.world.item.component.BlocksAttacks#disable(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,float,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.component.BlocksAttacks#onBlocked(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.world.entity.LivingEntity#blockUsingItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.LivingEntity#blockedByItem(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.player.Player#blockUsingItem(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.LivingEntity#getSecondsToDisableBlocking()`,
`net.minecraft.world.entity.monster.warden.Warden#getSecondsToDisableBlocking()`,
`net.minecraft.world.entity.monster.Ravager#blockedByItem(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.monster.hoglin.Hoglin#blockedByItem(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
`net.minecraft.world.entity.monster.Zoglin#blockedByItem(net.minecraft.world.entity.LivingEntity,net.minecraft.world.damagesource.DamageSource,float)`,
and
`net.minecraft.world.entity.monster.hoglin.HoglinBase#throwTarget(net.minecraft.world.entity.LivingEntity,net.minecraft.world.entity.LivingEntity)`.
Locked component report `reports/minecraft/components/item/shield.json`; axe component reports under
the same directory; damage tags `data/minecraft/tags/damage_type/bypasses_shield.json` and
`is_projectile.json`.

**Test vectors:**

`EXP-ENT-002`: elapsed ticks `4/5`; signed zero/negative/NaN/positive infinity;
front/side/rear/coincident/missing source and exact 90-degree incidence; bypassed ordinary source,
pierce `0/1`, projectile-tagged living/nonliving direct entity; custom ordered reduction lists with
type filters, negative factors and over-cap sum; shield blocked amounts just below/at/above `3`,
enchanted durability and breaking hit; default attacker, active/inactive axe, using offhand item,
warden, baby/adult hoglin and zoglin, ravager with roar and both RNG outcomes; assert
stat/durability/reaction/cooldown/stop-use/disable-sound/block-sound/outer-result order and RNG
cursor.
