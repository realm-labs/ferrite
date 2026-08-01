# G02-P4-F001 — Fabric Play custom-payload remediation

## Finding

The instrumented Fabric client could reach Play and receive terrain from the formal
`ferrite-server` entry, but the connection then terminated with:

```text
play serverbound packet minecraft:custom_payload is outside the implemented required families
```

The packet is not an unknown extension. The locked Play catalog assigns
`minecraft:custom_payload` serverbound ID 22, and the audited base-listener contract requires both
the `minecraft:brand` form and every bounded unknown-channel remainder to be consumed and ignored.
The required-family codec deliberately rejects this C4 packet when called directly, but the formal
connection driver incorrectly sent every Play packet through that codec.

## Remediation

The connection driver now identifies the custom-payload descriptor through the locked Play catalog
before required-family dispatch. It reuses the same identifier, UTF, and 32,767-byte remainder
decoder as configuration custom payloads, requires complete body consumption, and then applies the
base Play listener's no-effect behavior.

This does not enable arbitrary plugin handling or widen gameplay dispatch. Malformed identifiers,
oversized remainders, trailing bytes, unknown packet IDs, and every other optional family remain
fail closed. The required-family decoder also continues to reject direct custom-payload input, so
its original ownership boundary is unchanged.

## Evidence

The focused connection test sends both `minecraft:brand` and `fabric:registry/sync` payloads after
the real login/configuration/Play transition. Both leave the connection in Play and emit no semantic
event. The existing C4 test continues to prove that direct required-codec use rejects all optional
common-service identities.

After the fix, the exact 26.2 Fabric client passed the stronger Ferrite acceptance sequence:

- reached `PLAY` and then `screenType=NONE`, rather than accepting `LevelLoadingScreen`;
- observed complete nearby `minecraft:stone` terrain around `(8, 65, 8)`;
- remained in Play for more than 40 additional client ticks with the player on ground;
- retained exactly one server session and 25 Region authorities; and
- captured a 1708×960 gameplay framebuffer with SHA-256
  `dd40f050c8a8b9f38c5ae45ac12aa0e35efe685e3e65e0a41bd7f3f59a1fbfd6`.

The reproducible Java scenario and its secret-free evidence-bundle contract are committed by
`G02-P4-B2`; this remediation commit contains only the protocol boundary fix and its focused test.

## Verification

```text
cargo test -p ferrite-protocol --test c1 server_connection -- --nocapture
cargo test -p ferrite-protocol --test c4 play_serverbound_common_services -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```
