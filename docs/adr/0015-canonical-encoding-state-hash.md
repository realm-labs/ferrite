# ADR-0015: Canonical Encoding and State Hashing

## Status

Accepted

## Context

Replay, topology comparison, snapshot validation, and divergence diagnosis need stable bytes across
processes and supported platforms. Rust layout, Serde implementation details, map iteration, and
runtime numeric IDs are not stable enough to be that contract.

## Decision

Ferrite defines an explicit, versioned canonical binary encoding for stable semantic records and
immutable state projections:

- every envelope carries a magic/domain identifier and schema version;
- integers use declared widths and little-endian bytes;
- variable-length counts use a minimal unsigned LEB128 form and reject non-minimal encodings;
- strings are UTF-8 with a bounded canonical byte length;
- enums use explicit stable numeric discriminants;
- optional and sequence values carry explicit tags/counts;
- maps and sets sort by canonical key bytes before encoding;
- persistent resource and entity identities are encoded, never runtime registry IDs or Bevy IDs;
- authoritative floating-point fields encode exact IEEE bits after rejecting non-finite values;
- padding, struct memory, filesystem order, and platform-native widths never enter the stream.

Canonical Region and world hashes use BLAKE3-256 with a versioned Ferrite domain prefix. Hash inputs
are stable semantic projections, not replay files, compressed storage blocks, Minecraft packets, or
Lattice actor state. Region hashes sort by Region key; world hashes sort committed Region hashes and
include the world tick and content-manifest digest.

Golden vectors must cover encoding, decoding, rejection of noncanonical inputs, Region hashes, and
cross-platform output.

## Consequences

- Replay and topology comparisons have one byte-level source of truth.
- Schemas require explicit evolution rather than inheriting Rust/Serde changes.
- Encoding is additional code and must remain bounded and fuzzed.
- BLAKE3 is a compatibility dependency for hash outputs, not a gameplay random source.

## Alternatives Considered

- Hash serialized Rust structs: rejected because layout and serializer behavior may drift.
- Hash Minecraft packet bytes: rejected because projection order and adapter changes are not world
  state.
- Use Lattice's internal fingerprints: rejected because Ferrite state compatibility cannot depend on
  a substrate implementation.

## Migration or Reversal Plan

Add a new schema/hash-domain version and dual-read old records. Never change bytes produced by an
existing version. Golden vectors and replay migration tests gate retirement of old versions.
