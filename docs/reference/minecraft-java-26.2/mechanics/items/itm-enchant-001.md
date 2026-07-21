# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ENCHANT-001` — Typed enchantment hooks compose in stored and equipment order

**Parent:** `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked source fixes compatibility, iteration/composition, hook RNG ownership and
the complete enchanting-table offer transaction. The vanilla enchantment definitions and effect
objects are locked DataOnly inputs rather than an implementation enum switch.

**Applies when:**

Gameplay invokes a typed enchantment hook on an enchanted item/entity, or an enchanting menu
computes or commits an offer.

**Authoritative state:**

Stack `ENCHANTMENTS` entries and levels, definitions/effect lists/tags, equipment slots, hook
context, entity/server random source, enchantable value, bookshelf count, player enchantment seed,
experience and lapis.

**Transition and ordering:**

Runtime stack iteration reads `ENCHANTMENTS` in component entry order; `STORED_ENCHANTMENTS` is the
book storage/update representation and is not an active runtime hook source. Equipment hooks scan
`EquipmentSlot.VALUES` order and then each stack's entries, admitting only definitions that match
the slot. Value hooks carry a mutable float through every matching effect in effect-list order, so
each effect observes the preceding result. Boolean immunity ORs results but still invokes all
eligible hooks. Post-attack processing visits victim equipment before attacker source main hand.

The typed components cover durability/ammo, block and mob XP, damage immunity/protection/damage,
fall distance, armor effectiveness, knockback, post-attack and piercing, location changed/
deactivation/tick, projectile count/spread/piercing/spawn/hit, repair-with-XP, equipment drop chance,
attributes, fishing, trident, crossbow and the other registry-listed hook types. Predicated effect
lists test then apply in list order. Integer-returning durability, ammo, XP and related hooks use
Java numeric `intValue` truncation; projectile count/spread/piercing and repair results clamp to at
least zero where their helper does so.

Two enchantments are compatible only when they are distinct and neither definition's exclusive set
contains the other. Offer cost from enchantable value `v` and bookshelf count `b` uses
`selected = nextInt(8)+1+(b>>1)+nextInt(b+1)`, with `b` capped only above at `15`; slots yield
`max(selected/3,1)`, `selected*2/3+1`, and `max(selected,b*2)`. Selection adds
`1 + nextInt(v/4+1) + nextInt(v/4+1)`, multiplies by
`1 + (nextFloat+nextFloat-1)*0.15`, rounds and clamps to at least one. For each definition in
registry iteration order, scan levels maximum down to one and retain the first whose inclusive cost
range contains the adjusted cost. Choose the first result by weighted random. Then while
`nextInt(50) <= cost`, filter all remaining candidates against the last selected enchantment, stop
if none remain, weighted-select one, append it and halve cost.

The menu counts valid shelves at the locked offsets, seeds its RNG from the player's enchantment
seed, computes slots `0..2` in order, and zeros a displayed cost below `slot+1`. Each positive clue
rebuilds selection with seed `enchantmentSeed+slot`, then uses the menu RNG to choose a clue entry.
Ordinary books reject non-table enchantments and remove one random selection if the list has more
than one. Commit validates slot, stack, cost, lapis `slot+1`, and both displayed cost and minimum
level unless creative; it recomputes the list, deducts only `slot+1` experience levels through
`onEnchantmentPerformed`, transmutates a book if needed, applies all selected entries, consumes
lapis, awards stat/criterion, replaces the enchant seed, recomputes offers, then emits sound.

**Branches and aborts:**

Absent `ENCHANTABLE`, empty candidates, mismatched slots/predicates, incompatible definitions and
failed menu resources produce no hook/offer/commit. An ordinary book uses the book-specific removal
branch. Creative bypasses experience/lapis sufficiency and lapis consumption as defined by the menu.

**Constants and randomness:**

Offer constants are `8`, `15`, `0.15`, `50` and slot divisors/multipliers above. Item-filtered hooks
use `ServerLevel.random`; entity-filtered hooks use the entity RNG; damage-filtered hooks use victim
RNG; unfiltered helpers use their caller RNG. Even a later-failing continuation consumes its
`nextInt(50)` loop test.

**Side effects:**

Hook-dependent value/entity changes; durability, projectiles and attributes; menu stack/book
transmutation, enchantments, lapis, experience levels, seed, clues, stat/criterion and sound.

**Gates:**

Hook invocation, active component versus stored book component, matching slot, requirements/tags,
level, compatibility, enabled data, menu index/resources and creative mode.

**Boundary cases and quirks:**

Each continuation filters the already-reduced candidate list against the most recently selected
entry, so compatibility with earlier selections remains cumulative. Equipment immunity deliberately
does not short-circuit. Menu displayed cost is not the number of levels deducted: commit deducts
`slot+1`. Negative bookshelf input is not lower-clamped by the cost helper, although the real menu
count is nonnegative.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.item.enchantment.EnchantmentHelper#runIterationOnItem`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#runIterationOnEquipment`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#modifyDamage`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#isImmuneToDamage`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#doPostAttackEffects`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#getEnchantmentCost`,
`net.minecraft.world.item.enchantment.EnchantmentHelper#selectEnchantment`,
`net.minecraft.world.inventory.EnchantmentMenu#slotsChanged`,
`net.minecraft.world.inventory.EnchantmentMenu#clickMenuButton`; locked
`data/minecraft/enchantment/**/*.json`; `EXP-ITM-005`.

**Test vectors:**

Two effects on one stack; two equipment slots; non-short-circuit immunity; victim/attacker order;
int truncation and zero clamp; mutual/one-sided exclusivity; offer costs at `0/1/15` shelves;
single/multiple candidates with RNG trace; book removal; insufficient displayed cost versus
`slot+1`; creative commit and seed refresh.
