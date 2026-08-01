# Production integration manifest

Goal 01's implementation manifest proves locked reference ownership and conformance. It does not
prove that the formal `ferrite-server` path invokes a behavior. Goal 03 therefore established the separate
[production integration manifest](../../goals/minecraft-java-26.2/production-integration.toml).
Goals 04 through 07 advance the same denominator as their responsibilities enter the formal path.

Verify it from the workspace root:

```text
cargo ferrite production verify
```

The verifier also runs from `cargo ferrite-check`. It reads the current
`PlayServerboundEntryPacket` enum and requires all variants to appear exactly once across the
manifest's sorted `serverbound` rows. It also requires the fixed nonpacket formal-entry service set,
safe existing evidence/test paths, exact status counters, known target Goals, and a complete
classification of every integration stage.

## Status vocabulary

| Status | Meaning |
|---|---|
| `Integrated` | Every required stage is implemented or explicitly not applicable, focused production tests exist, and any claimed client acceptance links Goal 02 evidence. |
| `Partial` | At least one production stage exists and at least one required stage remains a named gap. |
| `Unsupported` | The formal entry has a tested explicit default-closed or rejection outcome and a rationale; it is not silently ignored. |
| `Planned` | Wire or configuration ingress may exist, but production semantics remain assigned to one or more future Goals. |

The seven stages are ordered and exhaustive:

```text
Ingress
  -> Semantic
  -> Authority
  -> Continuity
  -> Projection
  -> FocusedTest
  -> ClientAcceptance
```

`not_applicable_stages` is reserved for a stage that the responsibility genuinely does not require,
such as persistence for a connection-local keep-alive challenge. It must not be used to hide
unfinished gameplay state. Each row partitions all seven stages exactly once among
`implemented_stages`, `not_applicable_stages`, and `gaps`.

## Initial production boundary

The initial Goal 03 manifest deliberately records a narrower result than Goal 01 conformance:

- status, required configuration, base custom-payload handling, management drain, keep-alive,
  teleport acknowledgement, and chunk-batch feedback are integrated at their current responsibility
  boundary;
- login, play installation, the flat world, player/block Region ticks, movement, client lifecycle,
  and block interaction are partial because continuity, complete authority, projection, or
  exact-client gameplay evidence remains open;
- chat, commands, inventory, containers, entity interaction, vehicles, player modes/input, pong,
  and production storage are planned; their decoded Play packets now receive an explicit
  `Unsupported` application disposition until the assigned Goal installs authority;
- optional C4 services remain explicitly unsupported/default-closed until an enabled
  implementation receives its own production evidence.

Protocol codec tests may be listed as provenance for `Planned` rows, but they do not satisfy the
`FocusedTest` stage. A later batch changes a row only when the newly claimed stage has production
evidence. Player-visible rows do not claim `ClientAcceptance` merely because an unmodified or
instrumented client once connected; the scenario must exercise the responsibility being claimed.

## Goal 04 world denominator

Goal 04 replaces the former single `world/bootstrap-terrain` row with eight responsibility rows:
configuration, chunk lifecycle, generation, projection, collision, environment, dimensions, and
portals. Their frozen ownership and format contract is
[durable world production](durable-world-production.md). The split prevents the existing flat
projection, isolated generation algorithms, or Region lifecycle tests from accidentally satisfying
the complete generated-world claim.
