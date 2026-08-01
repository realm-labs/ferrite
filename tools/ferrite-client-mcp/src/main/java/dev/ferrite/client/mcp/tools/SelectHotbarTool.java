package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import java.util.Set;

/** Selects a normal zero-based player hotbar slot on the client thread. */
final class SelectHotbarTool implements McpTool {
    private static final Set<String> ALLOWED = Set.of("actionId", "slot");

    private final ClientControl control;

    SelectHotbarTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "select_hotbar";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        properties.add("slot", ToolSchemas.integerProperty("Zero-based hotbar slot.", 0, 8));
        return ToolSchemas.objectArguments(
                name(),
                "Select hotbar",
                "Select the local player's normal hotbar slot on the client thread.",
                properties,
                "actionId",
                "slot");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("select_hotbar received an unsupported argument");
        }
        try {
            ClientAction action = new ClientAction.SelectHotbar(
                    ControlToolSupport.actionId(arguments),
                    ControlToolSupport.boundedInt(arguments, "slot", 0, 8));
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
