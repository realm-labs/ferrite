package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import dev.ferrite.client.mcp.observation.ObservationJson;

/** Publishes the copied player inventory without retaining mutable ItemStacks. */
public final class InventoryStateTool implements McpTool {
    private final ClientObservationStore observations;

    public InventoryStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "inventory_state";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(), "Inventory state", "Read selected hotbar slot and non-empty inventory slots.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("inventory_state does not accept arguments");
        }
        ClientSnapshot snapshot = observations.latest();
        InventoryObservation result =
                new InventoryObservation(snapshot.clientTick(), snapshot.inventory());
        return new McpToolResult(
                ObservationJson.object(result),
                snapshot.inventory().available() ? "Inventory state observed" : "No inventory is loaded",
                false);
    }

    private record InventoryObservation(long clientTick, ClientSnapshot.Inventory inventory) {}
}
