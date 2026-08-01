package dev.ferrite.client.mcp.control;

import dev.ferrite.client.mcp.observation.ClientObservationStore;
import java.time.Duration;
import java.util.EnumMap;
import java.util.HashSet;
import java.util.Map;
import java.util.Set;
import net.minecraft.client.KeyMapping;
import net.minecraft.client.Minecraft;
import net.minecraft.client.player.LocalPlayer;

/** Owns bounded normal key state and view actions at client-tick boundaries. */
public final class MinecraftActionController implements ClientControl {
    private static final int MAXIMUM_ACTIONS_PER_TICK = 64;

    private final Minecraft client;
    private final ClientObservationStore observations;
    private final ClientActionQueue queue = new ClientActionQueue();
    private final EnumMap<ControlledInput, ActiveInput> activeInputs =
            new EnumMap<>(ControlledInput.class);

    private Object priorLevel;
    private long clientTick;
    private volatile boolean closed;

    public MinecraftActionController(Minecraft client, ClientObservationStore observations) {
        this.client = client;
        this.observations = observations;
    }

    @Override
    public ActionReceipt submit(ClientAction action) {
        return queue.submit(action);
    }

    @Override
    public ActionReceipt awaitApplied(String actionId, Duration timeout) throws InterruptedException {
        return queue.awaitApplied(actionId, timeout);
    }

    @Override
    public ActionReceipt status(String actionId) {
        return queue.status(actionId);
    }

    public void tick(Minecraft tickingClient) {
        clientTick++;
        queue.advanceTick(clientTick);
        if (tickingClient != client || !client.isSameThread()) {
            observations.recordError(
                    clientTick, "control", "control tick attempted on an unexpected client thread");
            releaseAndCancel("client thread boundary changed");
            return;
        }
        if (closed) {
            releasePhysicalInputs();
            return;
        }

        Object currentLevel = client.level;
        if (priorLevel != null && currentLevel != priorLevel) {
            releaseAndCancel("world or connection changed");
        }
        priorLevel = currentLevel;
        if (client.player == null && !activeInputs.isEmpty()) {
            releaseAndCancel("local player is no longer available");
        }

        expireInputs();
        for (int count = 0; count < MAXIMUM_ACTIONS_PER_TICK; count++) {
            ClientAction action = queue.poll(clientTick);
            if (action == null) {
                break;
            }
            apply(action);
        }
    }

    @Override
    public void close() {
        if (closed) {
            return;
        }
        closed = true;
        queue.close("client control shut down");
        if (client.isSameThread()) {
            releasePhysicalInputs();
        } else {
            client.execute(this::releasePhysicalInputs);
        }
    }

    private void apply(ClientAction action) {
        try {
            if (action instanceof ClientAction.ReleaseAll) {
                applyReleaseAll(action);
            } else if (!gameplayAvailable()) {
                queue.complete(
                        action.actionId(),
                        ActionState.REJECTED,
                        "gameplay requires a player, world, and no open screen");
            } else if (action instanceof ClientAction.Look look) {
                applyLook(look);
            } else if (action instanceof ClientAction.Inputs inputs) {
                applyInputs(inputs);
            }
        } catch (RuntimeException error) {
            observations.recordError(
                    clientTick,
                    "control",
                    error.getClass().getSimpleName() + ": " + error.getMessage());
            queue.complete(action.actionId(), ActionState.REJECTED, "client action failed safely");
            releaseAction(action.actionId(), ActionState.CANCELLED, "action failed safely");
        }
    }

    private boolean gameplayAvailable() {
        return client.player != null && client.level != null && client.gui.screen() == null;
    }

    private void applyLook(ClientAction.Look action) {
        LocalPlayer player = client.player;
        if (player == null) {
            throw new IllegalStateException("local player disappeared");
        }
        float yaw = action.relative() ? player.getYRot() + action.yaw() : action.yaw();
        float pitch = action.relative() ? player.getXRot() + action.pitch() : action.pitch();
        player.setYRot(wrapDegrees(yaw));
        player.setXRot(Math.clamp(pitch, -90.0f, 90.0f));
        queue.markApplied(action.actionId());
        queue.complete(action.actionId(), ActionState.SATISFIED, "view rotation applied");
    }

