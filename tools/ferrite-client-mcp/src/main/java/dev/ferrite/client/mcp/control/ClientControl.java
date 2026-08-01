package dev.ferrite.client.mcp.control;

import java.time.Duration;

/** Thread-safe control boundary implemented by the Minecraft client source set. */
public interface ClientControl extends AutoCloseable {
    ActionReceipt submit(ClientAction action);

    ActionReceipt awaitApplied(String actionId, Duration timeout) throws InterruptedException;

    ActionReceipt status(String actionId);

    @Override
    void close();

    static ClientControl unavailable() {
        return new ClientControl() {
            @Override
            public ActionReceipt submit(ClientAction action) {
                return new ActionReceipt(
                        action.actionId(),
                        action.actionName(),
                        ActionState.REJECTED,
                        0,
                        null,
                        0L,
                        "Minecraft client control is unavailable");
            }

            @Override
            public ActionReceipt awaitApplied(String actionId, Duration timeout) {
                return status(actionId);
            }

            @Override
            public ActionReceipt status(String actionId) {
                return new ActionReceipt(
                        actionId,
                        "unknown",
                        ActionState.REJECTED,
                        0,
                        null,
                        0L,
                        "Minecraft client control is unavailable");
            }

            @Override
            public void close() {}
        };
    }
}
