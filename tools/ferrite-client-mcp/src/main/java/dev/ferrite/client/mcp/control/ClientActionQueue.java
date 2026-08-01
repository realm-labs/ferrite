package dev.ferrite.client.mcp.control;

import java.time.Duration;
import java.util.ArrayDeque;
import java.util.EnumMap;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.Objects;

/** Bounded synchronized action and receipt store shared with the client thread. */
public final class ClientActionQueue {
    public static final int MAXIMUM_PENDING_ACTIONS = 64;
    private static final int MAXIMUM_RECEIPTS = 256;

    private final ArrayDeque<ClientAction> pending = new ArrayDeque<>();
    private final LinkedHashMap<String, ActionReceipt> receipts = new LinkedHashMap<>();
    private final EnumMap<ControlledInput, String> reservations =
            new EnumMap<>(ControlledInput.class);
    private long currentTick;
    private boolean closed;

    public synchronized ActionReceipt submit(ClientAction action) {
        Objects.requireNonNull(action, "action");
        String validation = validateIdentity(action);
        if (validation != null) {
            return rejection(action, validation);
        }
        if (closed) {
            return rejection(action, "client control is shut down");
        }
        if (receipts.containsKey(action.actionId())) {
            return rejection(action, "actionId has already been used");
        }
        if (pending.size() >= MAXIMUM_PENDING_ACTIONS
                && !(action instanceof ClientAction.ReleaseAll)) {
            return rejection(action, "client action queue is full");
        }
        if (action instanceof ClientAction.Inputs inputs && inputs.down()) {
            for (ControlledInput input : inputs.inputs()) {
                if (reservations.containsKey(input)) {
                    return rejection(action, "input " + input.name() + " is already owned");
                }
            }
            inputs.inputs().forEach(input -> reservations.put(input, action.actionId()));
        }

        ActionReceipt receipt = new ActionReceipt(
                action.actionId(),
                action.actionName(),
                ActionState.QUEUED,
                currentTick,
                null,
                null,
                "queued for the Minecraft client thread");
        putReceipt(receipt);
        if (action instanceof ClientAction.ReleaseAll) {
            pending.addFirst(action);
        } else {
            pending.addLast(action);
        }
        notifyAll();
        return receipt;
    }

    public synchronized ClientAction poll(long clientTick) {
        advanceTick(clientTick);
        return pending.pollFirst();
    }

    public synchronized void advanceTick(long clientTick) {
        if (clientTick < currentTick) {
            throw new IllegalArgumentException("client control tick moved backwards");
        }
        currentTick = clientTick;
    }

    public synchronized ActionReceipt markApplied(String actionId) {
        ActionReceipt prior = requireReceipt(actionId);
        ActionReceipt applied = new ActionReceipt(
                prior.actionId(),
                prior.action(),
                ActionState.APPLIED,
                prior.acceptedTick(),
                currentTick,
                null,
                "applied on the Minecraft client thread");
        putReceipt(applied);
        notifyAll();
        return applied;
    }

    public synchronized ActionReceipt complete(
            String actionId, ActionState state, String detail) {
        if (!state.completed()) {
            throw new IllegalArgumentException("terminal receipt state required");
        }
        ActionReceipt prior = requireReceipt(actionId);
        releaseReservations(actionId);
        ActionReceipt completed = new ActionReceipt(
                prior.actionId(),
                prior.action(),
                state,
                prior.acceptedTick(),
                prior.appliedTick(),
                currentTick,
                detail);
        putReceipt(completed);
        notifyAll();
        return completed;
    }

    public synchronized void releaseReservation(ControlledInput input, String actionId) {
        reservations.remove(input, actionId);
    }

    public synchronized void cancelOutstanding(String detail) {
        cancelOutstandingExcept(null, detail);
    }

    public synchronized void cancelOutstandingExcept(String retainedActionId, String detail) {
        for (ActionReceipt receipt : Map.copyOf(receipts).values()) {
            if (!receipt.state().completed()
                    && !receipt.actionId().equals(retainedActionId)) {
                complete(receipt.actionId(), ActionState.CANCELLED, detail);
            }
        }
        pending.removeIf(action -> !action.actionId().equals(retainedActionId));
        reservations.clear();
    }

    public synchronized ActionReceipt awaitApplied(String actionId, Duration timeout)
            throws InterruptedException {
        long remainingNanos = timeout.toNanos();
        long deadline = System.nanoTime() + remainingNanos;
        ActionReceipt receipt = status(actionId);
        while (receipt.state() == ActionState.QUEUED && remainingNanos > 0) {
            long millis = Math.max(1, remainingNanos / 1_000_000L);
            wait(millis);
            receipt = status(actionId);
            remainingNanos = deadline - System.nanoTime();
        }
        return receipt;
    }

    public synchronized ActionReceipt status(String actionId) {
        ActionReceipt receipt = receipts.get(actionId);
        if (receipt == null) {
            return new ActionReceipt(
                    actionId,
                    "unknown",
                    ActionState.REJECTED,
                    currentTick,
                    null,
                    currentTick,
                    "unknown actionId");
        }
        return receipt;
    }

    public synchronized void close(String detail) {
        closed = true;
        cancelOutstanding(detail);
    }

    public synchronized int pendingCount() {
        return pending.size();
    }

    private static String validateIdentity(ClientAction action) {
        String id = action.actionId();
        if (id == null || id.isBlank() || id.length() > 64) {
            return "actionId must contain 1 to 64 characters";
        }
        for (int index = 0; index < id.length(); index++) {
            char character = id.charAt(index);
            if (!(Character.isLetterOrDigit(character)
                    || character == '-'
                    || character == '_'
                    || character == '.')) {
                return "actionId contains an unsupported character";
            }
        }
        return null;
    }

    private ActionReceipt rejection(ClientAction action, String detail) {
        return new ActionReceipt(
                Objects.requireNonNullElse(action.actionId(), "invalid"),
                action.actionName(),
                ActionState.REJECTED,
                currentTick,
                null,
                currentTick,
                detail);
    }

    private ActionReceipt requireReceipt(String actionId) {
        ActionReceipt receipt = receipts.get(actionId);
        if (receipt == null) {
            throw new IllegalArgumentException("unknown actionId: " + actionId);
        }
        return receipt;
    }

    private void releaseReservations(String actionId) {
        reservations.entrySet().removeIf(entry -> entry.getValue().equals(actionId));
    }

    private void putReceipt(ActionReceipt receipt) {
        receipts.put(receipt.actionId(), receipt);
        while (receipts.size() > MAXIMUM_RECEIPTS) {
            String eldest = receipts.keySet().iterator().next();
            receipts.remove(eldest);
        }
    }
}
