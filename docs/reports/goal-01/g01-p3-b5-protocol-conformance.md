# G01-P3-B5 Protocol Conformance Report

## Result

`G01-P3-B5` passed its automated C0/C1 suites and an exact unmodified-client probe on
2026-07-29. The graphical client reached Play and acknowledged teleport challenge `1`.

## Locked inputs

| Input | Observation |
|---|---|
| Minecraft version | Java Edition `26.2` |
| Client artifact | 39,193,383 bytes |
| Client SHA-1 | `2dc72797acbc1b63fc16a11c4ac393605f453754` |
| Registry source | Ignored locked `registries.json` plus extracted locked server data |
| Known pack | `minecraft:core:26.2` |
| Runtime | Eclipse Temurin `25.0.4+7`, Windows x86-64 |

The JRE, client, libraries, assets, registry report, extracted server data, and runtime logs remain
under ignored `target/` storage. They are not repository evidence by presence; the committed probe
checks the client length and SHA-1 before listening and fails closed on an incomplete trace.

## Automated evidence

`cargo run -q -p protocol-conformance -- run` passed:

- four independent C0/C1 golden exchanges;
- seven malformed sessions;
- seven half-duplex transition checks;
- 34 ordered configuration packets.

`cargo run -q -p protocol-conformance -- tcp-smoke` passed separate loopback status and offline
login connections and required the Play teleport acknowledgement.

Workspace tests also rebuilt the exact-client projection and resolved 697 tag definitions across 15
network registries, including 224 item tags. This prevents an empty-tag fixture from satisfying the
automated projection check.

## Exact-client observation

The verified client used Quick Play to connect directly to `127.0.0.1:25565`. The probe observed:

| Boundary | Result |
|---|---|
| Server-list status on this direct connection | Not observed |
| Offline login and compression | Passed |
| Configuration start | Passed |
| Exact known-pack selection | Passed |
| 29 synchronized registries | Passed |
| Static and dynamic tag closure | Passed |
| Finish Configuration acknowledgement | Passed |
| Play installation | Passed |
| Teleport challenge `1` acknowledgement | Passed |

The direct Quick Play run does not claim a graphical status exchange. Status request/ping/pong is
covered by the independent headless goldens and real loopback TCP smoke.

The successful probe wrote the ignored machine evidence:

```toml
schema = "ferrite-unmodified-client-smoke-v1"
minecraft_version = "26.2"
client_jar_sha1 = "2dc72797acbc1b63fc16a11c4ac393605f453754"
status_observed = false
login_configuration_observed = true
play_teleport_acknowledged = true
```

## Gate

The batch is accepted only after `cargo ferrite task check`, which runs the headless and loopback
protocol suites before format, Clippy, all workspace tests, offline reference verification, and
repository policy checks.
