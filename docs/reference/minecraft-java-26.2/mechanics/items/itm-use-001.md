# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-USE-001` — Active item use is a stack-revalidated start/tick/release/finish state machine

**Parent:** `ITM-001`, `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes the base component dispatch, active-stack identity rules,
tick boundary, release/finish split, returned-stack installation, remainder and cooldown order, and
consumable cadence. Concrete item/component records are locked DataOnly inputs.

**Applies when:**

Air/context interaction dispatch reaches an item, or a living entity already has an active use.

**Authoritative state:**

Current hand stack, captured use stack, active hand, remaining/captured duration, `IS_USING_ITEM` and
offhand flags, item data components, abilities, cooldowns and server/client side.

**Transition and ordering:**

Base `Item.use` tries `CONSUMABLE`, then swappable `EQUIPPABLE`, then `BLOCKS_ATTACKS`, then
`KINETIC_WEAPON`; blocking and kinetic use call `startUsingItem` and return `CONSUME`, otherwise the
base result is `PASS`. `startUsingItem` ignores an empty stack or an existing active use, captures
the exact hand stack and its duration, sets the hand/use flags, optionally emits the interaction
start vibration, and initializes kinetic-enemy state on the server.

On every active tick, the current hand must have the same item as the captured stack; if so it
replaces the captured reference with the current stack, invokes `onUseTick` at the current remaining
value, then pre-decrements remaining. Reaching zero completes on the server unless the item uses
release. A consumable emits periodic effects only after
`usedTicks > floor(consumeTicks * 0.21875)` and when `remaining % 4 == 0`. Completion requires the
captured stack to equal the current hand stack including components; otherwise it releases. A valid
completion invokes `finishUsingItem`, installs the returned object only when it is a different stack
object, then stops.

Release accepts a current stack with the same item even if components changed, rebinds to it, and
invokes `releaseUsing(remaining)`. Only a `true` item release applies after-use components and may
install its returned object. A release-driven item receives one final `updatingUsingItem` call before
stop. Stop clears kinetic state and flags, optionally emits the finish vibration, empties the
capture, and zeros remaining.

`ItemStack.use` decides whether a use is instant from the pre-dispatch duration. An instant success
applies after-use processing to the success result; duration use defers it. Finish and successful
release copy the pre-use stack, run item behavior, then apply `USE_REMAINDER` followed by
`USE_COOLDOWN`, both read from that copy.

**Branches and aborts:**

Consumables reject a player who cannot eat unless `canAlwaysEat`; positive duration starts use,
while zero duration consumes immediately. `BLOCKS_ATTACKS` starts blocking unconditionally;
blocking becomes effective only when elapsed ticks reach its configured delay. Active item-type
replacement stops; same-item component mutation survives ordinary ticks and release but redirects
natural completion to release. Client completion is permitted only while still marked using.

**Constants and randomness:**

Component seconds convert by Java float multiplication and truncation: `(int)(seconds * 20.0F)`.
Consumable finish emits `16` particles. Its periodic threshold is `0.21875` of duration and cadence
is four remaining ticks. Sound evaluation performs the locked eat/drink random draws in source
order; consume effects, item subclasses and projectiles own any further RNG.

**Side effects:**

Consumable completion emits particles/sound, awards the used-item stat and consume criterion for a
server player, invokes every `ConsumableListener` component in component iteration order, applies
consume effects in list order on the server, emits `EAT` or `DRINK`, and consumes one item unless
materials are infinite. A remainder is returned directly if use emptied the stack; otherwise the
extra remainder callback receives it. Cooldown applies only to players and uses the optional group.

**Gates:**

Interaction result, cooldown at caller dispatch, food eligibility, active hand, empty state,
same-item/equal-stack checks, component predicates, feature enablement and infinite-material ability.

**Boundary cases and quirks:**

The tick callback observes the old remaining count because decrement happens afterward. Equality at
zero is therefore the natural-completion boundary. A stack whose count did not fall receives no use
remainder. A used stack that remains nonempty retains itself and routes the created remainder through
the callback. `getTicksUsingItem` is captured duration minus remaining; it is not a separate counter.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.entity.LivingEntity#startUsingItem(net.minecraft.world.InteractionHand)`,
`net.minecraft.world.entity.LivingEntity#updatingUsingItem()`,
`net.minecraft.world.entity.LivingEntity#updateUsingItem(net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.entity.LivingEntity#completeUsingItem()`,
`net.minecraft.world.entity.LivingEntity#releaseUsingItem()`,
`net.minecraft.world.entity.LivingEntity#stopUsingItem()`,
`net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`,
`net.minecraft.world.item.ItemStack#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`,
`net.minecraft.world.item.ItemStack#finishUsingItem(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.world.item.component.Consumable#onConsume(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.item.component.UseRemainder#convertIntoRemainder`,
`net.minecraft.world.item.component.UseCooldown#apply`; `EXP-ITM-001`.

**Test vectors:**

Zero- and one-tick consumables; cadence threshold on both sides; replace item versus mutate
components; release success/failure; release-driven final tick; returned same/different object;
empty/nonempty/infinite-material remainder; grouped cooldown; delayed blocking boundary.

Container transactions and crafting remain split into their dedicated leaf families in this manual.
