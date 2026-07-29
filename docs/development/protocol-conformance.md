# Minecraft 26.2 C0/C1 Conformance

`G01-P3-B5` closes the first protocol phase with executable tests at three distinct boundaries.
The repository gate runs both fully automated boundaries; the exact-client probe is
operator-assisted because the unmodified graphical client is an external licensed artifact.

## Headless conformance

```sh
cargo run -p protocol-conformance -- run
```

This suite drives the actual bounded `ServerConnection` and checks:

- independent C0 intention, request, ping, and pong goldens;
- fragmented input and send-before-close behavior;
- zero, over-wide, unknown-ID, trailing, truncated, duplicate-hello, and early-acknowledgement
  sessions, including terminal fault cleanup;
- compression installation only after the negotiation frame is sent;
- login and configuration terminal packets on the correct directional state;
- brand, feature, known-pack, all 29 registry records, tags, and finish order;
- the semantic admission boundary before serverbound Play is installed.

The command reports stable counts and exits nonzero on the first mismatch.

## Loopback TCP smoke

```sh
cargo run -p protocol-conformance -- tcp-smoke
```

This suite uses real loopback TCP sockets and independent client/server stream decoders. It opens
one status connection and one offline-login connection, performs compression and configuration,
sends the locked C1 Play-entry frames, and requires serverbound teleport challenge `1`. Receiving
that acknowledgement proves that the client side decoded Play login and the initial position
correction. Socket reads and writes are bounded by five-second deadlines.

Both commands run from `cargo ferrite task check`, before formatting, Clippy, workspace tests, and
offline reference verification.

## Exact unmodified-client probe

```sh
cargo run -p protocol-conformance -- vanilla-probe \
  --client-jar target/mc-reference/26.2/client.jar \
  --registry-report target/mc-reference/26.2/generated/reports/registries.json
```

The probe refuses to listen unless `client.jar` is exactly 39,193,383 bytes and has SHA-1
`2dc72797acbc1b63fc16a11c4ac393605f453754`. It reconstructs static entries in official protocol-ID
order from the ignored locked report and discovers synchronized data-pack keys from the locked
server artifact. Data-pack keys use a bounded canonical order, with the projected overworld
dimension fixed at raw ID zero, and are marked as supplied by the exact
`minecraft:core:26.2` known pack. It also resolves the base pack's nested tag references into 697
wire tag definitions across 15 synchronized or static registries. Unknown server-only tag
directories are excluded because the client has no matching network registry. No Mojang registry
report, extracted data, runtime dependency, or client artifact is committed.

After the message prints, launch that verified client through its normal launcher and connect to
the displayed endpoint. The probe accepts a separate server-list status connection if the client
makes one, then requires offline login, configuration finish, Play entry, and the client's
teleport acknowledgement. Only that acknowledgement writes
`target/protocol-conformance/vanilla-c0-c1.toml`; timeout, a wrong client artifact, malformed
registry data, or an incomplete trace produces no success evidence.

The locked artifact and registry projection are rebuilt during workspace tests. The graphical
launch itself is intentionally not part of the unattended repository gate; final Goal acceptance
reruns the probe on the supported client/platform matrix and records those external observations.

The first exact-client run is recorded in the
[G01-P3-B5 conformance report](../reports/goal-01/g01-p3-b5-protocol-conformance.md). An unmodified
Windows client completed offline login, configuration, Play installation, and teleport challenge
`1`; Quick Play connected directly, so status discovery remains independently covered by the
headless and loopback suites rather than being claimed from that graphical run.
