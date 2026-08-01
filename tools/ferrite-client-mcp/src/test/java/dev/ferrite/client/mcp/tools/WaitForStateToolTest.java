package dev.ferrite.client.mcp.tools;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import org.junit.jupiter.api.Test;

final class WaitForStateToolTest {
    @Test
    void satisfiesAfterARealObservationPublication() throws Exception {
        ClientObservationStore observations = new ClientObservationStore();
        WaitForStateTool tool = new WaitForStateTool(observations);
        JsonObject arguments = new JsonObject();
        arguments.addProperty("connectionState", "PLAY");
        arguments.addProperty("playerAvailable", true);
        arguments.addProperty("maxTicks", 10);

        CompletableFuture<McpToolResult> waiting = CompletableFuture.supplyAsync(
                () -> tool.call(arguments, new ToolContext("2025-11-25")));
        observations.publish(snapshot(1, "CONNECTED", null));
        observations.publish(snapshot(2, "PLAY", player(true)));

        McpToolResult result = waiting.get(2, TimeUnit.SECONDS);
        assertFalse(result.error());
        assertEquals("Satisfied", result.structuredContent().get("state").getAsString());
        assertEquals(2, result.structuredContent().get("clientTick").getAsLong());
    }

    @Test
    void timesOutByPublishedClientTicks() throws Exception {
        ClientObservationStore observations = new ClientObservationStore();
        WaitForStateTool tool = new WaitForStateTool(observations);
        JsonObject arguments = new JsonObject();
        arguments.addProperty("screenType", "InventoryScreen");
        arguments.addProperty("maxTicks", 2);

        CompletableFuture<McpToolResult> waiting = CompletableFuture.supplyAsync(
                () -> tool.call(arguments, new ToolContext("2025-11-25")));
        observations.publish(snapshot(1, "PLAY", player(true)));
        observations.publish(snapshot(2, "PLAY", player(true)));
        observations.publish(snapshot(3, "PLAY", player(true)));

        McpToolResult result = waiting.get(2, TimeUnit.SECONDS);
        assertTrue(result.error());
        assertEquals("TimedOut", result.structuredContent().get("state").getAsString());
    }

    @Test
    void rejectsAnEmptyCondition() {
        McpToolResult result = new WaitForStateTool(new ClientObservationStore())
                .call(new JsonObject(), new ToolContext("2025-11-25"));
        assertTrue(result.error());
    }

    @Test
    void usesAfterClientTickAsADeterministicFence() throws Exception {
        ClientObservationStore observations = new ClientObservationStore();
        observations.publish(snapshot(4, "PLAY", player(true)));
        WaitForStateTool tool = new WaitForStateTool(observations);
        JsonObject arguments = new JsonObject();
        arguments.addProperty("afterClientTick", 5);
        arguments.addProperty("maxTicks", 5);

        CompletableFuture<McpToolResult> waiting = CompletableFuture.supplyAsync(
                () -> tool.call(arguments, new ToolContext("2025-11-25")));
        observations.publish(snapshot(5, "PLAY", player(true)));
        observations.publish(snapshot(6, "PLAY", player(true)));

        McpToolResult result = waiting.get(2, TimeUnit.SECONDS);
        assertFalse(result.error());
        assertEquals(6, result.structuredContent().get("clientTick").getAsLong());
    }

    private static ClientSnapshot snapshot(
            long tick, String connectionState, ClientSnapshot.Player player) {
        return new ClientSnapshot(
                tick,
                new ClientSnapshot.Connection(connectionState, null, false, null, null),
                player,
                new ClientSnapshot.Inventory(false, -1, List.of()),
                new ClientSnapshot.Crosshair("NONE", null, null, null),
                new ClientSnapshot.Screen("NONE", null, false, 0, 0, null, null),
                new ClientSnapshot.NearbyBlocks(false, 2, null, false, List.of()));
    }

    private static ClientSnapshot.Player player(boolean onGround) {
        return new ClientSnapshot.Player(
                true,
                0,
                64,
                0,
                0,
                0,
                0,
                0,
                0,
                onGround,
                true,
                false,
                false,
                20,
                20,
                20,
                5,
                "survival",
                "minecraft:overworld",
                false,
                false);
    }
}
