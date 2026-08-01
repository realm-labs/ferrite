# G03-P4-B2 — Production runtime integration completion record

## Result

`Satisfied`. Goal 03 establishes one machine-checked production denominator for the formal
`ferrite-server` entry, removes Goal 01 planning-phase ownership from active architecture, migrates
legacy continuity identities without rewriting history, runs one deterministic composite Region
runtime, makes every decoded Play disposition explicit, and proves the route with the exact
Minecraft Java 26.2 client MCP.

This completion closes production integration, not the complete Minecraft feature set. Durable
generated worlds, full player survival, entity/multiplayer gameplay, and distributed operation
remain truthfully assigned to Goals 04 through 07.

## Production manifest truth

`cargo ferrite production verify` passed with 11 formal-entry service rows, 12 serverbound
responsibility rows, and all 48 decoded Play serverbound packets assigned exactly once. The
manifest distinguishes `Integrated`, `Partial`, `Unsupported`, and `Planned`; Goal 01 conformance
evidence is provenance and never promotes a row automatically.

Every row claimed `Integrated` supplies every applicable stage or marks a stage not applicable.
Rows that exercise authority but do not yet have formal-server durable continuity—including Region
tick composition, bootstrap terrain, block interaction, movement, and Play installation—remain
`Partial` with a `Continuity` gap. Chat/commands, inventory, broader player modes, and production
storage remain `Planned` or explicitly unsupported for their later owning Goals. Goal 03 therefore
does not misrepresent a connectable flat-world server as a feature-complete Minecraft server.

## Active naming and source boundaries

The active source audit searched `crates`, `apps`, and `tools` for `Phase5` through `Phase9`,
`phase_5` through `phase_9`, and module paths `::phase5` through `::phase9`; it found no active
planning-phase owner. The remaining repository-wide lowercase matches are required provenance:

- read-only `ferrite:phase5/*` through `ferrite:phase8/*` compatibility identities in migration
  code and `world-inspector`;
- immutable Goal 01 report/evidence locators in the implementation and production manifests;
- Goal 01 development records whose headings explicitly label them historical provenance;
- the Goal 03 naming and migration documents that describe the old-to-new mapping.

Those paths are deliberately retained: changing persisted bytes would break old worlds, while
renaming historical evidence would break stable audit links. Active modules and diagnostics use
simulation, player-service, entity-service, world-service, composite, dispatch, and projection
responsibilities instead.

`cargo ferrite source verify` passed for 1,271 handwritten Rust files with a 1,200-physical-line
maximum. A separate `super::super` audit over `crates` and `apps` returned no matches. Goal 03 adds
no broad lint suppression and does not widen a public API solely to shorten paths.

## Migration and inspection rerun

The terminal audit reran the migration and inspection suites from committed source:

```text
cargo test -p ferrite-server-runtime --test continuity_migration --all-features
cargo test -p world-inspector --all-features
```

The first passed all six cases: clean legacy upgrade, current-generation idempotence, interrupted
prepare/retry, mixed or unsupported generation rejection, duplicate canonical identity rejection,
and rollback denial. The inspector passed legacy/current classification plus mixed/unsupported
fail-closed behavior and its stable CLI usage test. Current writers emit only responsibility-owned
versioned identities; migration remains append-and-repoint and verifies the source digest again at
commit.

## Composite, dispatch, and exact-client evidence

The formal gateway uses the documented nine-stage composite Region order and requires a same-tick
composite commit. Canonical replay evidence binds ingress, service receipts, continuity preparation,
authoritative commit, and semantic projection. Application dispatch classifies all 48 decoded Play
variants as handled, rejected, gated, or unsupported; unsupported input is observable and cannot be
reported as a successful mutation.

The [G03-P4-B1 exact-client run](g03-p4-b1-exact-client-composite-acceptance.md) used normal client
input to move from Region `(0, 0)` to `(0, 1)`, commit block targeting/attack results, expose
`ChatMessage` as unsupported, render converged post-transfer terrain, and disconnect without leaving
authority behind. The final 1708×960 framebuffer PNG has SHA-256
`ec2b20b08b2e438d6e2f5bdb16605873a42053b20e13a507d9d87175290e34a8`. Generated game state,
secrets, logs, and framebuffer bytes remain ignored under `target`.

## Clean-source terminal gates

Commit `850449dbb5a9af7cf35d0ed3bff37ecbeef62bd3` was the clean implementation baseline.
`git status --porcelain=v1` was empty before and after the following terminal sequence:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo ferrite source verify
cargo ferrite production verify
JAVA_HOME=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home \
  tools/ferrite-client-mcp/gradlew --no-daemon -p tools/ferrite-client-mcp check build
git diff --check
```

Every command passed. The Java run included tests, distribution verification, and all three build
artifacts. The exact-client scenario had already passed against the identical committed source in
G03-P4-B1; the completion audit did not create or commit another Mojang-derived evidence bundle.

## Terminal evidence index

| Requirement | Evidence |
|---|---|
| Formal-entry denominator and truthful gaps | [production manifest contract](../../development/production-integration-manifest.md), `production-integration.toml`, and this report |
| Responsibility naming and compatibility migration | [G03-P1-B1](g03-p1-b1-simulation-player-naming.md), [G03-P1-B2](g03-p1-b2-entity-world-service-naming.md), and [G03-P1-B3](g03-p1-b3-continuity-identity-migration.md) |
| Composite production execution | [composite contract](../../development/composite-region-runtime.md) and G03-P2 reports |
| Explicit dispatch and post-commit projection | [dispatch contract](../../development/serverbound-dispatch.md) and G03-P3 reports |
| Exact 26.2 composite acceptance | [G03-P4-B1](g03-p4-b1-exact-client-composite-acceptance.md) |
| Migration, source, Java, Rust, and clean-source gates | This report |

All Goal 03 batches are terminally complete. Goal 04 is the next ready implementation goal and owns
replacement of the minimal flat terrain with a configured, generated, collision-aware, durable
world.
