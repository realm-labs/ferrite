# Minecraft Java Edition 26.2 Behavior and Protocol Reference

This is Ferrite's version-locked reference for observable gameplay behavior and unmodified-client
server protocol compatibility in Minecraft: Java Edition `26.2`. Before implementing or testing a
mechanic, start at its stable parent rule, follow the implementation-level leaf rule, and query the
locked content catalog. Before implementing a connection or packet path, start at the
[protocol reference](protocol/README.md). Turn unresolved details into evidence instead of filling
gaps from memory.

The baseline is locked by the
[official 26.2 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-2)
and the official version manifest: Data Pack `107.1`, Resource Pack `88.0`. See the
[source lock](sources.md) for artifact SHA-1 values, report-generation procedures, and legal
boundaries.

This English library is the single normative documentation source. Keeping one maintained language
avoids mirror drift; rule IDs and evidence IDs remain the stable references used by implementation
and tests.

## Scope

The behavior manual uses two prose layers. Leaf specifications describe algorithms, state machines,
ordering, constants, branch/abort behavior, side effects and executable vectors. The content catalog
classifies every locked ID in the covered registries as an inherited behavior family, explicit
special behavior, `DataOnly`, or an explicit `Unreviewed` backlog; concrete official values are
queried locally instead of copied into Git. The
[`behavior-surfaces.toml`](behavior-surfaces.toml) ledger independently inventories the root ways
behavior enters or leaves the game model, so a tidy domain index cannot hide an unowned command,
lifecycle, reload, persistence or cross-system boundary. The protocol reference separately owns
wire framing, connection states, packet catalogs, field layouts, registry projection,
acknowledgements, ordering, and conformance vectors.

In scope:

- server-authoritative state, transitions, and ordering;
- edge cases and quirks that a player can observe or exploit;
- observable client prediction, server rejection, and correction semantics;
- the way data-driven content parameterizes generic algorithms.
- exact server-side wire compatibility required by an unmodified 26.2 client;
- protocol connection states, packet direction/identity/layout, registry mappings, acknowledgements,
  and observable packet order.

Out of scope:

- original save formats, server implementation internals, plugin APIs, and renderer internals;
- repository copies of decompiled sources, Mojang assets, Wiki prose, or generated reports;
- block-for-block same-seed world-generation identity. Ferrite retains the existing architecture's
  player-visible-equivalence goal.

## Specification index

| # | Specification |
|---:|---|
| 1 | [Ticks, time, and chunks](behavior/01-tick-time-and-chunks.md) |
| 2 | [Blocks and updates](behavior/02-blocks-and-updates.md) |
| 3 | [Environment](behavior/03-environment.md) |
| 4 | [Redstone and explosions](behavior/04-redstone-and-explosions.md) |
| 5 | [Player movement and interaction](behavior/05-player-movement-and-interaction.md) |
| 6 | [Items, inventories, and progression](behavior/06-items-inventories-and-progression.md) |
| 7 | [Entities and combat](behavior/07-entities-and-combat.md) |
| 8 | [Mobs, AI, and spawning](behavior/08-mobs-ai-and-spawning.md) |
| 9 | [World generation and dimensions](behavior/09-worldgen-and-dimensions.md) |
| 10 | [Client-observable behavior](behavior/10-client-observable-behavior.md) |

Companion documents:

- [Copy-ready Codex Goal Prompt](goal-prompt.md)
- [Protocol compatibility reference](protocol/README.md)
- [Implementation-level leaf rules](mechanics/README.md)
- [Content behavior catalog](catalog/README.md)
- [Behavior-surface ownership ledger](behavior-surfaces.toml)
- [Mapped network-ingress root inventory](network-ingress-roots.md)
- [Recoverable command-root ownership map](command-roots.toml)
- [Recoverable player-lifecycle root inventory](player-lifecycle-roots.md)
- [Recoverable world-lifecycle root inventory](world-lifecycle-roots.md)
- [Recoverable persistence/reload root inventory](persistence-reload-roots.md)
- [Recoverable data-reload root inventory](data-reload-roots.md)
- [Recoverable cross-system join matrix](cross-system-joins.toml)
- [Directed experiments](experiments/README.md)
- [Locked coverage report](coverage.md)
- [Methodology](methodology.md)
- [Source lock](sources.md)

