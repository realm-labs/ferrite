package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientControl;

/** Retrieves the latest immutable receipt for an asynchronous control action. */
final class ActionStatusTool implements McpTool {
    private final ClientControl control;

    ActionStatusTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "action_status";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Previously submitted action ID."));
        return ToolSchemas.objectArguments(
                name(),
                "Action status",
                "Read the latest client-thread action receipt.",
                properties,
                "actionId");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.keySet().stream().allMatch("actionId"::equals)) {
            return ToolSchemas.rejected("action_status accepts only actionId");
        }
        try {
            return ControlToolSupport.receipt(
                    control.status(ControlToolSupport.actionId(arguments)));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
