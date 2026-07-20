# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-USE-001` — Item use separates start, per-tick use, release, and finish

**Parent:** `ITM-001`, `ITM-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — item-family durations, cadence, returned-stack branches, durability RNG, and
cooldown values remain unexpanded.

**Applies when:**

Interaction dispatch reaches an item's use behavior and it starts or performs an action.

**Authoritative state:**

Hand stack/components, active hand and remaining use ticks, cooldowns, player state, target context
and returned interaction result.

**Transition and ordering:**

Invoke context/air use; if immediate, apply returned stack/result now; if consuming, record active
hand and duration; each player tick invoke use-tick behavior at the item's cadence; on release
invoke release behavior with elapsed/remaining duration; on natural completion invoke finish
behavior and install its returned stack. Revalidate that the active stack is compatible before each
stage. Anchors:
`net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`
and `net.minecraft.world.entity.LivingEntity#completeUsingItem()`.

**Branches and aborts:**

Fail/pass/success; instant versus duration use; active stack replaced; player stops; duration
reaches zero; cooldown/feature gate; creative exemption; item returns a container/replacement.
Release and finish are mutually selected by how use ends.

**Constants and randomness:**

Duration and animation are item/component data. Effects, projectile divergence, food outcomes or
durability may consume RNG only in their branch. Tick counters are integers; elapsed calculation
must match the source off-by-one boundaries.

**Side effects:**

Stack count/components/replacement, cooldown, active pose, food/effects, projectile/entity spawn,
durability, statistics/criteria/game events, sounds/particles and inventory synchronization.

**Gates:**

Interaction result, cooldown, hunger/always-edible, hand, feature flags, player abilities, target
conditions and active-stack identity.

**Boundary cases and quirks:**

The stack returned by finish can differ in item type and must replace the correct hand slot.
Interrupting on the last apparent client frame may still be release rather than finish depending on
server tick receipt.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; locators above; tick boundary `EXP-ITM-001`.

**Test vectors:**

Immediate use, full-duration food, release bow at every boundary tick, replace held stack while
using, creative container item, cooldown rejection and simultaneous inventory synchronization.

The source-specified click transaction, transfer primitive, all 25 registered menu layouts,
dedicated controls, synchronization and close behavior are split into
[container transaction leaf rules](README.md#itm-container-001). Recipe lookup, all registered
serializers and the manual crafting commit are split into
[recipe and manual-crafting leaf rules](README.md#itm-recipe-serializer-001).
