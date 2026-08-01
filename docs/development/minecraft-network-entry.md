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
server ticks. `MinimalTerrain` remains a temporary formal projection provider until Goal 04's
production generator replaces it; it is not the durable world identity or metadata authority.

Before Region bootstrap, the gateway opens the configured overworld control-Region store under
`storage.root/worlds/<world-id>/dimensions/minecraft/overworld/regions/r.0.0`. A pristine store gets
one initial `ferrite:world-service/world_v1` record. Restart loads and verifies its world ID, seed,
generator, configured spawn, dimension catalog, Region mapping, chunk format, and content manifest.
Unsupported versions, corrupt commits, mismatched configuration, pre-existing stores without a
committed metadata record, and symlinked path components fail before the process can become ready.

Every formal composite tick leaves one bounded immutable continuity capture per owned Region.
Configured autosave intervals commit full Region recovery points in stable order, with the
overworld control Region last; its committed tick is the published world checkpoint. Startup loads
that prefix with `load_at_or_before`, validates later committed frames, and permits at most one
autosave interval of unpublished successors before bounded no-input catch-up. Clean drain publishes
one final capture, accounts for the synchronous commit as pending durable work, flushes every Region,
and releases Region authorities only after the control checkpoint receipt succeeds.

Installed Play sessions contribute their bounded `PlayerView` and `PlayerSimulation` tickets to one
formal ticket resolver. The resolver routes each demanded chunk to its mapping-owned Region, admits
bounded generation work, validates request ID, Region activation generation, source revision,
target status, and content manifest before publication, and derives accessible/block-ticking/entity-
ticking activity from the strongest ticket. Generation work may execute outside the authority
thread, but until versioned continuation is introduced it must publish before that tick's composite
continuity commit; an in-flight marker is rejected rather than written as an unrecoverable promise.

When the last ticket disappears, active chunks demote before receiving an identity-fenced unload
token. They remain resident until the full composite recovery point is durably committed and the
exact Region receipt is returned. A new matching demand cancels the pending unload. Receipt digest,
persistence revision, committed tick, Region generation, and captured world-service records are all
checked before memory is released.

The formal generation worker uses the configured `ferrite:overworld_v1` seed. Named height, detail,
temperature, humidity, and cave streams produce replay-stable biome cells, density terrain, surface,
carvers, bounded surface features, and spawn-headroom validation directly in the authoritative
`ChunkColumn`. Work is capped at four completed stage requests per gateway tick. These generated
columns are durable now, but Play terrain still comes from the temporary `MinimalTerrain` adapter;
P2-B4 owns switching projection, registry mapping, heightmaps, lighting payloads, and unload packets
to committed generated columns.

## Tick and shutdown behavior

The gateway advances the Region runner at 20 ticks per second with bounded catch-up. Player input
targets the next uncommitted tick. After commit, player state, block results, recenter events, and
the next flow-controlled chunk batch are projected back to each connection.

The initial local world preloads 25 version-1 Region authorities around the configured spawn in the
configured world identity. Their count is visible in lifecycle status. Until the Goal 04 collision
batch, player movement uses the same flat-world surface at feet Y 64 that is projected in terrain,
so collision and flying checks agree with the client-visible world. Drain
first drops the listener and closes admission, sends a bounded
Play disconnect where possible, routes each semantic leave at the next uncommitted tick, and only
then releases Region authorities. `NodeProcess` cannot reach `drained` until sessions and Region
authority both reach zero.

## Verification

The committed integration test starts `NodeProcess`, performs a real framed status/ping exchange,
holds a TCP session across repeated process polls, and proves graceful session/Region drain. The
formal persistence test runs two-tick autosaves, drains all 25 Region stores, verifies world/level/
simulation records in the control store, restarts at the exact committed tick, and rejects a
corrupted control log. The
external C2 client command targets an already running formal server:

```text
protocol-conformance connect-c2 127.0.0.1:25565
```

It completes offline login, configuration, Play correction acknowledgement, terrain batch
feedback, `PlayerLoaded`, movement, and `ClientTickEnd`. Graphical acceptance uses the locked
unmodified HMCL-launched 26.2 client and the external full registry/tag projection.
