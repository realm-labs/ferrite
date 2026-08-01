package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;

/** Publishes the last player state copied on the Minecraft thread. */
public final class PlayerStateTool implements McpTool {
    private final ClientObservationStore observations;

    public PlayerStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "player_state";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(), "Player state", "Read position, motion, health, mode, and dimension.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("player_state does not accept arguments");
        }
        ClientSnapshot snapshot = observations.latest();
        PlayerObservation result =
                new PlayerObservation(snapshot.clientTick(), snapshot.player() != null, snapshot.player());
        return new McpToolResult(
                ObservationJson.object(result),
                snapshot.player() == null ? "No player is loaded" : "Player state observed",
                false);
    }

    private record PlayerObservation(
            long clientTick, boolean available, ClientSnapshot.Player player) {}
}
