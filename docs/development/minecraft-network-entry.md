# Formal Minecraft network entry

The production `ferrite-server` process owns the Minecraft Java 26.2 listener. The listener is no
longer a port reservation or a conformance-only probe: each accepted socket is registered with the
node lifecycle, driven through the protocol state machine, normalized through `SessionBridge`, and
routed into the local Region runner.

## Configuration and compatibility data

The closed server schema-2 `[minecraft]` table remains:

```toml
[minecraft]
enabled = true
bind = "127.0.0.1:25565"
registry_report = "target/mc-reference/26.2/generated/reports/registries.json" # optional
```

`registry_report` selects an external Mojang-generated 26.2 registry report. Ferrite resolves the
matching extracted `server-classes/data/minecraft` tree, validates all synchronized registries,
resolves the complete tag closure, and builds connection-local Play raw-ID tables. These artifacts
remain ignored and are never copied into the repository or runtime image.

When the field is absent, the server uses a compact project-owned bootstrap suitable for status,
headless conformance, and deployments that have not provisioned the compatibility data. Exact
unmodified-client acceptance is performed with the locked external report and data tree.

Schema 2 additionally requires the responsibility-owned `[world]`, `[world.spawn]`, and
`[world.save]` tables defined by the
[durable world production contract](durable-world-production.md). Schema 1 is accepted only through
the deterministic migration boundary; `ferrite-server --config <old> --migrate-config <new>` writes
one new canonical file and refuses to overwrite an existing target.

## Connection ownership

Each connection has bounded read work, protocol events, outbound frames, pending partial writes,
and chunk state. Nonblocking reads and writes are polled by the immutable server process; partial
writes retain their exact protocol sequence until `outbound_sent` commits the transition.
Malformed input and projection failures terminate only the owning connection and remain available
as the process's last-session diagnostic; they do not stop unrelated sessions or the node.

The state path is:

```text
TCP accept
  -> lifecycle admission
  -> handshake virtual-host route
  -> offline login admission / duplicate resolution
  -> configuration registry and tag synchronization
  -> semantic SessionBridge join
  -> structured Play entry projection
  -> JavaPlayerConnection
  -> Region command routing and committed projection
```

The Play entry projection is structured protocol data. Production code does not send the static
hex frames used by historical conformance fixtures. Initial terrain is projected from
`MinimalTerrain` through `ClientChunkSession`, honors client batch feedback, and continues across
server ticks.

## Tick and shutdown behavior

The gateway advances the Region runner at 20 ticks per second with bounded catch-up. Player input
targets the next uncommitted tick. After commit, player state, block results, recenter events, and
the next flow-controlled chunk batch are projected back to each connection.

The initial local world preloads 25 version-1 Region authorities around spawn. Their count is
visible in lifecycle status. Player movement uses the same flat-world surface at feet Y 64 that is
projected in terrain, so collision and flying checks agree with the client-visible world. Drain
first drops the listener and closes admission, sends a bounded
Play disconnect where possible, routes each semantic leave at the next uncommitted tick, and only
then releases Region authorities. `NodeProcess` cannot reach `drained` until sessions and Region
authority both reach zero.

## Verification

The committed integration test starts `NodeProcess`, performs a real framed status/ping exchange,
holds a TCP session across repeated process polls, and proves graceful session/Region drain. The
external C2 client command targets an already running formal server:

```text
protocol-conformance connect-c2 127.0.0.1:25565
```

It completes offline login, configuration, Play correction acknowledgement, terrain batch
feedback, `PlayerLoaded`, movement, and `ClientTickEnd`. Graphical acceptance uses the locked
unmodified HMCL-launched 26.2 client and the external full registry/tag projection.
