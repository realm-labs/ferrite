# G04-P2-B2 — Deterministic overworld generation

> Historical evidence: this batch established Ferrite's first deterministic generated overworld,
> but its former equivalence boundary is superseded by the current
> [vanilla exactness contract](../../development/worldgen-equivalence-boundary.md). It does not
> constitute Minecraft 26.2 compatibility completion.

## Outcome

The formal generation worker now executes `ferrite:overworld_v1` instead of returning an unchanged
column. The configured signed 64-bit seed derives independent named streams for broad height,
detail, temperature, humidity, and caves. Generation mutates the same `ChunkColumn` later used by
continuity, simulation access, collision, and client projection.

The status pipeline now performs:

- `BIOMES`: deterministic quart-cell climate selection across the complete vertical column;
- `NOISE`: non-flat, cross-chunk-continuous density terrain over a bounded height range;
- `SURFACE`: deterministic surface replacement;
- `CARVERS`: seed-stable three-dimensional cave removal below protected surface depth;
- `FEATURES`: bounded positional surface decoration;
- `SPAWN`: vertical headroom validation before the chunk reaches the spawn milestone.

Structure starts/references remain P2-B3. Lighting/environment semantics remain P3-B2. The
authoritative generated columns remain intentionally disconnected from Java terrain projection
until P2-B4 installs complete block-state/biome registry mappings, heightmaps, light payloads, and
unload projection.

## Representation and bounds

Biome replacement and uniform block filling were added as atomic `ChunkColumn` mutations. Candidate
generation happens in the bounded external worker; any coordinate, palette, revision, or spawn
validation failure discards the candidate and fails the formal tick closed. The gateway admits at
most four completed generation stages per tick so density and carving cannot consume an unbounded
tick budget.

The generator's compatibility promise is deterministic Ferrite replay and Goal 01's audited
player-visible equivalence class. It does not claim Mojang same-seed block-for-block identity.

## Verification

- `ferrite-world` generator tests prove same-seed identity, seed sensitivity, cross-chunk height
  continuity, non-flat terrain, cave carving, bounded features, and spawn preparation.
- Formal lifecycle tests drive every sequential status through `FULL`, inspect generated surface
  blocks, commit continuity, derive ticket activity, and retain all P2-B1 fencing/receipt cases.
- Focused Clippy for `ferrite-world` and `ferrite-server-runtime` passes with warnings denied.
- Universal Rust, source-policy, production-manifest, and diff gates run before commit.
