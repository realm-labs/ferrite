# G04-P5-B2 — Durable generated world completion record

> **Superseded completion boundary (2026-08-02):** this record remains historical evidence for the
> former player-visible-equivalence contract. Goal 04 has been reopened by G04-D024 through D026.
> It does not prove same-input vanilla 26.2 semantic world identity, and the later production audit
> found a flat block-interaction shadow state plus a cross-Region gateway-fatal path. The current
> `RegionFileStore` evidence is local-only and does not prove recovery after permanent source-node
> loss. The current source of truth is the
> [Goal 04 status ledger](../../goals/04-durable-generated-world-status.md); distributed storage is
> owned by Goal 07 and [ADR-0026](../../adr/0026-location-independent-region-storage.md).

## Result

`Satisfied`. Goal 04 replaces the formal server's production flat fixture with one configured,
generated, Region-owned, durable world path. The same committed chunk columns drive lifecycle,
simulation, collision, persistence, lighting, environment work, portals, and Java 26.2 projection.
The exact client can explore the generated Overworld, disconnect, rejoin the same durable state,
and travel through an authoritative portal into a fully streamed Nether.

This closes world authority, not the complete Minecraft game. Player inventory, survival,
progression, commands, broad entity tracking, mobs, and distributed ownership remain assigned to
Goals 05 through 07.

## Terminal acceptance audit

| Requirement | Terminal evidence |
|---|---|
| Versioned world configuration | Schema 2 owns world ID, seed, generator, spawn, distances, dimensions, border/environment defaults, and save policy; schema 1 has a closed fail-closed migration |
| Durable bootstrap | `FWM0` metadata and dimension-scoped control records load or create the configured identity instead of hard-coding world 1 |
| Chunk lifecycle | Bounded tickets drive load/generation, activation, save receipt, unload, continuation, and recovery through the formal composite Region runtime |
| One authority | Projection, collision, environment work, portal contact, and persistence all consume the committed `ChunkColumn`; no production `MinimalTerrain` or `FlatWorldCollision` reference remains |
| Generated content | Deterministic biome, density, surface, carver, feature, waystone structure, heightmap, fluid, and light stages publish only after fenced generation commits |
| Dimensions and portals | Configured Overworld, Nether, and End have independent durable level/control state; cross-Region portal writes and player transfer publish one checkpoint-wide committed prefix |
| Failure behavior | Unknown/mixed/corrupt formats, stale generations, capacity exhaustion, save/unload races, interrupted flush, and incompatible metadata fail closed |
| Exact client | Two `Satisfied` Java 26.2 MCP bundles prove complete 25-chunk views, ordinary grounded movement and jump, clock/weather observation, restart identity, Overworld-to-Nether travel, and three inspected framebuffer captures |

The exact-client evidence and screenshot hashes are retained in
[G04-P5-B1](g04-p5-b1-exact-client-world-acceptance.md). Full chunk installation also exercises the
biome palettes, three heightmaps, block/fluid states, and light arrays in the normal Java packet;
the client-side nearby-block and framebuffer observations are intentionally not server-authority
oracles.

## Production manifest and responsibility naming

`cargo ferrite production verify` passes with 18 service rows, 12 serverbound responsibility rows,
and all 48 decoded Play serverbound packets assigned exactly once. All eight world responsibility
rows—configuration, chunk lifecycle, generation, projection, collision, environment, dimensions,
and portals—are `Integrated` with every applicable production stage and exact-client evidence.
Unrelated login, player, entity, command, and distributed gaps remain truthfully partial, planned,
or unsupported for later goals.

The active source audit found historical planning-phase continuity bytes only in
`continuity/legacy_identity.rs` and `world-inspector/compatibility_identity.rs`. Both modules state
that these are read-only persisted compatibility inputs. Production writers use simulation,
player-service, entity-service, and world-service identities; the inspector entry has no planning
phase constants. Historical reports and architecture roadmap headings remain immutable provenance,
not active responsibility names.

## Format and migration audit

The terminal matrix reran current round trips and every retained input boundary:

- configuration schema 1 to closed schema 2, plus unknown-field/version rejection;
- `FWC1`/`FWC2` to `FWC3` without synthetic structure or light authority;
- `P8C1` to `P8C2` without inventing a generation continuation;
- `P8L1` to `FWL2` with deterministic environment initialization;
- historical continuity identity classification and atomic rewrite to current identities;
- current-generation idempotence, mixed/unsupported rejection, duplicate identity rejection,
  interrupted migration retry, and rollback denial;
- corrupt formal control-store rejection and coordinated three-dimension restart;
- offline inspector classification of current and compatibility records.

Focused `ferrite-world`, `ferrite-server-runtime`, formal persistence, durable-world, continuity
migration, and `world-inspector` suites passed. New commits write only `FWC3`, `P8C2`, `FWL2`,
current responsibility identities, and the versioned world metadata record.

## Bounded work and performance audit

World work remains capacity-driven rather than unbounded: formal generation owns a fixed pool of at
most four workers, bounded request/result channels, at most four admitted generation results per
Region tick, bounded lifecycle/event work, and explicit overload errors. Production collection is
nonblocking and result publication remains submission-ordered. The exact-client run exposed and
removed repeated whole-world snapshot encoding: immutable chunk snapshots and durable records are
revision-cached, collision captures one committed view per session poll, and ingress uses bounded
reads plus a shared event budget.

The nonblocking in-flight accounting test, lifecycle fault matrix, full-view exact-client status
(`25` sent, `0` pending), and clean shutdown/restart all pass. `cargo ferrite capacity verify` also
validates the three committed synthetic Region profiles and the versioned benchmark report. Those
profiles are regression inputs, not a player-count or world-generation throughput claim; no
unsupported capacity promise is introduced here.

## Clean-source terminal gates

The containing commit is the clean Goal 04 source baseline. `git status --porcelain=v1` is empty
before and after the terminal run:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo ferrite source verify
cargo ferrite production verify
JAVA_HOME=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home \
  tools/ferrite-client-mcp/gradlew --no-daemon -p tools/ferrite-client-mcp check build
cargo ferrite capacity verify
git diff --check
```

Every command passes. The source verifier enforces the 1,200-physical-line ceiling across all
handwritten Rust files; the largest current file is 1,191 lines. Separate audits find no
`super::super`, broad lint suppression in the completion change, or production flat-world provider.

All Goal 04 batches are terminally complete. Goal 05 is the next ready implementation goal and owns
durable players, inventory, item/block transactions, menus, crafting, survival, progression, chat,
commands, and operator-facing gameplay configuration.