    private void applyInputs(ClientAction.Inputs action) {
        queue.markApplied(action.actionId());
        if (!action.down()) {
            Set<String> owners = new HashSet<>();
            for (ControlledInput input : action.inputs()) {
                ActiveInput active = activeInputs.get(input);
                if (active != null) {
                    owners.add(active.actionId());
                }
            }
            owners.forEach(owner -> releaseAction(
                    owner, ActionState.CANCELLED, "input explicitly released by a later action"));
            action.inputs().forEach(this::releaseInputWithoutOwner);
            queue.complete(action.actionId(), ActionState.SATISFIED, "input released");
            return;
        }

        long releaseTick = clientTick + action.ticks();
        for (ControlledInput input : action.inputs()) {
            key(input).setDown(true);
            activeInputs.put(input, new ActiveInput(action.actionId(), releaseTick));
        }
    }

    private void applyReleaseAll(ClientAction action) {
        queue.markApplied(action.actionId());
        Set<String> owners = activeActionIds();
        owners.forEach(owner -> releaseAction(
                owner, ActionState.CANCELLED, "all MCP-owned inputs were explicitly released"));
        releasePhysicalInputs();
        queue.cancelOutstandingExcept(
                action.actionId(), "cancelled by release_all_inputs");
        queue.complete(
                action.actionId(), ActionState.SATISFIED, "all MCP-owned inputs released");
    }

    private void expireInputs() {
        Set<String> expired = new HashSet<>();
        for (ActiveInput active : activeInputs.values()) {
            if (clientTick >= active.releaseTick()) {
                expired.add(active.actionId());
            }
        }
        expired.forEach(owner ->
                releaseAction(owner, ActionState.SATISFIED, "bounded input duration elapsed"));
    }

    private void releaseAndCancel(String detail) {
        releasePhysicalInputs();
        queue.cancelOutstanding(detail);
    }

    private void releaseAction(String actionId, ActionState state, String detail) {
        boolean found = false;
        for (Map.Entry<ControlledInput, ActiveInput> entry :
                Set.copyOf(activeInputs.entrySet())) {
            if (entry.getValue().actionId().equals(actionId)) {
                key(entry.getKey()).setDown(false);
                activeInputs.remove(entry.getKey());
                queue.releaseReservation(entry.getKey(), actionId);
                found = true;
            }
        }
        if (found && !queue.status(actionId).state().completed()) {
            queue.complete(actionId, state, detail);
        }
    }

    private void releasePhysicalInputs() {
        for (ControlledInput input : ControlledInput.values()) {
            key(input).setDown(false);
        }
        activeInputs.clear();
    }

    private void releaseInputWithoutOwner(ControlledInput input) {
        key(input).setDown(false);
        activeInputs.remove(input);
    }

    private Set<String> activeActionIds() {
        Set<String> actionIds = new HashSet<>();
        activeInputs.values().forEach(active -> actionIds.add(active.actionId()));
        return actionIds;
    }

    private KeyMapping key(ControlledInput input) {
        return switch (input) {
            case FORWARD -> client.options.keyUp;
            case BACKWARD -> client.options.keyDown;
            case LEFT -> client.options.keyLeft;
            case RIGHT -> client.options.keyRight;
            case JUMP -> client.options.keyJump;
            case SNEAK -> client.options.keyShift;
            case SPRINT -> client.options.keySprint;
        };
    }

    private static float wrapDegrees(float degrees) {
        float wrapped = degrees % 360.0f;
        if (wrapped >= 180.0f) {
            wrapped -= 360.0f;
        }
        if (wrapped < -180.0f) {
            wrapped += 360.0f;
        }
        return wrapped;
    }

    private record ActiveInput(String actionId, long releaseTick) {}
}
