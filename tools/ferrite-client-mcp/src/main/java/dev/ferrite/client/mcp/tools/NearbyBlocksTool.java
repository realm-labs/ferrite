package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;
import java.util.List;

/** Reads a radius-filtered view of the bounded block snapshot. */
public final class NearbyBlocksTool implements McpTool {
    private static final int MAXIMUM_RADIUS = 2;

    private final ClientObservationStore observations;

    public NearbyBlocksTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "nearby_blocks";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.boundedIntegerArgument(
                name(),
                "Nearby blocks",
                "Read non-air blocks copied around the player at the latest client tick.",
                "radius",
                "Inclusive block radius from zero through two; defaults to two.",
                0,
                MAXIMUM_RADIUS);
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ToolSchemas.hasOnly(arguments, "radius")) {
            return ToolSchemas.rejected("nearby_blocks accepts only radius");
        }
        Integer radius = readRadius(arguments);
        if (radius == null) {
            return ToolSchemas.rejected("radius must be an integer between 0 and 2");
        }

        ClientSnapshot snapshot = observations.latest();
        ClientSnapshot.NearbyBlocks nearby = snapshot.nearbyBlocks();
        List<ClientSnapshot.Block> filtered = nearby.center() == null
                ? List.of()
                : nearby.blocks().stream()
                        .filter(block -> within(block, nearby.center(), radius))
                        .toList();
        ClientSnapshot.NearbyBlocks result = new ClientSnapshot.NearbyBlocks(
                nearby.available(), radius, nearby.center(), nearby.complete(), filtered);
        NearbyObservation observation = new NearbyObservation(snapshot.clientTick(), result);
        return new McpToolResult(
                ObservationJson.object(observation),
                nearby.available() ? "Nearby blocks observed" : "No world is loaded",
                false);
    }

    private static Integer readRadius(JsonObject arguments) {
        if (!arguments.has("radius")) {
            return MAXIMUM_RADIUS;
        }
        JsonElement radius = arguments.get("radius");
        if (!radius.isJsonPrimitive() || !radius.getAsJsonPrimitive().isNumber()) {
            return null;
        }
        double value = radius.getAsDouble();
        int integer = radius.getAsInt();
        return Double.isFinite(value)
                        && value == integer
                        && integer >= 0
                        && integer <= MAXIMUM_RADIUS
                ? integer
                : null;
    }

    private static boolean within(
            ClientSnapshot.Block block, ClientSnapshot.BlockPosition center, int radius) {
        return Math.abs(block.x() - center.x()) <= radius
                && Math.abs(block.y() - center.y()) <= radius
                && Math.abs(block.z() - center.z()) <= radius;
    }

    private record NearbyObservation(long clientTick, ClientSnapshot.NearbyBlocks nearbyBlocks) {}
}
