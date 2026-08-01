package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;

/** Closes the current screen through its normal client callback. */
final class CloseScreenTool implements McpTool {
    private final ClientControl control;

    CloseScreenTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "close_screen";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        return ToolSchemas.objectArguments(
                name(), "Close screen", "Close the current Minecraft screen normally.", properties, "actionId");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.keySet().stream().allMatch("actionId"::equals)) {
            return ToolSchemas.rejected("close_screen accepts only actionId");
        }
        try {
            return ControlToolSupport.receipt(control.submit(
                    new ClientAction.CloseScreen(ControlToolSupport.actionId(arguments))));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
