# World-generation vanilla exactness contract

Ferrite targets the semantic world output of the locked Minecraft Java Edition 26.2 server. Given
the same seed, world configuration, enabled data packs, dimension, chunk coordinates, generation
status inputs, and relevant neighboring state, Ferrite must produce the same authoritative world
state. Terrain that merely looks similar, matches broad distributions, or passes a statistical
population is not completion evidence.

The historical filename remains stable because Goal 01 reports and machine ledgers link to it. This
document supersedes the earlier player-visible-equivalence policy.

The current `implementation.toml` and production-integration manifest remain frozen records of the
earlier completion claim. Their statistical deferral text must not be interpreted as overriding
this contract. `G01-P8-B4` owns the schema, renderer, and regenerated-manifest migration needed to
make the machine-readable policy authoritative again.

## Exact semantic denominator

Differential acceptance normalizes and compares every vanilla-significant field that the relevant
generation stage owns, including:

- block and fluid states, biomes, block entities, and post-processing marks;
- world-surface and ocean-floor heightmaps plus any additional published heightmap state;
- chunk status, structure starts and references, carving and feature results;
- scheduled generated ticks, light state, inhabited or generation metadata where applicable;
- deterministic random-call order and stage side effects that influence later generation.

Exactness does not require the same Java object graph, thread scheduling, Region size, snapshot
encoding, compression, or Anvil file bytes. Ferrite may retain its native Region journals and
recovery points if they preserve the complete semantic denominator and reconstruct it exactly.
Anvil/NBT import and export are separate, versioned interoperability adapters and never a second
live authority.

## Reference-oracle requirement

The locally locked official 26.2 server and data reports are the differential oracle. A committed
suite must run matching generation requests against the official reference and Ferrite, normalize
both results, and report the first differing stage, field, and coordinate. Coverage must include:

- fixed positive, negative, boundary, and far-coordinate chunks across multiple seeds;
- Overworld, Nether, and End generation;
- biome boundaries, caves, aquifers, carvers, ores, placed features, and structures;
- forward, reverse, parallel, restart, and continuation request plans wherever request order can
  affect the official result;
- custom 26.2 data-pack inputs supported by the vanilla server contract.

Golden Ferrite hashes prove replay only. Screenshots prove client projection only. Neither can
replace official-server differential evidence.

## Status of former experiments

`EXP-WGEN-001`, `EXP-WGEN-005`, and `EXP-WGEN-006` were originally scoped as statistical
equivalence experiments. Under the exactness contract they remain useful for coverage selection and
diagnostics, but their former thresholds cannot close compatibility. Their replacement condition is
a committed same-input official/Ferrite differential suite with zero unexplained semantic
divergence for the declared denominator.

Until that evidence exists, the source-derived world-generation algorithms remain valuable partial
implementation evidence, but Goal 01 world-generation exactness and Goal 04 vanilla generated-world
acceptance remain open.