## Reference tooling

The independent `mc-ref` CLI is a workspace development tool and is not a Ferrite runtime
dependency:

```sh
cargo run -p mc-reference --bin mc-ref -- fetch --version 26.2
cargo run -p mc-reference --bin mc-ref -- reports
cargo run -p mc-reference --bin mc-ref -- query block minecraft:observer
cargo run -p mc-reference --bin mc-ref -- symbols
cargo run -p mc-reference --bin mc-ref -- coverage
cargo run -p mc-reference --bin mc-ref -- readiness
cargo run -p mc-reference --bin mc-ref -- surface coverage
cargo run -p mc-reference --bin mc-ref -- surface readiness
cargo run -p mc-reference --bin mc-ref -- surface verify
cargo run -p mc-reference --bin mc-ref -- experiment verify
cargo run -p mc-reference --bin mc-ref -- verify --offline
```

All downloaded jars, extracted server code container, generated reports, libraries, logs and
experiment worlds live in `target/mc-reference/26.2/`. The cache can be reused for fully offline
query and verification.

[`completion.toml`](completion.toml) is the recoverable gameplay-slice work queue;
[`behavior-surfaces.toml`](behavior-surfaces.toml) is the independent root-boundary work queue.
`mc-ref readiness` validates both ledgers, all 65 parent rules, every leaf rule, and the scope of all
95 locked registries, then exits nonzero while `Todo`, `InProgress`, or `Unreviewed` work remains.
The slice ledger currently has no `Todo` or `InProgress` entries, but the catalog has 862 explicitly
`Unreviewed` IDs across six recoverable fallback families. The surface ledger additionally has seven
`InProgress` roots and no `Todo` roots, so gameplay readiness is intentionally blocked for both reasons.
Three roots are structurally `Mapped`; this only means that their inventories and owners are explicit,
not that referenced slice work is promoted. Four `SourceInconclusive` slices retain explicit
experiments for facts that source alone cannot settle. `mc-ref verify --offline` validates all three
gameplay structures while protocol readiness remains a separate passing gate.

Seven lookup paths lead to the same evidence graph:

| Starting point | Where to go |
|---|---|
| Root behavior entry or exit | `mc-ref surface coverage`, then `behavior-surfaces.toml` |
| Subsystem | Parent specification index, then its `Verification owner`/leaf IDs |
| Rule ID | Search the stable parent or semantic leaf heading |
| Registry ID | `mc-ref query <kind> <minecraft:id>` |
| Connection state or packet | [Protocol compatibility reference](protocol/README.md), then the locked `packets.json` report and cited codec symbols |
| Source symbol | `mc-ref symbols`, then search the class/method locator |
| Experiment ID | `mc-ref experiment list` and `experiments/definitions.toml` |

## Usage

1. Use the stable parent rule to identify the gameplay boundary, then follow its leaf rules.
2. Resolve a concrete `minecraft:<id>` with `mc-ref query`; implement the returned behavior family
   plus any special leaf rule and locked data. An `Unreviewed` result is a reference blocker, not an
   implementation default.
3. A `Confirmed` branch may drive implementation and tests. A `Cross-checked` or
   `Implementation blocker` branch must first run its linked `EXP-*` procedure when exactness
   matters.
4. Run `mc-ref symbols` and `mc-ref coverage` whenever a locator or catalog family changes.
5. Never silently turn a provisional/conflicting result into implementation and never infer a
   missing constant from prose.
6. Give later versions sibling directories. Never silently rewrite conclusions locked to `26.2`
   here.
7. For protocol work, verify state, direction, packet ID, field layout, bounds, ordering, and
   semantic projection independently; encode/decode round trips alone are not compatibility
   evidence.

“Source” in this library means a class-and-method locator in an official jar. The prose is an
independent behavioral specification and contains no Mojang implementation code.
