# G04-P0-B1 — Durable world production truth

## Outcome

Goal 04 now has a frozen formal-entry denominator for a configured, generated, durable world. The
[production contract](../../development/durable-world-production.md) binds one authoritative
`ChunkColumn` representation to generation, simulation, collision, persistence, and Java
projection; fixes versioned configuration and durable identity boundaries; defines fenced
generation, save acknowledgement, unload, compaction, startup, and shutdown rules; and names the
exact-client and fault evidence required for completion.

The contract deliberately does not select Mojang Anvil/NBT as Ferrite's production format. Existing
At this audit point, `RegionRecoveryPoint`, `RegionFileStore`, `FWC1`, `P8C1`, and `P8L1` were the
canonical starting point. Mojang-format import or export can be an adapter without making protocol
or save layout part of Region authority.

## Production manifest baseline

The machine verifier now requires eight sorted world responsibilities:

- `world/configuration`;
- `world/chunk-lifecycle`;
- `world/generation`;
- `world/projection`;
- `world/collision`;
- `world/environment`;
- `world/dimensions`;
- `world/portals`.

The former `world/bootstrap-terrain` row became the narrower `world/projection` responsibility.
This prevents a fixed terrain packet provider from satisfying generation, collision, or durable
authority. Initial production truth is intentionally conservative:

| Disposition | Count |
|---|---:|
| `Integrated` | 7 |
| `Partial` | 10 |
| `Unsupported` | 1 |
| `Planned` | 12 |

Across 18 service rows and 12 serverbound rows, all 48 decoded Play packets remain assigned exactly
once. Goal 01 algorithms and conformance tests remain evidence, not automatic production stages.

## Frozen compatibility decisions

- Server configuration schema 2 owns world ID, seed, generator version, spawn policy, distances,
  dimensions, and save policy. Schema 1 migration uses the former formal constants and refuses a
  conflicting durable store.
- Each `SimulationRegionKey` receives one path-contained store beneath its world/dimension root.
  Durable identities never contain process, worker, Lattice placement, or Minecraft packet IDs.
- New writes use only responsibility-owned continuity identities. Legacy Phase 8 records remain
  read-only migration inputs and historical evidence remains link-stable.
- Save acknowledgement is stronger than Mojang's observed asynchronous dirty-clear behavior: only
  a matching synced commit receipt clears a dirty revision.
- World generation claims audited control flow and deterministic Ferrite behavior, not
  block-for-block same-seed identity with Mojang. Deferred statistical experiments remain separate.

## Verification

- `cargo test -p ferrite-tooling production --all-features`: passed; six production verifier tests.
- `cargo ferrite production verify`: passed; 18 service rows, 12 serverbound rows, 48 packets.
- `cargo fmt --all -- --check`: passed.
- Universal Clippy, workspace tests, source policy, and diff checks run before the batch commit.
