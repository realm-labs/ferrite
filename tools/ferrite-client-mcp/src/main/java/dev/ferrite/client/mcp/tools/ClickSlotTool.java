package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import java.util.Set;

/** Queues a revision-fenced pickup or quick-move operation on an open container slot. */
final class ClickSlotTool implements McpTool {
    private static final Set<String> ALLOWED =
            Set.of("actionId", "containerId", "stateId", "slot", "button", "input");

    private final ClientControl control;

    ClickSlotTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "click_slot";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        properties.add("containerId", ToolSchemas.integerProperty("Observed menu container ID.", 0, 255));
        properties.add("stateId", ToolSchemas.integerProperty("Observed menu state revision.", 0, Integer.MAX_VALUE));
        properties.add("slot", ToolSchemas.integerProperty("Observed menu slot index.", 0, 1023));
        properties.add("button", ToolSchemas.integerProperty("Mouse button: 0 left, 1 right.", 0, 1));
        JsonObject input = ToolSchemas.stringProperty("PICKUP or QUICK_MOVE.");
        JsonArray values = new JsonArray();
        values.add("PICKUP");
        values.add("QUICK_MOVE");
        input.add("enum", values);
        properties.add("input", input);
        return ToolSchemas.objectArguments(
                name(),
                "Click container slot",
                "Apply a validated slot click only if the observed container and revision still match.",
                properties,
                "actionId",
                "containerId",
                "stateId",
                "slot",
                "button",
                "input");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("click_slot received an unsupported argument");
        }
        try {
            String input = ControlToolSupport.string(arguments, "input");
            if (!input.equals("PICKUP") && !input.equals("QUICK_MOVE")) {
                throw new IllegalArgumentException("input must be PICKUP or QUICK_MOVE");
            }
            ClientAction action = new ClientAction.ClickSlot(
                    ControlToolSupport.actionId(arguments),
                    ControlToolSupport.boundedInt(arguments, "containerId", 0, 255),
                    ControlToolSupport.boundedInt(arguments, "stateId", 0, Integer.MAX_VALUE),
                    ControlToolSupport.boundedInt(arguments, "slot", 0, 1023),
                    ControlToolSupport.boundedInt(arguments, "button", 0, 1),
                    input);
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
