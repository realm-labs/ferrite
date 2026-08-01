package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;

/** Minimal lifecycle observation available before game-state observation is implemented. */
public final class ClientStatusTool implements McpTool {
    private final ClientObservationStore observations;

    public ClientStatusTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "client_status";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(),
                "Ferrite client instrumentation status",
                "Report transport readiness and the latest immutable client connection state.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("client_status does not accept arguments");
        }

        ClientSnapshot snapshot = observations.latest();
        JsonObject status = new JsonObject();
        status.addProperty("state", "Ready");
        status.addProperty("protocolVersion", context.protocolVersion());
        status.addProperty("clientTick", snapshot.clientTick());
        status.addProperty("connectionState", snapshot.connection().state());
        status.addProperty("gameObservationAvailable", snapshot.player() != null);
        return new McpToolResult(status, "Ferrite client MCP transport is ready", false);
    }
}
