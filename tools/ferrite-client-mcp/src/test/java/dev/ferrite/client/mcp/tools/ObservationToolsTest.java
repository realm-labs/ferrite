package dev.ferrite.client.mcp.tools;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import java.util.List;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

final class ObservationToolsTest {
    private ClientObservationStore observations;
    private ToolRegistry tools;

    @BeforeEach
    void publishSnapshot() {
        observations = new ClientObservationStore();
        observations.publish(snapshot());
        observations.recordError(8, "connection", "Connection reset");
        tools = ToolRegistry.forObservations(observations);
    }

    @Test
    void allObservationToolsExposeCopiedState() {
        JsonObject status = call("client_status", new JsonObject()).structuredContent();
        assertTrue(status.get("gameObservationAvailable").getAsBoolean());
        assertEquals("PLAY", status.get("connectionState").getAsString());

        JsonObject player = call("player_state", new JsonObject()).structuredContent();
        assertTrue(player.get("available").getAsBoolean());
        assertEquals(
                "minecraft:overworld",
                player.getAsJsonObject("player").get("dimension").getAsString());

        JsonObject world = call("world_state", new JsonObject()).structuredContent();
        assertEquals(
                6000,
                world.getAsJsonObject("world").get("defaultClockTime").getAsLong());

        JsonObject inventory = call("inventory_state", new JsonObject()).structuredContent();
        assertEquals(
                "minecraft:stone",
                inventory
                        .getAsJsonObject("inventory")
                        .getAsJsonArray("items")
                        .get(0)
                        .getAsJsonObject()
                        .get("itemId")
                        .getAsString());

        assertEquals(
                "BLOCK",
                call("crosshair_state", new JsonObject())
                        .structuredContent()
                        .getAsJsonObject("crosshair")
                        .get("kind")
                        .getAsString());
        assertEquals(
                7,
                call("screen_state", new JsonObject())
                        .structuredContent()
                        .getAsJsonObject("screen")
                        .getAsJsonObject("menu")
                        .get("stateId")
                        .getAsInt());
        assertEquals(
                1,
                call("client_errors", new JsonObject())
                        .structuredContent()
                        .get("count")
                        .getAsInt());
    }

    @Test
    void nearbyRadiusFiltersAndBoundsFailAsToolErrors() {
        JsonObject radius = new JsonObject();
        radius.addProperty("radius", 1);
        McpToolResult filtered = call("nearby_blocks", radius);
        assertFalse(filtered.error());
        assertEquals(
                1,
                filtered
                        .structuredContent()
                        .getAsJsonObject("nearbyBlocks")
                        .getAsJsonArray("blocks")
                        .size());

        JsonObject invalid = new JsonObject();
        invalid.addProperty("radius", 3);
        assertTrue(call("nearby_blocks", invalid).error());

        JsonObject unknown = new JsonObject();
        unknown.addProperty("unexpected", true);
        assertTrue(call("client_errors", unknown).error());
    }

    private McpToolResult call(String name, JsonObject arguments) {
        return tools.find(name)
                .orElseThrow()
                .call(arguments, new ToolContext("2025-11-25"));
    }

    private static ClientSnapshot snapshot() {
        ClientSnapshot.Item inventoryItem =
                new ClientSnapshot.Item(0, "minecraft:stone", 16, 0, 0);
        ClientSnapshot.Player player = new ClientSnapshot.Player(
                true,
                0.5,
                65.0,
                0.5,
                90.0f,
                0.0f,
                0.1,
                0.0,
                0.0,
                true,
                true,
                false,
                false,
                20.0f,
                20.0f,
                20,
                5.0f,
                "survival",
                "minecraft:overworld",
                false,
                false);
        ClientSnapshot.BlockPosition center = new ClientSnapshot.BlockPosition(0, 65, 0);
        return new ClientSnapshot(
                9,
                new ClientSnapshot.Connection("PLAY", "127.0.0.1:25565", false, 1L, 1),
                player,
                new ClientSnapshot.World(
                        true, "minecraft:overworld", 6000, 6000, 0.25f, 0.0f, true, false),
                new ClientSnapshot.Inventory(true, 0, List.of(inventoryItem)),
                new ClientSnapshot.Crosshair(
                        "BLOCK",
                        new ClientSnapshot.Point(0.5, 64.0, 0.5),
                        new ClientSnapshot.BlockTarget(0, 64, 0, "up", "minecraft:stone"),
                        null),
                new ClientSnapshot.Screen(
                        "InventoryScreen",
                        "Inventory",
                        false,
                        320,
                        240,
                        null,
                        new ClientSnapshot.Menu(1, 7, 46, null, List.of(inventoryItem))),
                new ClientSnapshot.NearbyBlocks(
                        true,
                        2,
                        center,
                        true,
                        List.of(
                                new ClientSnapshot.Block(
                                        0, 64, 0, "minecraft:stone", "minecraft:stone"),
                                new ClientSnapshot.Block(
                                        2, 65, 0, "minecraft:dirt", "minecraft:dirt"))));
    }
}
