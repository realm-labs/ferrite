# Minecraft Java 26.2 Reference Audit — Wave 1, Worker 3: Fixed-Template Structures

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

Baseline: `feba9fac70272c8eaa4a87ea10aacb430b34294b`

This worker audited the following source-reference leaves without changing Ferrite runtime code or
implementation disposition:

- `WGEN-STRUCTURE-BURIED-001`
- `WGEN-STRUCTURE-NETHER-FOSSIL-001`
- `WGEN-STRUCTURE-IGLOO-001`
- `WGEN-STRUCTURE-SWAMP-HUT-001`
- `WGEN-STRUCTURE-DESERT-PYRAMID-001`
- `WGEN-STRUCTURE-JUNGLE-TEMPLE-001`
- `WGEN-STRUCTURE-SHIPWRECK-001`
- `WGEN-STRUCTURE-OCEAN-RUIN-001`
- `WGEN-STRUCTURE-RUINED-PORTAL-001`

The audit covered entry points and source locators, fixed piece/template choice, terrain anchoring,
chunk clipping, palette and processor behavior, archaeology, markers, containers, occupants,
post-placement effects, constants/data inputs, RNG consumption, aborts, mutation order,
persistence/reload, generic cross-rule/protocol handoffs, and executable reproduction vectors.

## Evidence and method

Only repository-locked inputs were used: the official 26.2 client/server jars, extracted data and
generated reports, existing repository documentation, and `mc-ref`. The locked artifacts inspected
had these SHA-256 digests:

- `client.jar`: `40896ee9f1e2bec3c934daac7e93d41e9e3d9c2f8ae0ca366d52ffbfd1afa290`
- `server.jar`: `cdacdfb25898de5e4b4b0e5ddcc2722f77067e46605709c2d886c000ebb63ec5`
- `server-26.2.jar`: `183c0499c5f855570ee487dd38e141a53f0121f83a0b07a3bac2d8b6698823e8`

`javap -c -p -s` was used against the locked server jar for the owning structure, piece,
template-placement and serialization classes. `mc-ref query worldgen` was run for every assigned
structure/setup record, including both shipwrecks, both ocean-ruin climates, and all seven
ruined-portal variants. Locked templates and loot/data records were read from the generated data
tree and reports.

## Findings

1. Template-piece reload has a cross-cutting effective-box rule. The base tag contains `BB`, but
   `TemplateStructurePiece` reconstructs and replaces the effective box from saved template position
   and rebuilt settings. This is now explicit for Nether fossils, igloos, shipwrecks, ocean ruins
   and ruined portals.
2. Several post-placement paths have no saved completion latch. Re-entry can repeat Nether-fossil
   dried-ghast evaluation, igloo occupants/marker/top repair, shipwreck marker seeding, ocean-ruin
   chest/drowned markers, and the ruined portal's center-owned template/apron/drip/vine/leaf
   transaction. Live-state gates still affect each replay as documented.
3. Desert-pyramid support traversal is X outer, then Z inner. The prior “Z-major then X” wording was
   corrected to the bytecode-observed loop order.
4. Non-template save fields and missing-field defaults are now explicit. Buried treasure saves its
   final one-cell box after support admission; swamp hut, desert pyramid and jungle temple retain
   their scattered-piece height state and subclass latches, while negative height remains a live
   rescan sentinel. Desert archaeology candidates/roof state and jungle masonry choices are not
   persisted.
5. The audited families introduce no family-specific packet transaction. They hand resulting blocks,
   block entities, loot state and entities to generic structure-start, chunk, persistence and
   synchronization owners.

All nine leaves remain `SourceSpecified`; no implementation item was marked `Verified`.

## Unresolved items

No rule-local source ambiguity was found, so no new experiment was added. `EXP-WGEN-001` remains a
planned experiment owned by `WGEN-PIPELINE-001` for separately owned placement/distribution or
locate equivalence. Running its procedure materialized only the planned procedure record and did not
produce an automated result; no claim in these leaves depends on treating it as completed.

## Evidence and verification

- `MC_REF_JAVA="$JAVA_HOME/bin/java" cargo run -q -p mc-reference --bin mc-ref -- reports` — passed;
  reports regenerated from locked inputs.
- `mc-ref query worldgen minecraft:structure/<id>` — passed for every assigned structure/setup
  record listed above.
- `mc-ref experiment run EXP-WGEN-001` — correctly reported the experiment as planned/manual; no
  automated result was claimed.
- `MC_REF_JAVA="$JAVA_HOME/bin/java" MC_REF_JAVAP="$JAVA_HOME/bin/javap" cargo run -q -p mc-reference --bin mc-ref -- verify --offline`
  — passed: 417 documentation IDs, 331 completion slices, 2,798 symbol locators, 9,078 locked IDs
  with complete coverage, 307 experiment definitions, protocol version 776 inventory/coverage, and
  the implementation manifest all verified.
- `git diff --check` — passed.

Rust formatting, Clippy and crate tests were not run because this worker changed documentation and
reference-ledger text only.
