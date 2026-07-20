# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CONTAINER-001` — The server replays predicted clicks before choosing delta or full correction

**Parent:** `ITM-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — packet admission, stale-state handling, remote mirrors, state-ID changes,
listener order and client correction boundaries are explicit in the locked server and client control
flow.

**Applies when:**

The client predicts a menu click and submits a `ContainerInput`, or menu/container state changes
while a player is viewing it.

**Authoritative state:**

The server player's current menu object and `containerId`; its ordered slots, carried stack and data
slots; 15-bit `stateId`; local listener snapshots; hashed remote slot/carried mirrors; `stillValid`;
spectator/death state. Client slots are only speculative mirrors.

**Transition and ordering:**

The client first copies every menu slot, invokes the same `clicked(slot,button,input,player)` state
machine locally, hashes only slots whose full stack value changed, hashes the resulting carried
stack, and sends those predictions with the pre-click menu state ID. The server processes the packet
on its level thread, resets idle time, and then performs this sequence: (1) ignore a mismatching
container ID; (2) for spectator or dead/dying player send all authoritative data and do not click;
(3) reject an invalid `stillValid` menu; (4) reject a slot index for which `isValidSlotIndex` is
false; (5) remember whether the packet state ID differs; (6) suppress outbound remote comparisons;
(7) execute the click against current server state even when the ID was stale; (8) replace remote
slot hashes for every client-reported changed index and replace the remote carried hash, without
changing authoritative stacks; (9) resume remote comparisons; (10) if stale, run
`broadcastFullState`, otherwise `broadcastChanges`. Anchors:
`net.minecraft.client.multiplayer.MultiPlayerGameMode#handleContainerInput(int,int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClick(net.minecraft.network.protocol.game.ServerboundContainerClickPacket)`
and
`net.minecraft.world.inventory.AbstractContainerMenu#clicked(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`.

**Branches and aborts:**

A wrong menu ID, invalid menu or rejected slot produces no click. Spectator/dead sends a full
snapshot. A stale state ID does **not** reject or roll back the operation: it selects a
post-operation full snapshot. `clicked` converts any operation exception into a reported crash
containing menu type/class, slot count, slot, button and input. The packet admits at most 128
changed-slot hash entries. Dedicated menu-button packets follow `ITM-CONTAINER-CONTROL-001`, not
this click packet.

**Constants and randomness:**

`stateId = (stateId + 1) & 32767`. A full-content packet increments it once; each emitted individual
slot delta increments it once. Cursor and data-slot deltas do not increment it. Slot and button are
checked-cast by the client to signed 16-bit and signed 8-bit values. These algorithms consume no
RNG.

**Side effects:**

The click may change authoritative slots/cursor and invoke hooks. `broadcastChanges` visits slots in
ascending menu index, triggers local listeners on full `ItemStack.matches` inequality, then
compares/sends each remote slot, then the carried stack, then data slots in ascending index.
`broadcastFullState` refreshes local slot/data listeners before one complete content snapshot and
subsequent data packets. The client applies a slot/content packet only to inventory menu ID `0` or
the matching current menu; full content overwrites all listed slots, cursor and state ID. A cursor
packet overwrites the current cursor except while the creative inventory screen owns it.

**Gates:**

Level-thread dispatch, current container ID, spectator/dead state, `stillValid`, slot admission,
input-specific gates in `ITM-CONTAINER-CLICK-001`, feature flags for item click overrides, and
client creative-screen special handling.

**Boundary cases and quirks:**

`isValidSlotIndex` explicitly accepts `-1` and `-999`, and otherwise tests only `index < slotCount`;
it does not reject other negative values. Inputs that subsequently index the slot list can therefore
throw. `-999` has outside-click meaning only in the pickup branch. Client changed-slot hashes are
prediction baselines, not validation evidence and never authorize an item mutation. With two
viewers, the first server-thread transaction and its container callbacks are visible to the second
transaction before that second click runs.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`;
`net.minecraft.world.inventory.AbstractContainerMenu#isValidSlotIndex(int)`,
`net.minecraft.world.inventory.AbstractContainerMenu#broadcastChanges()`,
`net.minecraft.world.inventory.AbstractContainerMenu#broadcastFullState()`,
`net.minecraft.world.inventory.AbstractContainerMenu#incrementStateId()`,
`net.minecraft.server.level.ServerPlayer$1#sendInitialData(net.minecraft.world.inventory.AbstractContainerMenu,java.util.List,net.minecraft.world.item.ItemStack,int[])`,
`net.minecraft.server.level.ServerPlayer$1#sendSlotChange(net.minecraft.world.inventory.AbstractContainerMenu,int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.server.level.ServerPlayer$1#sendCarriedChange(net.minecraft.world.inventory.AbstractContainerMenu,net.minecraft.world.item.ItemStack)`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerSetSlot(net.minecraft.network.protocol.game.ClientboundContainerSetSlotPacket)`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleContainerContent(net.minecraft.network.protocol.game.ClientboundContainerSetContentPacket)`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleSetCursorItem(net.minecraft.network.protocol.game.ClientboundSetCursorItemPacket)`;
`EXP-ITM-002`.

**Test vectors:**

Current ID/current state with one slot delta; stale state with a legal pickup (the pickup commits,
followed by full contents); wrong menu ID; spectator and dead player; invalid `stillValid`; indices
`slotCount`, `-999`, `-1` and `-2`; 128 changed hashes; simultaneous viewers; state wrap
`32767 -> 0`; one action changing two slots plus cursor, asserting two successive slot state IDs and
no cursor increment.
