# Minecraft 26.2 packet catalog

Ferrite keeps packet IDs inside `ferrite-protocol::java_26_2::catalog`. They are scoped by
connection state and direction and are not registry IDs, semantic event types, persisted values, or
simulation inputs.

The implementation has three distinct artifacts:

1. The official `packets.json` report remains ignored under
   `target/mc-reference/26.2/generated/reports/` and is never committed.
2. `reference/minecraft-java-26.2-packets.toml` is the compact Ferrite-owned lock. Each of its nine
   lanes lists identities in numeric-ID order, so an array index is the wire ID without repeating
   report records.
3. `ferrite-protocol/build.rs` validates the lock and emits Rust descriptors only into Cargo's
   `OUT_DIR`. Generated Rust is neither edited nor committed.

The build fails unless the lock is schema 1, Minecraft `26.2`, protocol `776`, exactly 256 packets
across nine unique nonempty lanes, canonical namespaced identities, contiguous lane IDs, and the
locked inventory SHA-1 `f34b0956b6399c749d4638cd6d3c9226685f41fa`.

`PacketCatalog` supports state/direction-local lookup by validated wire ID or packet identity.
Unknown, negative, oversized, wrong-state, and wrong-direction lookups return no descriptor.
Runtime packet dispatch must use this catalog; later packet-family codecs may not duplicate numeric
IDs in packet structs or export them into session, simulation, Region, or persistence APIs.

## Reproduction and drift checks

After regenerating the locked official reports, update the normalized lock explicitly:

```text
cargo run -p mc-reference --bin mc-ref -- protocol catalog --write
```

Ordinary verification is read-only:

```text
cargo run -p mc-reference --bin mc-ref -- protocol catalog
cargo run -p mc-reference --bin mc-ref -- protocol verify
cargo run -p mc-reference --bin mc-ref -- verify --offline
```

The verifier reconstructs the lock from the report, independently rechecks the 256-entry inventory
digest, and compares normalized content. The build script then repeats structural and digest checks
without requiring Mojang artifacts, so a clean source build consumes only the reviewed Ferrite
lock.
