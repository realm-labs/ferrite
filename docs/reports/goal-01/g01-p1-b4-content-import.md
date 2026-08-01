# G01-P1-B4 Content Import Evidence

Date: 2026-07-29  
Reference: Minecraft Java Edition 26.2  
Bundle schema: 1

## Locked source verification

| Artifact | Bytes | SHA-1 | BLAKE3 |
|---|---:|---|---|
| client | 39,193,383 | `2dc72797acbc1b63fc16a11c4ac393605f453754` | `321eab22a01a87715a43143a7858b6483893864bb8bddd9e3fd15ce24224764a` |
| server | 60,894,273 | `823e2250d24b3ddac457a60c92a6a941943fcd6a` | `c56e73c7b3ac5542621a71fb94e0d48755affe593334eaf97c925295e24309cd` |

The importer consumed the ignored local cache. No JAR, generated report, extracted JSON, content
entry, or generated bundle is included in this report or committed by the batch.

## Result

| Measure | Result |
|---|---:|
| Catalog registries | 32 |
| Classified and lowered IDs | 9,078 |
| Unreviewed or ambiguous IDs | 0 |
| Bundle BLAKE3 | `f552cb304008493d763774f2e59d1ebabb3fc5850a1fa827dfb9413d2ef46cb9` |
| Content-manifest BLAKE3 | `9647b1f54a12e729a1fe212aa5f84310c618784e4d7154fdfb05d0761414ef53` |

Each registry matched its committed expected count and sorted-ID SHA-1. Every ID resolved to exactly
one reviewed catalog family. Bundle deserialization recalculated each canonical payload digest.

## Commands

```text
cargo ferrite content import
cargo ferrite content verify
FERRITE_CONTENT_BUNDLE=<ignored bundle> cargo test -p ferrite-registry --test catalog --all-features
```

The locked import and verify commands passed. The bundle-backed partition suite passed 32/32 tests
in 20.62 seconds. The ordinary no-artifact form of the same suite remains part of workspace tests.

## Phase 10 revalidation note

`G01-P10-B1` rebuilt the ignored bundle from the same locked artifacts and all 9,078 canonical
entries. The entry-level content-manifest digest remained
`9647b1f54a12e729a1fe212aa5f84310c618784e4d7154fdfb05d0761414ef53`, while the aggregate bundle
digest reproduced as `887fb5b7be081828a492bcc41c08b880a8fc40648322b5630628be057d7391e2`.
The reviewed bundle lock now records the reproducible aggregate value; the artifact hashes,
registry counts, ID digests, entry payload digests, and content-manifest digest did not change.
