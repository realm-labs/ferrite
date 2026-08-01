package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;

/** Queues a priority release of every input currently owned by the MCP. */
final class ReleaseAllInputsTool implements McpTool {
    private final ClientControl control;

    ReleaseAllInputsTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "release_all_inputs";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        return ToolSchemas.objectArguments(
                name(),
                "Release MCP inputs",
                "Priority-release every gameplay key held by this MCP session.",
                properties,
                "actionId");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.keySet().stream().allMatch("actionId"::equals)) {
            return ToolSchemas.rejected("release_all_inputs accepts only actionId");
        }
        try {
            return ControlToolSupport.receipt(
                    control.submit(new ClientAction.ReleaseAll(ControlToolSupport.actionId(arguments))));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
