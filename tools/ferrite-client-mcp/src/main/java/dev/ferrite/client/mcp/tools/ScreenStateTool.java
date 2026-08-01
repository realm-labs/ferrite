package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;

/** Publishes the current screen, overlay, and copied menu revision. */
public final class ScreenStateTool implements McpTool {
    private final ClientObservationStore observations;

    public ScreenStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "screen_state";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(), "Screen state", "Read the current screen and container menu revision.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("screen_state does not accept arguments");
        }
        ClientSnapshot snapshot = observations.latest();
        ScreenObservation result = new ScreenObservation(snapshot.clientTick(), snapshot.screen());
        return new McpToolResult(ObservationJson.object(result), "Screen state observed", false);
    }

    private record ScreenObservation(long clientTick, ClientSnapshot.Screen screen) {}
}
