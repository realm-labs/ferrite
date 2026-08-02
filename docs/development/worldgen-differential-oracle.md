# World-generation differential oracle

The Goal 01 world-generation oracle compares semantic chunk state produced by the locked official
Minecraft Java Edition 26.2 server with state produced by Ferrite. It is an acceptance instrument,
not a second generator and not a compatibility waiver. A successful comparison means all fields in
the frozen denominator are identical for one input. `G01-P8-B5` must repeat that proof for the
complete declared population.

## Frozen contract

[`worldgen-exactness.toml`](../../goals/minecraft-java-26.2/worldgen-exactness.toml) fixes:

- official server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- normalization schema `ferrite:worldgen-semantic-chunk/1`;
- 16 semantic field families and the narrow non-semantic representation exclusions;
- Overworld, Nether, and End vertical ranges and official 26.2 storage paths;
- seeds, chunk coordinates, data packs, and request-order populations;
- zero unexplained semantic divergence as the only acceptance result.

The implementation-manifest verifier checks the contract SHA-256 before accepting progress. A
contract change is therefore an explicit reviewed denominator change, not a way to hide a
divergence.

## Normalization boundary

`ferrite-testkit::worldgen_oracle` owns both adapters and the comparator:

- the official adapter reads the 26.2 Anvil location table, supported compression envelope, and
  modern padded palette storage, then converts NBT state into stable names and canonical NBT;
- the Ferrite adapter consumes a protocol-neutral `ChunkSnapshot`, resolves process-local block and
  biome IDs through stable-name callbacks, and records the same semantic shape;
- the comparator ignores only the source label and frozen representation exclusions, walks fields
  in generation-stage order, and reports the first differing stage, field, coordinate, and bounded
  values.

The normalized document includes chunk status, block/fluid states, biomes, block entities,
post-processing, heightmaps, structure starts/references, scheduled block/fluid ticks, light,
inhabited time, and generation metadata. Palette order, NBT compound order, Anvil sectors,
compression, and Ferrite recovery representation are intentionally absent.

## Reproduction

Generate and save an official world beneath ignored `target/` using the locked server and JDK 25.
The seed, packs, dimension, generation radius, and shutdown sequence must be recorded for every
population run. Then normalize matching official and Ferrite chunks:

```text
cargo run -q -p behavior-runner -- \
  worldgen-normalize-official <official-world-root> minecraft:overworld 0 0 \
  <evidence-root>/official-overworld-0-0.json

cargo run -q -p behavior-runner -- \
  worldgen-normalize-ferrite minecraft:overworld 2602 0 0 \
  <evidence-root>/ferrite-overworld-0-0.json

cargo run -q -p behavior-runner -- \
  worldgen-compare <evidence-root>/official-overworld-0-0.json \
  <evidence-root>/ferrite-overworld-0-0.json
```

The compare command exits successfully only for semantic identity. Divergence exits nonzero and
prints structured JSON suitable for a population runner and CI artifact.

## Repository hygiene

Official jars, extracted classes, Anvil regions, worlds, and normalized multi-megabyte documents
remain below ignored `target/` paths. The independently distributed Gradle Wrapper required by the
tracked client automation project is the sole tracked `.jar` exception: both repository auditors
bind its exact path to SHA-256
`497c8c2a7e5031f6aa847f88104aa80a93532ec32ee17bdb8d1d2f67a194a9c7`. Gradle caches, build output,
and client runtime state are scanned out only at their three exact ignored roots; no Mojang artifact
is whitelisted.

## Closure boundary

`G01-P8-B4` is complete when both adapters, the first-divergence comparator, the frozen contract,
machine-manifest binding, tests, and one real official-server smoke comparison are committed.
Expected differences from the current Ferrite generator are truthful B5 input.

`G01-P8-B5` is complete only when every declared seed/dimension/chunk/data-pack/request-plan case
passes with zero unexplained semantic divergence. A golden Ferrite replay, screenshot, statistical
threshold, or a tool that merely detects mismatches cannot close B5.
