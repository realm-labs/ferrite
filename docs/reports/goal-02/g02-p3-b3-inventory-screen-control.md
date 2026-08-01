# G02-P3-B3 — Inventory screen control

## Outcome

The client MCP now completes the minimum GUI surface with `open_inventory`, `close_screen`,
`move_cursor`, and `click_slot`.

- Inventory opening is one ordinary inventory-key click consumed by Minecraft's own key handler.
- Screen closing invokes the current screen's normal `onClose` callback.
- Cursor movement accepts finite current GUI coordinates, rejects points outside the initialized
  screen, converts them to raw window coordinates, and moves the native GLFW cursor.
- Slot clicks require an actual open `AbstractContainerScreen`, active player and game mode, a valid
  active slot, mouse button 0 or 1, and either `PICKUP` or `QUICK_MOVE`.
- Every slot request carries the observed container ID and menu state revision. Both must still
  equal the live menu on the client thread before the original `handleContainerInput` path runs.
  Stale automation therefore fails before local prediction or packet production.

All four operations use the shared action IDs, bounded queue, client-thread receipts, and shutdown
rules from P3-B1. No screen object, menu contents, carried stack, or player inventory is directly
mutated by the MCP.

## Exact-client evidence

The graphical Fabric client connected by Quick Play to the locked original 26.2 server in an
isolated survival flat world.

1. `open_inventory` entered the normal key path. The later observation showed a real
   `InventoryScreen`, title `Crafting`, GUI size 427×240, container ID 0, state revision 1, and 46
   slots.
2. `move_cursor` accepted the GUI center coordinates through the client-thread queue and applied
   the native window position without a client error.
3. A slot-0 pickup carrying container 0 but deliberately stale revision 2 changed from `Queued` to
   `Rejected` at the next tick with `container ID or state revision is stale`.
4. The same slot-0 pickup with live revision 1 was applied and satisfied at tick 825 through
   `MultiPlayerGameMode.handleContainerInput`.
5. `close_screen` used the screen callback, and the tick-fenced wait reached `screenType=NONE` at
   tick 830 while the connection remained in Play.

The sequence used returned acceptance ticks for every wait. It did not synthesize a mouse packet,
invoke a server command, or bypass the current menu revision.

## Verification

The batch's containing commit runs:

```text
JAVA_HOME=<local-jdk-25> ./gradlew --no-daemon check build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Focused tests cover tool discovery, non-finite cursor rejection, coordinate schemas, slot and
button bounds, accepted input kinds, unsupported quick-craft rejection, and shared queue behavior.
