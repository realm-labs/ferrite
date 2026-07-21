# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ADVANCEMENT-001` — Advancement listeners mutate a grouped requirement matrix exactly once per transition

**Parent:** `ITM-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — listener registration/removal, criterion idempotence, requirement evaluation,
reward order, visibility dirtiness, persistence and packet flushing are explicit in locked source;
trigger predicates and definitions are locked registry/DataOnly inputs.

**Applies when:**

A player trigger fires, a command awards/revokes a named criterion, advancement data loads/reloads,
or dirty progress is flushed to the client.

**Authoritative state:**

Definition criteria and ordered requirement groups, per-criterion obtained instants, per-player
progress map, trigger-key map, visible set, changed-progress set, dirty roots, rewards and selected tab.

**Transition and ordering:**

Starting progress deletes criterion records not named by the current requirement matrix and creates
missing named records. Completion is false for an empty requirement list; otherwise every outer
group must contain at least one completed named criterion (AND across groups, OR within a group).
Loading applies known saved progress, marks it changed and its root dirty; definitions with no
criteria are then automatically awarded and their rewards explicitly granted; finally listeners
are registered for every incomplete criterion.

Award snapshots `wasDone`, grants only an existing incomplete criterion, unregisters listeners now
done or rendered unnecessary by whole-advancement completion, and marks progress changed. On the
first incomplete→complete transition it grants rewards, then conditionally broadcasts the display
announcement; independently, that transition marks the root for visibility recomputation. Revoke
clears only an existing completed criterion, re-registers currently incomplete listeners, marks
progress changed, and marks the root only for a done→incomplete transition. Repeating either
operation without a criterion-state change returns false and has no reward/listener side effect.

Reward order is experience, then loot tables in list order, then one inventory broadcast if any
loot inserted, recipes, then optional function. Reward loot uses advancement-reward parameters
`THIS_ENTITY` and `ORIGIN`; each generated stack first tries player inventory, otherwise drops with
no pickup delay and player target. Each successful insertion consumes two player-random floats for
pickup pitch and sets the later broadcast flag. The optional function runs as the player with
suppressed output and gamemaster permission.

On flush, dirty roots recompute visibility before changed visible progress is collected. Roots and
progress dirtiness are cleared, and a packet is sent only if added, removed or progress maps are
nonempty. Its reset flag is true only for the first flush after load/reload. Saving writes only
advancements with at least one completed criterion; criterion storage retains obtained instants.

**Branches and aborts:**

Unknown saved definitions are warned and ignored. Missing/already matching criteria are no-ops.
Announcements require a display with announce-chat plus `showAdvancementMessages`. Missing reward
loot/function entries follow their registry/function resolution behavior. Only visible changed
progress enters the packet.

**Constants and randomness:**

The persistence data-fix fallback version is `1343`. Criterion instants use the system clock for
storage/order but event dispatch controls gameplay order. Reward loot uses its loot context RNG;
successful inserted stacks consume pickup-pitch RNG in reward iteration order.

**Side effects:**

Progress timestamps, trigger maps, dirty/visible sets, XP, inventory or item entities, pickup sound,
recipes, reward function, chat announcement, advancement update packets and selected-tab packets.

**Gates:**

Registered trigger and predicate, known definition/criterion, requirement transition, display and
gamerule, loot/inventory admission, recipe/function resolution and current visibility.

**Boundary cases and quirks:**

Requirements may complete without every criterion. Revoking one satisfied member of an OR group
does not make the advancement incomplete if another member remains satisfied. Automatic empty-
criteria definitions call `award("\")`, which cannot mutate an empty matrix, and then grant rewards
directly on each load path. Reload clears triggers/progress/visibility/dirtiness/tab and repeats load.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.server.PlayerAdvancements#load`,
`net.minecraft.server.PlayerAdvancements#award`,
`net.minecraft.server.PlayerAdvancements#revoke`,
`net.minecraft.server.PlayerAdvancements#registerListeners`,
`net.minecraft.server.PlayerAdvancements#unregisterListeners`,
`net.minecraft.server.PlayerAdvancements#flushDirty`,
`net.minecraft.advancements.AdvancementProgress#update`,
`net.minecraft.advancements.AdvancementRequirements#test`,
`net.minecraft.advancements.AdvancementRewards#grant`; locked
`data/minecraft/advancement/**/*.json`; `EXP-ITM-006`.

**Test vectors:**

Empty, all-of, any-of and two-by-two matrices; repeated award/revoke; revoke redundant OR member;
reward function re-entrancy; successful and overflow loot; chat gamerule; invisible changed progress;
first/second flush; removed definition and online reload.
