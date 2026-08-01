package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;

/** Publishes the last normal client crosshair pick result. */
public final class CrosshairStateTool implements McpTool {
    private final ClientObservationStore observations;

    public CrosshairStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "crosshair_state";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(), "Crosshair state", "Read the client's current miss, block, or entity hit.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("crosshair_state does not accept arguments");
        }
        ClientSnapshot snapshot = observations.latest();
        CrosshairObservation result =
                new CrosshairObservation(snapshot.clientTick(), snapshot.crosshair());
        return new McpToolResult(
                ObservationJson.object(result), "Crosshair state observed", false);
    }

    private record CrosshairObservation(long clientTick, ClientSnapshot.Crosshair crosshair) {}
}
