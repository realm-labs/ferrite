package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;

/** Minimal lifecycle observation available before game-state observation is implemented. */
public final class ClientStatusTool implements McpTool {
    @Override
    public String name() {
        return "client_status";
    }

    @Override
    public JsonObject definition() {
        JsonObject schema = new JsonObject();
        schema.addProperty("type", "object");
        schema.add("properties", new JsonObject());
        schema.addProperty("additionalProperties", false);

        JsonObject definition = new JsonObject();
        definition.addProperty("name", name());
        definition.addProperty("title", "Ferrite client instrumentation status");
        definition.addProperty(
                "description",
                "Report whether the instrumented client MCP transport is running. Game connection state is added by the observation batch.");
        definition.add("inputSchema", schema);
        return definition;
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            JsonObject error = new JsonObject();
            error.addProperty("state", "Rejected");
            error.addProperty("reason", "client_status does not accept arguments");
            return new McpToolResult(error, "client_status does not accept arguments", true);
        }

        JsonObject status = new JsonObject();
        status.addProperty("state", "Ready");
        status.addProperty("protocolVersion", context.protocolVersion());
        status.addProperty("gameObservationAvailable", false);
        return new McpToolResult(status, "Ferrite client MCP transport is ready", false);
    }
}
