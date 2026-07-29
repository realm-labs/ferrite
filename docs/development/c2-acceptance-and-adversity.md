# C2 acceptance and adversity

`G01-P4-B5` closes the minimal playable phase with independent TCP, exact-client, malformed-input,
backpressure, delayed-feedback, and cross-Region evidence.

## Unattended C2 TCP smoke

```text
cargo run -p protocol-conformance -- c2-smoke
```

The server and independent client use real loopback TCP, compression, and separate stream
decoders. After the complete C0/C1 login and configuration exchange, the server sends:

1. the locked Play entry and teleport challenge;
2. cache center, view radius, and simulation distance;
3. an exact batch start;
4. one full 24-section chunk with biome and 26 sky/block light layers;
5. an exact one-chunk batch finish.

The client must return the exact teleport acknowledgement, chunk-batch feedback, `player_loaded`,
a movement packet, and client tick end. The command runs the feedback exchange twice: once normally
and once after a deliberate 25 ms delay with all four compressed response frames concatenated and
written in three-byte fragments. Both observations must be identical.

This smoke runs from `cargo ferrite task check` before format, Clippy, workspace tests, and offline
coverage verification.

## Exact unmodified-client probe

```text
cargo run -p protocol-conformance -- vanilla-c2-probe \
  --client-jar target/mc-reference/26.2/client.jar \
  --registry-report target/mc-reference/26.2/generated/reports/registries.json
```

The probe retains the C0/C1 artifact and registry checks: the client must be exactly 39,193,383
bytes with SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`, and configuration is reconstructed from the
locked registry report, extracted known-pack data, and complete tag closure. It then uses the same
C2 terrain sequence as the unattended smoke and refuses success unless every C2 response above is
observed.

The external graphical client remains an operator-launched licensed artifact and is not committed
or downloaded by the repository gate. The `G01-P4-B5` acceptance run used the normal 26.2 Quick
Play client with Eclipse Temurin `25.0.4+7` on Windows x86-64. The ignored evidence file recorded:

```toml
schema = "ferrite-unmodified-client-c2-smoke-v1"
minecraft_version = "26.2"
client_jar_sha1 = "2dc72797acbc1b63fc16a11c4ac393605f453754"
login_configuration_observed = true
play_teleport_acknowledged = true
chunk_batch_feedback_observed = true
player_loaded_observed = true
movement_observed = true
client_tick_end_observed = true
```

## Adverse and bounded behavior

The server-runtime integration suite additionally locks these outcomes:

| Case | Required outcome |
|---|---|
| Delayed initial chunk ACK | The initial one-batch window prevents another batch. |
| Valid delayed feedback | The bounded window reopens and the requested rate clamps normally. |
| Command mailbox full | Connection-side movement state rolls back; previously admitted authority work remains and commits. |
| C2 trailing or truncated body | Decode fails without semantic admission. |
| Cross-Region movement and edit | The player owner switches only after transfer commit; subsequent placement, rejection correction, and break converge identically through local and Lattice routing. |

Codec-family suites retain the exhaustive exceptional float, enum, VarInt, bounds, malformed,
ordering, acknowledgement, light, palette, and batch cases owned by `G01-P4-F001` through
`G01-P4-F005`.

## Phase boundary

This evidence closes the C2 minimal playable phase. It does not claim the complete survival
behavior denominator: generic block content, simulation, environment, redstone, inventories,
entities, world generation, and remaining C3/C4 services remain in Phases 5 through 9.
