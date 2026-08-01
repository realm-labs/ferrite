# G03-P4-B1 — Exact-client composite runtime acceptance

## Outcome

`Satisfied`. The repository-owned Goal 02 MCP launched the locked Minecraft Java 26.2 client with
Java 25 and drove the formal `ferrite-server` listener without clicks, direct player mutation,
server commands, or hand-built gameplay packets. One scenario now proves sustained Play, ordinary
movement, a committed Region transfer, post-commit block interaction, explicit unsupported
dispatch, visual terrain convergence, and clean process drain through the composite production
route.

The generated evidence bundle remains ignored below `target/client-mcp-evidence-g03-p4-b1-final2`.
This committed record contains the stable assertions and image digest without publishing Mojang
artifacts, the bearer secret, the isolated game directory, logs, or framebuffer bytes.

## Composite and transfer evidence

The client joined at `(8.5, 65.0, 8.5)` and reached Play with one formal session. Management status
reported 25 committed composite Region receipts at each observed server tick. Three bounded
200-client-tick forward-key actions moved the client normally to
`(8.5, 64.0, 137.58238116077894)`.

The server independently reported:

- stable player `cc678e2a0c613e9b9aa1814bd5a49b8b`;
- Region `(0, 0)` changing to `(0, 1)` at the 8-chunk/128-block mapping boundary;
- `region_transfers=1` and 25 same-tick composite Region commits;
- one active session, zero pending commits, and no lifecycle failure after the transfer;
- a clean `ferrite-server node=goal03-node stopped` terminal record after client cleanup.

The disconnect path now routes the leave command to the player's current Region. The first
diagnostic run exposed that the old path retained the initial join Region and faulted after a
transferred client disconnected; the corrected route closes the transferred authority before
drain.

## Block, unsupported, and visual evidence

At Region `(0, 1)` the ordinary client look action acquired `minecraft:stone` at `(8, 63, 137)`.
Holding the normal attack key produced committed block command sequence 1 with `tracking`; release
produced sequence 2 with `cleared`. A later radius-2 client-world observation still contained the
target as stone, proving client prediction converged with the authoritative minimal terrain.

The scenario then sent ordinary chat text. The formal application dispatch retained
`packet=ChatMessage`, `responsibility=chat-and-command`, and `disposition=unsupported`; it emitted no
false handled update. The client remained in Play for a further render fence.

The first post-transfer image revealed a real chunk-stream defect: recenter published cache center
and forget packets but did not enqueue entered chunks or resend the new center. The stream now
marks every entered chunk pending and requeues the center under the existing bounded capacity.
The final actual-framebuffer image visibly contains continuous flat stone terrain after transfer:

| Field | Value |
|---|---|
| Client tick | `798` |
| Dimensions | `1708×960` |
| PNG bytes | `549072` |
| SHA-256 | `ec2b20b08b2e438d6e2f5bdb16605873a42053b20e13a507d9d87175290e34a8` |

This is instrumented exact-client evidence. It does not replace the separate Goal 01 unmodified
client compatibility claim.

## Production observability

`GET /status` now adds one bounded `minecraft` snapshot to the existing lifecycle fields. It
contains the committed tick, number of composite Region commits, and one fixed-size record per
bounded active session: stable player, current Region, transfer count, latest dispatch, latest
unsupported dispatch, and latest committed block result. The endpoint remains read-only and cannot
drive gameplay or mutate authority.

The formal Region voxel bootstrap now initializes the same uniform stone sections projected by
`MinimalTerrain`. Block commands therefore read the state the client actually received instead of
an unrelated all-air authority.

## Reproduction

```text
cargo build -p ferrite-server
cd tools/ferrite-client-mcp
JAVA_HOME=<jdk-25> ./gradlew --no-daemon check build
<jdk-25>/bin/java \
  -jar build/libs/ferrite-client-mcp-0.1.0-SNAPSHOT-acceptance.jar \
  --workspace <workspace> \
  --java-home <jdk-25> \
  --mode ferrite
```

The batch also runs `cargo ferrite production verify`, `cargo ferrite source verify`, the complete
Java checks, all workspace format/lint/tests, and `git diff --check` before commit.
