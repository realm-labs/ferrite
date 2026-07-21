# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-LOOT-001` — Loot evaluation is an ordered, guarded producer consumed once by its caller

**Parent:** `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the shared evaluator, pool selection, function nesting, recursion guard, stack
normalization and container fill algorithm are explicit in locked source. Registered codecs and all
vanilla loot JSON are locked DataOnly inputs; the event-specific context and destination remain
owned by the invoking gameplay leaf.

**Applies when:**

A caller has selected a loot table and built its declared parameter-set context.

**Authoritative state:**

Table/pool/entry/function lists in data order, parameter set and values, luck, random source or
seed/random-sequence selection, visited-entry set, feature set and output consumer/container.

**Transition and ordering:**

Raw evaluation first pushes a breadcrumb for this table. A duplicate logs an infinite-loop warning
and produces nothing. Otherwise table functions wrap the ultimate consumer, then pools run in list
order, after which the breadcrumb is popped. Each pool tests its conditions as ordered `allOf`; a
failure skips rolls. Roll count is
`rolls.getInt(context) + floor(bonusRolls.getFloat(context) * luck)`.

For every roll, entries expand in list order. Expanded entries with current weight `<= 0` are
dropped; positive entries are retained and their weights summed. Zero candidates produce nothing.
One candidate is selected without a weighted RNG draw. With multiple candidates, draw
`nextInt(totalWeight)`, subtract each candidate's current `getWeight(luck)` in retained order, and
select the first negative remainder. The selected entry creates stacks through pool functions;
those outputs then pass through the enclosing table functions. Conditions, entries, number
providers and functions use their registered codec-dispatched semantics and retain JSON list order.

Normal rather than raw output applies the stack splitter. Feature-disabled stacks vanish. A count
strictly below maximum passes through as the same stack; a count equal to or above maximum is
emitted as consecutive maximum-sized copies followed by the remainder. The caller must generate
once and consume each emitted stack once; insertion, spawning, XP and retry policy are not implicit
table effects.

Container `fill` generates once, collects empty slot indices ascending and shuffles them, then
removes empty generated stacks and separates counts above one. While spare slots exceed the number
of final plus splittable stacks, it randomly selects a splittable stack, splits a random count in
`[1,count/2]`, and independently uses one boolean for each still-splittable half to requeue it.
Remaining splittables are appended and the result is shuffled. Outputs take shuffled empty slots
from the list end; exhaustion logs overfill and stops.

**Branches and aborts:**

Invalid/missing context is rejected when its parameter set is constructed or validated. Runtime
empty paths include recursion, failed conditions, nonpositive rolls/weights, entry expansion to
nothing and disabled output. Nested tables share the visited set and context RNG. A full or partly
full destination does not authorize re-evaluation.

**Constants and randomness:**

Optional table seed `0` means randomize; an explicit random source/seed otherwise participates in
`LootContext.Builder` selection, with the table's optional random sequence used by the no-override
path. All provider/effect draws occur at their codec implementation's call site. The single-entry
pool optimization consumes no weighted-selection draw. Fill consumes slot shuffle, split selection,
split count, two conditional booleans and final shuffle in the stated order.

**Side effects:**

The evaluator emits stacks and warnings only. Caller-owned leaves define gamerule admission,
context construction, item insertion/spawn, recipe/stat/criterion effects and what happens when a
destination refuses a stack.

**Gates:**

Caller event and gamerules, table existence, context-key set, data conditions, feature enablement,
entry weight and destination admission.

**Boundary cases and quirks:**

A maximum-sized stack enters the copying branch, unlike a stack one below maximum. Table functions
are outside pool functions. Weight is queried once during expansion and again during subtraction;
implementations must preserve that call pattern. Container filling writes an empty generated stack
only if one survives to the assignment loop, though the current splitter removes empties first.

**Evidence:**

`OFF-SERVER-001`, `OFF-DATA-001`;
`net.minecraft.world.level.storage.loot.LootTable#getRandomItemsRaw`,
`net.minecraft.world.level.storage.loot.LootTable#createStackSplitter`,
`net.minecraft.world.level.storage.loot.LootTable#fill`,
`net.minecraft.world.level.storage.loot.LootTable#shuffleAndSplitItems`,
`net.minecraft.world.level.storage.loot.LootPool#addRandomItems`,
`net.minecraft.world.level.storage.loot.LootPool#addRandomItem`; locked registries
`minecraft:loot_condition_type`, `minecraft:loot_function_type`,
`minecraft:loot_pool_entry_type`, `minecraft:loot_number_provider_type` and
`minecraft:slot_source_type`; `EXP-ITM-004`.

**Test vectors:**

Recursive table; failed first/middle condition; zero/one/multiple candidate pools; changing dynamic
weights; seeded nested table; count `max-1`, `max`, and `max+1`; disabled item; partially empty
container with repeatable split/shuffle trace; consumer refusal without regeneration.
