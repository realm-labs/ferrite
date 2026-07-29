# Determinism and Replay Primitives

Ferrite treats deterministic state as a versioned compatibility contract. Topology, executor
schedule, Rust memory layout, Serde map order, runtime registry IDs, and protocol packet IDs are
outside that contract.

## Named random streams

`ferrite-simulation` owns Region-local named streams. A stream is derived from the world seed and
its validated resource identifier, so requesting `ferrite:weather` before or after
`ferrite:loot` produces the same independent sequences.

The initial algorithm is `Xoshiro256StarStarV1`. SplitMix64 expands each derived seed into the
nonzero four-word state. Algorithm identity and every materialized stream state are included in
snapshots. Algorithm output, stream derivation, and snapshot continuation have locked tests.

Selection validates its bounds before advancing state. Empty selection, a zero denominator, or a
numerator greater than its denominator therefore cannot consume random state. New gameplay systems
must use a responsibility-specific stream name rather than share a catch-all generator.

## Canonical encoding and hashes

`ferrite-replay` implements ADR-0015 directly:

- fixed-width integers are little-endian;
- counts use minimal unsigned LEB128;
- strings and byte payloads are length-bounded;
- booleans and enum tags reject unknown encodings;
- maps and sets sort canonical key bytes and reject duplicate canonical keys;
- non-finite authoritative floats are rejected;
- decoders reject truncation, non-minimal encodings, invalid UTF-8, and trailing bytes.

Region hashes use the `ferrite:region-state:v1` BLAKE3 domain and include the Region key and
committed tick. World hashes use `ferrite:world-state:v1`, include world identity, committed tick,
the content-manifest digest, and Region hashes sorted by canonical Region key. Locked vectors cover
the codec, RNG, Region hash, world hash, and complete replay-log encoding.

## Replay boundary

Command and event envelopes carry their own magic, schema version, tick, sequence, semantic kind,
stable Region/entity identity, and a one-MiB maximum payload. Replay headers bind a log to:

- the Ferrite implementation identity;
- stable world identity;
- content-manifest digest;
- Region mapping version;
- random algorithm version;
- initial committed tick.

Frames have enforced command, event, Region, and total-log bounds. Commands/events require matching
frame ticks and strictly increasing sequences. Region keys must match the header world and mapping
version.

`verify_replay` feeds recorded commands into a topology-independent `ReplayTarget`, then compares
events, ordered Region hashes, and the world hash. It stops at the first mismatch and reports the
frame, tick, exact category, and expected/actual diagnostic hashes or Region identity. Local and
Lattice-backed targets will implement this same interface in later runtime batches.
