# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-LOOT-001` — Loot is generated from a context and consumed exactly once by its caller

**Parent:** `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — caller context construction, pool/function ordering, RNG consumption, and
every special loot hook remain unexpanded.

**Applies when:**

A block/entity/container/gameplay event invokes a loot table.

**Authoritative state:**

Selected table ID, loot context parameter set, luck, tool/components/enchantments,
killer/source/origin, table RNG/seed policy and destination consumer.

**Transition and ordering:**

Construct the table-specific context; validate required parameters; evaluate pools in data order,
rolls/conditions/functions and nested entries with the supplied random source; normalize/split
generated stacks as the API requires; pass each stack once to the caller, which spawns, inserts, or
stores it. Data at `data/minecraft/loot_table/**/*.json` is normative and queryable.

**Branches and aborts:**

Missing/invalid required context; conditions fail; zero rolls; empty entry; explosion decay;
player/tool/enchantment predicate; nested table; destination full. Empty generation is successful
evaluation, not an error.

**Constants and randomness:**

Every number provider and weighted selection consumes the supplied RNG at its evaluation site.
Integer/floating providers and stack splitting retain their native rounding. A table with an
explicit seed uses that path; otherwise caller RNG determines results.

**Side effects:**

Generated stacks, item entities/container contents, XP through separate functions/callers,
criteria/statistics and later merge/pickup behavior. Loot evaluation itself must not silently insert
into a player inventory.

**Gates:**

Caller event, gamerules such as block/entity drops, context predicates, data-pack table,
difficulty/biome/dimension predicates and destination capacity.

**Boundary cases and quirks:**

Re-evaluating after a partial consumer failure duplicates loot; generate once then consume the
produced sequence. Explosion decay and surviving explosion are context-driven.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`, `OFF-DATA-001`; exact data through catalog/query; RNG sequence
`EXP-ITM-004`.

**Test vectors:**

Same seeded context twice; missing required parameter; fortune/silk/explosion contexts; nested
table; oversized count splitting; full container consumer; ensure a caller retry does not
regenerate.
