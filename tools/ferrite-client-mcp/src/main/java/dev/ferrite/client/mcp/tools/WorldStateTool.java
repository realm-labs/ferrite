package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;

/** Publishes the last dimension clock and weather state copied on the Minecraft thread. */
public final class WorldStateTool implements McpTool {
    private final ClientObservationStore observations;

    public WorldStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "world_state";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(), "World state", "Read dimension clock and weather state.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("world_state does not accept arguments");
        }
        ClientSnapshot snapshot = observations.latest();
        WorldObservation result =
                new WorldObservation(snapshot.clientTick(), snapshot.world());
        return new McpToolResult(
                ObservationJson.object(result),
                snapshot.world().available() ? "World state observed" : "No world is loaded",
                false);
    }

    private record WorldObservation(long clientTick, ClientSnapshot.World world) {}
}
