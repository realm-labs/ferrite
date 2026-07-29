# G01-P4-B5 C2 Acceptance and Adversity Report

## Result

`G01-P4-B5` closes Phase 4. The unattended C2 TCP smoke, delayed/fragmented variant, exact
unmodified Minecraft Java 26.2 probe, malformed inputs, bounded backpressure, delayed chunk
feedback, and cross-Region local/Lattice scenario all pass.

The executable contract is recorded in
[C2 acceptance and adversity](../../development/c2-acceptance-and-adversity.md).

## Exact-client observation

The verified 39,193,383-byte client with SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754` used Quick Play against the C2 probe. It completed:

| Boundary | Result |
|---|---|
| Offline login, compression, and configuration | Passed |
| Complete synchronized registry/tag projection | Passed |
| Play teleport challenge acknowledgement | Passed |
| Full 24-section chunk/light batch decode | Passed |
| Chunk-batch feedback | Passed |
| `player_loaded` | Passed |
| Movement | Passed |
| Client tick end | Passed |

The run used Eclipse Temurin `25.0.4+7` on Windows x86-64. Runtime artifacts, assets, logs, and the
structured TOML evidence remain under ignored `target/` storage.

## Automated evidence

- the C2 loopback command performs the complete TCP exchange with independent codecs;
- a second exchange delays feedback and fragments compressed response frames into three-byte
  writes;
- delayed batch acknowledgement holds the one-batch bootstrap window and valid feedback reopens
  it;
- a one-command Region mailbox rejects movement without mutating connection state or replacing
  admitted work;
- truncated and trailing C2 bodies fail closed;
- the seven-tick two-Region scenario still produces the locked equal local/Lattice state and packet
  trace.

The acceptance commands are:

```text
cargo run -p protocol-conformance -- c2-smoke
cargo test -p protocol-conformance
cargo test -p ferrite-server-runtime --test playable_adversity
cargo run -p ferrite-cluster -- verify-playable
cargo ferrite task check
git diff --check
```

The exact-client probe was also run with `vanilla-c2-probe`; it is intentionally operator-assisted
and is not part of the unattended repository gate.

## Coverage boundary

Phase 4 proves the minimal playable C2 spine, not all audited Minecraft gameplay. Phase 5 begins
with the generated `simulation`, `blocks`, `environment`, and `redstone` slice batches. Protocol
coverage remains 14/44 required families until later C3 family batches are implemented.
