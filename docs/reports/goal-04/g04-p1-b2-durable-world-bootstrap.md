# G04-P1-B2 — Configured durable world bootstrap

## Outcome

The formal `ferrite-server -> NodeProcess -> MinecraftGateway` path no longer chooses world ID 1,
the overworld, or chunk `(0, 0)` independently of configuration. It resolves the validated schema-2
world, opens its overworld control Region, creates or restores durable metadata, and then builds the
route and all 25 initial Region authorities with the configured identity and spawn.

Generated spawn currently resolves to the deterministic bootstrap block `(8, 64, 8)`; a fixed
spawn preserves its exact signed block coordinates and height. Goal 04's generation and spawn-ticket
batches will replace the provisional generated-spawn selection without restoring a hard-coded route.

## Durable boundary

`ferrite:world-service/world_v1` is a current-only `Extension` continuity domain with its own
`FWM0` magic and schema 1 payload. It records and bounds:

- world identity, signed seed, and canonical generator version;
- resolved bootstrap spawn and ordered dimension catalog;
- Region mapping version and authoritative chunk-format version;
- the formal content-manifest digest.

The record is committed as persistence revision 1 through `RegionFileStore` in the overworld
control Region. Restart normalizes and validates the recovery point, canonical state hash, header,
record count, codec bounds, and every compatibility field before returning metadata to the formal
world loader. The world configuration production row therefore advances from `Partial` to
`Integrated`; full service-state continuity remains owned by G04-P1-B3.

## Storage safety and failure policy

The path follows
`worlds/<world>/dimensions/<namespace>/<resource-path>/regions/r.0.0`. Each child is created and
rechecked as a contained directory. Empty, dot, separator-bearing, symlinked, non-directory, or
escaping components fail closed. An existing Region directory without a fully committed matching
metadata recovery point is never reinterpreted as a new world.

Focused tests cover codec round trip, unsupported schema, first creation, restart load, seed
mismatch, an uncommitted/corrupt store, symlink rejection, and configured formal route identity,
spawn, and negative-Region mapping.

## Verification

- `cargo test -p ferrite-server-runtime --all-features`: passed, including all formal network and
  durable-world integration tests.
- `cargo fmt --all -- --check`, workspace Clippy with warnings denied, complete workspace tests,
  source policy, production manifest, and diff checks run before commit.
