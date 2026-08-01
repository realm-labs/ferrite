# Minecraft Java 26.2 Reference Audit — Wave 1, Worker 2: Jigsaw Content

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Baseline: `feba9fac70272c8eaa4a87ea10aacb430b34294b`
- Version lock: Minecraft Java `26.2`, server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`,
  client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`
- Scope: reference correction only. No Ferrite runtime or implementation-disposition file was
  changed, and this audit does not mark implementation `Verified`.

## Audited leaves

- `WGEN-JIGSAW-ANCIENT-CITY-001`
- `WGEN-JIGSAW-BASTION-001`
- `WGEN-JIGSAW-OUTPOST-001`
- `WGEN-JIGSAW-TRAIL-RUINS-001`
- `WGEN-JIGSAW-TRIAL-CHAMBERS-001`
- `WGEN-JIGSAW-VILLAGES-001`

Generic pool expansion, connector matching, collision, projection and single/list/legacy placement
remain owned by `WGEN-JIGSAW-CORE-001`; shared processor semantics remain owned by
`WGEN-JIGSAW-PROCESSORS-001`. This audit did not duplicate those engines in the content leaves.

## Findings

1. The trial-chamber physical census was internally consistent at four barrels and 29
   randomizable-container cells, but its prose incorrectly described four corridor-loot barrels, two
   intersection-loot barrels and one disposal barrel. Direct decoding of the four official template
   inputs proves the exact split is one corridor-loot barrel (`decor/barrel`), two intersection-loot
   barrels (`intersection_2` and `intersection_3`) and one no-table disposal barrel
   (`decor/disposal`). The leaf and its completion selector now agree with the locked NBT.
2. `net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`
   injects `LootTableSeed` and consumes one caller `nextLong` for every successfully written
   `net.minecraft.world.RandomizableContainer`, including fixed/no-table inventories.
   `net.minecraft.world.RandomizableContainer#tryLoadLootTable` accepts that seed even with a null
   table. `net.minecraft.world.RandomizableContainer#trySaveLootTable` emits a seed tag only when
   the table key is nonnull and the seed is nonzero; a present table with an omitted seed reloads
   through the zero default. The leaves now distinguish placement-stream consumption from
   save/reload persistence:
   - ancient city: 15 chest draws, 14 persisted deferred table/seed values, one fixed-apple no-table
     seed omitted on save;
   - trial chambers: 29 container draws, 25 persisted deferred table/seed values, four
     fixed/no-table seeds omitted on save;
   - villages: 80 chest/barrel draws, 62 persisted deferred table/seed values, 18 empty-barrel
     no-table seeds omitted on save.

3. All six leaves now state their reload boundaries and downstream owners. Template processing,
   terrain matching, aliases, capped archaeology, feature calls and STRUCTURE mob finalization are
   generation-time operations and are not rerun merely by loading a saved chunk. Deferred loot,
   block-entity runtime, spawner/vault runtime, AI and protocol projections remain with their named
   owners.
4. The remaining locked pool/template censuses, processor assignments, connector finals, archaeology
   caps, trial alias roster, fixed NBT, loot roots, spawner/vault configs and raw-entity
   finalization descriptions matched the official data and bytecode entry points inspected in this
   wave. No new missing asset or unowned content branch was found.

## Reproduction

The following commands operate only on repository-locked official artifacts and ignored generated
outputs:

```sh
shasum target/mc-reference/26.2/server.jar target/mc-reference/26.2/client.jar
MC_REF_JAVA="$JAVA_HOME/bin/java" \
  cargo run -q -p mc-reference --bin mc-ref -- reports
javap -classpath target/mc-reference/26.2/server-26.2.jar -c -p \
  net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement \
  net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate \
  net.minecraft.world.RandomizableContainer
```

For the corrected trial-barrel split, run Java 25 `jshell` with the extracted server jar and all
`target/mc-reference/26.2/libraries/**/*.jar` files on the classpath, iterate
`data/minecraft/structure/trial_chambers/**/*.nbt` with `NbtIo.readCompressed`, resolve each block's
palette state, and print `LootTable` for `minecraft:barrel`. The exact output is:

```text
decor/barrel -> minecraft:chests/trial_chambers/corridor
decor/disposal -> <none>
intersection/intersection_2 -> minecraft:chests/trial_chambers/intersection_barrel
intersection/intersection_3 -> minecraft:chests/trial_chambers/intersection_barrel
```

The leaf test-vector sections contain the rule-specific executable assertions, now including
save/reload boundaries for deferred loot, fixed inventories, brushable archaeology, spawner/vault
state and finalized entities.

## Unresolved items

`EXP-WGEN-001` remains planned and attached through the world-generation pipeline for end-to-end
generation/loading parity and order-sensitive seam checks. The official source and locked data were
sufficient for the leaf-local corrections above, so no new guessed constant or inferred asset was
introduced and each assigned completion record retains `unknowns = []` and
`status = "SourceSpecified"`.

## Evidence and verification

- `shasum target/mc-reference/26.2/{server.jar,client.jar}` — passed; both SHA-1 values matched the
  repository lock.
- `MC_REF_JAVA="$JAVA_HOME/bin/java" cargo run -q -p mc-reference --bin mc-ref -- reports` — passed;
  official reports regenerated from the locked server jar.
- Java 25 `jshell` plus `NbtIo.readCompressed` over all trial-chamber templates — passed; exactly
  the four barrel rows shown above were found.
- `MC_REF_JAVAP="$JAVA_HOME/bin/javap" cargo run -q -p mc-reference --bin mc-ref -- symbols` —
  passed; 2,789 locators across 952 classes.
- `MC_REF_JAVA="$JAVA_HOME/bin/java" MC_REF_JAVAP="$JAVA_HOME/bin/javap" cargo run -q -p mc-reference --bin mc-ref -- verify --offline`
  — passed; 417 documentation IDs, 331 completion slices, 9,078 locked catalog IDs, 307 experiment
  definitions and all protocol, behavior-surface, join and implementation-manifest consistency
  checks completed.
- `git diff --check` — passed.

These are documentation-only changes, so the `AGENTS.md` documentation exception applies and Rust
formatting, Clippy and crate tests are not required. The successful mc-ref commands compiled and ran
the affected `mc-reference` tool as part of the reference-specific verification.
