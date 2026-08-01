# G02-P3-B2 — Client interaction paths

## Outcome

The pure-Java client MCP now exposes `attack`, `use_item`, `select_hotbar`, `drop_item`,
`swap_hands`, and `send_chat`. Attack, use, drop, and swap inject one click into the corresponding
26.2 `KeyMapping`; Minecraft's own `handleKeybinds` consumes it and owns target selection,
prediction, animation, and packet construction. Ferrite does not construct gameplay packets.

The only mixin is a narrow accessor for `KeyMapping.clickCount`. It does not replace or inject into
Minecraft's gameplay methods. Held attack/use keys reuse the P3-B1 ownership, tick expiry,
disconnect, world-change, and shutdown release guarantees.

`select_hotbar` runs on the client thread through `Inventory.setSelectedSlot`, the same final
operation used by the locked client's ordinary hotbar-key branch. `send_chat` calls the active
client connection's public ordinary-chat method. Blank text, more than 256 Unicode code points,
control characters, and every slash-prefixed command are rejected before queueing.

## Bounds and rejection

- attack and use holds default to one tick and are limited to 20 ticks;
- drop and hand swap are single clicks and reject duration arguments;
- hotbar slots are zero-based and limited to 0–8;
- all calls retain the action-ID, bounded-queue, receipt, gameplay-focus, and shutdown rules from
  P3-B1;
- server commands remain unavailable: `send_chat` rejects `/...` rather than routing it to
  `sendCommand`.

## Exact-client packet-producing evidence

The graphical Fabric client connected by Quick Play to the repository-locked original 26.2 server
jar in an isolated survival flat world. The MCP used only client tools; no server command, RCON,
direct inventory edit, or hand-built packet prepared the result.

1. A 20-tick normal attack click/hold was accepted at client tick 968, applied at 969, and released
   at 989. The client removed the targeted surface block through ordinary mining and later observed
   one `minecraft:dirt` in inventory slot 0. The crosshair then targeted the next dirt block below.
2. Hotbar selection completed on the client thread and the later observation reported selected
   slot 8; selecting slot 0 restored the dirt as the active stack.
3. A normal swap-hand click moved the dirt from inventory slot 0 to offhand slot 40. A second swap
   returned it, and a normal drop click changed the later inventory observation to empty.
4. A two-tick use-item action, empty-hand swap, and empty selected-slot drop all completed through
   the same original key handler without client-control errors.
5. `send_chat` completed at client tick 995. The original server logged the ordinary player message
   `ferrite mcp interaction verified`, proving that the client connection produced and the server
   accepted the chat packet.
6. `client_errors` remained empty after the interaction sequence, including the runtime mixin
   accessor path.

The deterministic script fenced every later observation using the receipt's `acceptedTick`, not a
sleep. Because an HTTP request can arrive between START and END tick events, callers must wait for
an observation strictly after the returned acceptance tick before asserting application effects.

## Verification

The batch's containing commit runs:

```text
JAVA_HOME=<local-jdk-25> ./gradlew --no-daemon check build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Focused tests cover discovery, duration bounds, single-click schemas, hotbar bounds, normal chat,
slash-command rejection, finite action receipts, and the shared input queue.
