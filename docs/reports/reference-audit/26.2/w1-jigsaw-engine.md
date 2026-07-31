# Minecraft Java 26.2 reference audit — jigsaw engine worker 1

## Scope and provenance

- Worktree: `/Users/mikai/CLionProjects/ferrite-worktrees/w1-jigsaw-engine`
- Branch: `codex/ref-wgen-jigsaw-engine`
- Baseline: `feba9fac70272c8eaa4a87ea10aacb430b34294b`
- Rules: `WGEN-JIGSAW-CORE-001`, `WGEN-JIGSAW-RECORDS-001`,
  `WGEN-JIGSAW-PROCESSORS-001`
- Evidence: locked official server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`,
  bundled data, regenerated reports and `javap` control flow from the inner 26.2 server jar

No Ferrite runtime code or implementation disposition was changed.

## Audit coverage

- Entry points: followed `JigsawStructure`, `JigsawPlacement`, `Placer`, template processing and
  template placement. The existing graph behavior matched; liquid and NBT write boundaries were
  expanded.
- Constants/data: decoded 10 structures, 10 biome tags, six sets, 188 pools and 40 processor lists,
  then checked the generated registries. Locked counts and values matched.
- RNG: separated the structure stream, positional aliases, per-position rules, capped origin stream
  and placement stream, including every conditional draw in the audited branches.
- Branch/abort: checked missing and empty pools, the Empty sentinel, depth, attachment, collision,
  clipping and null processors. Existing source-specified branches matched.
- Mutation/order: checked discovery/priority queues, processor/finalizer order and block/NBT/liquid
  writes. The underdocumented liquid, NBT alias and load-time seed handoffs were corrected.
- Persistence/reload: checked jigsaw piece fields and defaults, junctions, liquid setting and moves.
  Existing claims matched bytecode.
- Cross-rule handoff: followed records into the core, processors into template writes and processed
  NBT into block-entity load. Randomizable versus brushable seed ownership is now explicit.
- Reproduction: expanded the completion and leaf vectors for liquid and NBT alias/load edges.

## Material findings

1. Apply-waterlogging snapshots the live pre-write fluid, reinserts it into compatible placed
   blocks, and repeatedly resolves non-source pending cells from source neighbors in the fixed order
   `UP, NORTH, EAST, SOUTH, WEST`. It never checks down and excludes source positions placed by the
   template. Ignore-waterlogging skips the entire transaction.
2. `append_static` copies configured data only for null input. With non-null input it mutates the
   supplied compound via `merge` and returns the same object. Generic template processing has
   already copied raw NBT, but direct callers can observe the alias.
3. After a successful NBT-bearing template write, a `RandomizableContainer` overwrites
   `LootTableSeed` with one caller-placement `nextLong` immediately before load outside debug edit
   mode. `BrushableBlockEntity` is not randomizable, so trail-ruins append-loot seeds remain the
   position-derived rule result.
4. The exact spawn override category key is `axolotls`; all eight ancient-city full-box and
   trial-chamber piece-box keys are now enumerated explicitly.

## Confirmed censuses

- Pools: 188; weighted entries: 1,198; expanded weight: 4,880; nine fallback IDs.
- Pool elements: 601 legacy single, 527 single, 36 feature, 31 empty and three list.
- Processor lists: 40; top-level processors: 52; nested rules: 164; pool references: 757.
- Registries: 11 processors, six rule tests, three position tests and four NBT modifiers; jigsaw
  structure-piece protocol ID 55.
- Structure records and sets matched every documented pool, height, projection, range, adaptation,
  padding, liquid, spawn override, weight, spacing, separation, salt, frequency and exclusion value.

## Unresolved experiments

No new source-inconclusive behavior was found. `EXP-WGEN-001` remains attached only for the shared
structure-placement/distribution behavior owned outside these three rules; this audit did not claim
an implementation verification result.

## Verification

The exact reference checks were:

```sh
MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
./target/debug/mc-ref reports

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
./target/debug/mc-ref symbols

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
./target/debug/mc-ref experiment verify

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
./target/debug/mc-ref verify --offline

MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java \
MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap \
./target/debug/mc-ref readiness

git diff --check
```

All passed. `reports` regenerated 1,545 official report files, `symbols` validated 2,789 locators
across 952 classes, and `experiment verify` validated 307 definitions. The full offline verifier
classified all 9,078 locked IDs without unclassified or ambiguous entries after the official client
jar's root `version.json` was extracted into its ignored `client-classes` cache. Readiness passed for
both source and behavior-surface readiness.

Rust formatting, Clippy and crate tests were not run because this wave changes reference
documentation and its completion metadata only.
