package dev.ferrite.client.mcp.control;

import java.util.Objects;

/** Immutable action acknowledgement copied between the client and MCP threads. */
public record ActionReceipt(
        String actionId,
        String action,
        ActionState state,
        long acceptedTick,
        Long appliedTick,
        Long completedTick,
        String detail) {
    public ActionReceipt {
        Objects.requireNonNull(actionId, "actionId");
        Objects.requireNonNull(action, "action");
        Objects.requireNonNull(state, "state");
        Objects.requireNonNull(detail, "detail");
        if (acceptedTick < 0 || (appliedTick != null && appliedTick < acceptedTick)) {
            throw new IllegalArgumentException("action receipt ticks are inconsistent");
        }
        if (completedTick != null && appliedTick != null && completedTick < appliedTick) {
            throw new IllegalArgumentException("completion precedes application");
        }
    }
}
