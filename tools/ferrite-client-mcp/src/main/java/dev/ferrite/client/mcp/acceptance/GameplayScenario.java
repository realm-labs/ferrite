package dev.ferrite.client.mcp.acceptance;

import com.google.gson.JsonObject;
import java.io.IOException;

/** Deterministic reference gameplay and Ferrite visual connection scenarios. */
final class GameplayScenario {
    private GameplayScenario() {}

    static void runReference(AcceptanceConfig config, EvidenceBundle evidence) throws Exception {
        try (ManagedServer server = ManagedServer.startReference(config, evidence);
                ManagedLauncher launcher = ManagedLauncher.start(config, evidence, server.endpoint());
                McpClient mcp = launcher.connectMcp()) {
            JsonObject play = new JsonObject();
            play.addProperty("connectionState", "PLAY");
            play.addProperty("screenType", "NONE");
            play.addProperty("playerAvailable", true);
            play.addProperty("maxTicks", 300);
            mcp.call("wait-reference-play", "wait_for_state", play);

            JsonObject before = mcp.call("player-before", "player_state", new JsonObject());
            requireAvailable(before);
            mcp.call("inventory-before", "inventory_state", new JsonObject());
            mcp.call("crosshair-before", "crosshair_state", new JsonObject());

            JsonObject look = new JsonObject();
            look.addProperty("yaw", 0.0);
            look.addProperty("pitch", 0.0);
            look.addProperty("relative", false);
            mcp.submitAndAwait("look-south", "look", look, 1);

            JsonObject movement = new JsonObject();
            movement.addProperty("forward", true);
            movement.addProperty("ticks", 10);
            mcp.submitAndAwait("move-forward", "hold_movement", movement, 10);
            JsonObject after = mcp.call("player-after-move", "player_state", new JsonObject());
            requireMoved(before, after);

            JsonObject jump = new JsonObject();
            jump.addProperty("ticks", 2);
            mcp.submitAndAwait("jump", "jump", jump, 2);

            JsonObject hotbar = new JsonObject();
            hotbar.addProperty("slot", 1);
            mcp.submitAndAwait("select-hotbar", "select_hotbar", hotbar, 1);

            JsonObject attack = new JsonObject();
            attack.addProperty("ticks", 1);
            mcp.submitAndAwait("attack", "attack", attack, 1);
            JsonObject use = new JsonObject();
            use.addProperty("ticks", 1);
            mcp.submitAndAwait("use-item", "use_item", use, 1);

            mcp.submitAndAwait("open-inventory", "open_inventory", new JsonObject(), 1);
            JsonObject inventoryScreen = new JsonObject();
            inventoryScreen.addProperty("screenType", "InventoryScreen");
            inventoryScreen.addProperty("maxTicks", 40);
            mcp.call("wait-inventory-screen", "wait_for_state", inventoryScreen);
            mcp.call("inventory-screen-state", "screen_state", new JsonObject());
            mcp.submitAndAwait("close-inventory", "close_screen", new JsonObject(), 1);
            JsonObject gameplay = new JsonObject();
            gameplay.addProperty("screenType", "NONE");
            gameplay.addProperty("connectionState", "PLAY");
            gameplay.addProperty("maxTicks", 40);
            mcp.call("wait-gameplay-screen", "wait_for_state", gameplay);

            mcp.call("nearby-blocks", "nearby_blocks", radius(2));
            mcp.screenshot("reference-screenshot", "reference-world.png");
            mcp.call("client-errors", "client_errors", new JsonObject());
        }
    }

    static void runFerrite(AcceptanceConfig config, EvidenceBundle evidence) throws Exception {
        try (ManagedServer server = ManagedServer.startFerrite(config, evidence);
                ManagedLauncher launcher = ManagedLauncher.start(config, evidence, server.endpoint());
                McpClient mcp = launcher.connectMcp()) {
            server.captureStatus(evidence, "ferrite-status-before-client.json");
            JsonObject play = new JsonObject();
            play.addProperty("connectionState", "PLAY");
            play.addProperty("playerAvailable", true);
            play.addProperty("maxTicks", 300);
            try {
                mcp.call("wait-ferrite-play", "wait_for_state", play);
                JsonObject terrain = new JsonObject();
                terrain.addProperty("connectionState", "PLAY");
                terrain.addProperty("screenType", "NONE");
                terrain.addProperty("playerAvailable", true);
                terrain.addProperty("maxTicks", 200);
                mcp.call("wait-ferrite-terrain", "wait_for_state", terrain);
            } catch (IOException error) {
                server.captureStatus(evidence, "ferrite-status-after-disconnect.json");
                mcp.call("ferrite-client-errors", "client_errors", new JsonObject());
                throw error;
            }
            JsonObject player = mcp.call("ferrite-player", "player_state", new JsonObject());
            mcp.call("ferrite-nearby-blocks", "nearby_blocks", radius(2));
            JsonObject sustained = new JsonObject();
            sustained.addProperty("afterClientTick", player.get("clientTick").getAsLong() + 40);
            sustained.addProperty("connectionState", "PLAY");
            sustained.addProperty("screenType", "NONE");
            sustained.addProperty("playerAvailable", true);
            sustained.addProperty("maxTicks", 100);
            mcp.call("wait-ferrite-sustained-play", "wait_for_state", sustained);
            JsonObject status = server.captureStatus(evidence, "ferrite-status-in-play.json");
            if (status.get("active_sessions").getAsInt() != 1) {
                throw new IOException("Ferrite did not retain exactly one active client session");
            }
            mcp.screenshot("ferrite-screenshot", "ferrite-world.png");
        }
    }

    private static JsonObject radius(int radius) {
        JsonObject arguments = new JsonObject();
        arguments.addProperty("radius", radius);
        return arguments;
    }

    private static void requireAvailable(JsonObject state) throws IOException {
        if (!state.has("available") || !state.get("available").getAsBoolean()) {
            throw new IOException("reference player is unavailable");
        }
    }

    private static void requireMoved(JsonObject before, JsonObject after) throws IOException {
        JsonObject left = before.getAsJsonObject("player");
        JsonObject right = after.getAsJsonObject("player");
        double delta = Math.abs(left.get("x").getAsDouble() - right.get("x").getAsDouble())
                + Math.abs(left.get("y").getAsDouble() - right.get("y").getAsDouble())
                + Math.abs(left.get("z").getAsDouble() - right.get("z").getAsDouble());
        if (delta < 0.01) {
            throw new IOException("normal movement produced no observable position change");
        }
    }
}
