# G04-P1-B1 — World configuration and schema migration

## Outcome

The formal server configuration is now schema 2 and owns one closed, versioned world surface.
`WorldConfig` validates a canonical nonzero 32-hex world ID, signed seed, the supported generator
version, generated or fixed spawn, view/simulation distances, ordered enabled dimensions, and
bounded autosave/checkpoint/shutdown policy. `ValidatedServerConfig` retains the parsed stable
`WorldId` for the runtime bootstrap rather than letting downstream code reinterpret the string.

The development-cluster generator and exact-client MCP both emit schema-2 configuration. Existing
schema-1 configurations remain accepted through a deterministic in-memory migration to the former
formal constants: world ID 1, seed 0, `ferrite:overworld_v1`, generated spawn, distances 10/10,
overworld only, 6,000-tick autosave, 128 pending Region saves, 64-commit checkpoints, and required
shutdown flush.

## Migration safety

Parsing first selects schema 1 or 2, then decodes the complete corresponding closed structure.
Missing schema, unknown fields, unknown schemas, invalid or noncanonical identifiers, unsupported
generators/dimensions, duplicate or misordered dimensions, invalid distances, fixed spawn outside
the build range, and unbounded save policies fail before process construction.

A schema-1 migration inspects `<storage.root>/worlds` after validation. It permits no durable entry
other than the canonical legacy world-ID directory, rejects symlinks and non-directory aliases, and
therefore cannot silently attach the old flat configuration to another durable world.

`ferrite-server --config <old> --migrate-config <new>` exposes an explicit operator migration. It
serializes canonical schema 2, creates the output with `create_new`, syncs it, and refuses to
overwrite any existing path. Ordinary loading never rewrites the operator's source file.

## Production truth

`world/configuration` advances from `Planned` to `Partial`: ingress, validation semantics,
configuration authority, and focused tests now exist. `Continuity` remains the truthful gap until
G04-P1-B2 binds the configuration to durable world metadata during formal startup.

## Verification

- `cargo test -p ferrite-server-runtime --lib --all-features`: passed; 18 tests, including schema
  migration, conflict rejection, closed-field and world-bound validation.
- `cargo test -p ferrite-server --all-features`: passed; the CLI creates one schema-2 file and
  rejects overwrite on the second attempt.
- client MCP `./gradlew --no-daemon check build` under Java 25: passed.
- `cargo ferrite production verify`: passed with 18 services, 12 serverbound rows, and 48 packets.
- Universal Rust and source/diff gates run before commit.
