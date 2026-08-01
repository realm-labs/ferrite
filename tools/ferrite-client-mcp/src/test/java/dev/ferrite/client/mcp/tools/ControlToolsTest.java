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
            "set_sprinting",
            "attack",
            "use_item",
            "select_hotbar",
            "drop_item",
            "swap_hands",
            "send_chat",
            "open_inventory",
            "close_screen",
            "move_cursor",
            "click_slot"
        }) {
            assertTrue(tools.find(name).isPresent(), name);
        }
    }

    @Test
    void queuesInteractionToolsAndRejectsUnsafeChat() {
        assertFalse(call("attack", actionId("attack-1")).error());

        JsonObject use = actionId("use-1");
        use.addProperty("ticks", 20);
        assertFalse(call("use_item", use).error());

        JsonObject excessiveUse = actionId("use-too-long");
        excessiveUse.addProperty("ticks", 21);
        assertTrue(call("use_item", excessiveUse).error());

        JsonObject command = actionId("chat-command");
        command.addProperty("message", "/give @s stone");
        assertTrue(call("send_chat", command).error());

        JsonObject chat = actionId("chat-ordinary");
        chat.addProperty("message", "ferrite reference hello");
        assertFalse(call("send_chat", chat).error());
    }

    @Test
    void validatesHotbarAndSingleClickSchemas() {
        JsonObject hotbar = actionId("hotbar-8");
        hotbar.addProperty("slot", 8);
        assertFalse(call("select_hotbar", hotbar).error());

        JsonObject invalidHotbar = actionId("hotbar-9");
        invalidHotbar.addProperty("slot", 9);
        assertTrue(call("select_hotbar", invalidHotbar).error());

        JsonObject invalidDrop = actionId("drop-with-duration");
        invalidDrop.addProperty("ticks", 2);
        assertTrue(call("drop_item", invalidDrop).error());
        assertFalse(call("swap_hands", actionId("swap-1")).error());
    }

    @Test
    void validatesGuiCoordinatesAndRevisionFields() {
        assertFalse(call("open_inventory", actionId("open-inventory")).error());
        assertFalse(call("close_screen", actionId("close-inventory")).error());

        JsonObject cursor = actionId("cursor-center");
        cursor.addProperty("x", 160);
        cursor.addProperty("y", 120);
        assertFalse(call("move_cursor", cursor).error());

        JsonObject nonFinite = actionId("cursor-nan");
        nonFinite.addProperty("x", Double.NaN);
        nonFinite.addProperty("y", 0);
        assertTrue(call("move_cursor", nonFinite).error());

        JsonObject click = actionId("pickup-slot");
        click.addProperty("containerId", 0);
        click.addProperty("stateId", 4);
        click.addProperty("slot", 36);
        click.addProperty("button", 0);
        click.addProperty("input", "PICKUP");
        assertFalse(call("click_slot", click).error());

        click.addProperty("input", "QUICK_CRAFT");
        click.addProperty("actionId", "unsupported-click");
        assertTrue(call("click_slot", click).error());
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
