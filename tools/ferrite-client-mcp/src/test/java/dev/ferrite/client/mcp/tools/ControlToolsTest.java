package dev.ferrite.client.mcp.tools;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.capture.ScreenshotCapture;
import dev.ferrite.client.mcp.control.ActionReceipt;
import dev.ferrite.client.mcp.control.ActionState;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientActionQueue;
import dev.ferrite.client.mcp.control.ClientControl;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import java.time.Duration;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

final class ControlToolsTest {
    private ClientActionQueue queue;
    private ToolRegistry tools;

    @BeforeEach
    void createTools() {
        queue = new ClientActionQueue();
        tools = ToolRegistry.forClient(
                new ClientObservationStore(), ScreenshotCapture.unavailable(), control());
    }

    @Test
    void queuesBoundedMovementAndPublishesReceipts() {
        JsonObject arguments = new JsonObject();
        arguments.addProperty("actionId", "walk-forward");
        arguments.addProperty("forward", true);
        arguments.addProperty("ticks", 20);

        McpToolResult queued = call("hold_movement", arguments);
        assertFalse(queued.error());
        assertEquals("Queued", queued.structuredContent().get("state").getAsString());

        ClientAction action = queue.poll(4);
        assertEquals("walk-forward", action.actionId());
        queue.markApplied(action.actionId());
        assertEquals(
                "Applied",
                call("action_status", actionId("walk-forward"))
                        .structuredContent()
                        .get("state")
                        .getAsString());
    }

    @Test
    void rejectsOpposingMovementAndUnboundedHeldState() {
        JsonObject opposing = new JsonObject();
        opposing.addProperty("actionId", "opposing");
        opposing.addProperty("forward", true);
        opposing.addProperty("backward", true);
        opposing.addProperty("ticks", 1);
        assertTrue(call("hold_movement", opposing).error());

        JsonObject unboundedSneak = new JsonObject();
        unboundedSneak.addProperty("actionId", "sneak-forever");
        unboundedSneak.addProperty("enabled", true);
        assertTrue(call("set_sneaking", unboundedSneak).error());

        JsonObject excessiveJump = actionId("jump-too-long");
        excessiveJump.addProperty("ticks", 21);
        assertTrue(call("jump", excessiveJump).error());
    }

    @Test
    void rejectsNonFiniteLookAndAcceptsBoundedRelativeLook() {
        JsonObject invalid = actionId("bad-look");
        invalid.addProperty("yaw", Double.NaN);
        invalid.addProperty("pitch", 0);
        assertTrue(call("look", invalid).error());

        JsonObject valid = actionId("turn-left");
        valid.addProperty("yaw", -90);
        valid.addProperty("pitch", 10);
        valid.addProperty("relative", true);
        assertFalse(call("look", valid).error());
    }

    @Test
    void exposesEveryPhaseThreeBatchOneTool() {
        for (String name : new String[] {
            "wait_for_state",
            "release_all_inputs",
            "action_status",
            "look",
            "hold_movement",
            "jump",
            "set_sneaking",
            "set_sprinting"
        }) {
            assertTrue(tools.find(name).isPresent(), name);
        }
    }

    private ClientControl control() {
        return new ClientControl() {
            @Override
            public ActionReceipt submit(ClientAction action) {
                return queue.submit(action);
            }

            @Override
            public ActionReceipt awaitApplied(String actionId, Duration timeout)
                    throws InterruptedException {
                return queue.awaitApplied(actionId, timeout);
            }

            @Override
            public ActionReceipt status(String actionId) {
                return queue.status(actionId);
            }

            @Override
            public void close() {
                queue.close("closed");
            }
        };
    }

    private McpToolResult call(String name, JsonObject arguments) {
        return tools.find(name)
                .orElseThrow()
                .call(arguments, new ToolContext("2025-11-25"));
    }

    private static JsonObject actionId(String actionId) {
        JsonObject arguments = new JsonObject();
        arguments.addProperty("actionId", actionId);
        return arguments;
    }
}
