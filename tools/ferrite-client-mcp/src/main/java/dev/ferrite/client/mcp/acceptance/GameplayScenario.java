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
            JsonObject player = mcp.call("ferrite-player-before", "player_state", new JsonObject());
            requireAvailable(player);
            mcp.call("ferrite-nearby-blocks-before", "nearby_blocks", radius(2));
            JsonObject initialStatus = server.awaitStatus(
                    evidence,
                    "ferrite-status-composite-ready.json",
                    GameplayScenario::compositeSessionReady,
                    "Ferrite did not publish one committed composite session");
            JsonObject initialSession = onlySession(initialStatus);
            int initialRegionX = initialSession.get("region_x").getAsInt();
            int initialRegionZ = initialSession.get("region_z").getAsInt();

            JsonObject lookSouth = new JsonObject();
            lookSouth.addProperty("yaw", 0.0);
            lookSouth.addProperty("pitch", 0.0);
            lookSouth.addProperty("relative", false);
            mcp.submitAndAwait("ferrite-look-south", "look", lookSouth, 1);
            JsonObject moved = player;
            for (int segment = 1; segment <= 4; segment++) {
                JsonObject movement = new JsonObject();
                movement.addProperty("forward", true);
                movement.addProperty("ticks", 200);
                mcp.submitAndAwait(
                        "ferrite-move-segment-" + segment, "hold_movement", movement, 200);
                moved = mcp.call(
                        "ferrite-player-after-segment-" + segment,
                        "player_state",
                        new JsonObject());
                JsonObject status = server.captureStatus(
                        evidence, "ferrite-status-after-segment-" + segment + ".json");
                JsonObject session = onlySession(status);
                if (session.get("region_x").getAsInt() != initialRegionX
                        || session.get("region_z").getAsInt() != initialRegionZ) {
                    break;
                }
            }
            requireMoved(player, moved);
            JsonObject transferStatus = server.awaitStatus(
                    evidence,
                    "ferrite-status-after-region-transfer.json",
                    status -> transferred(status, initialRegionX, initialRegionZ),
                    "normal movement did not commit a Region transfer");
            if (onlySession(transferStatus).get("region_transfers").getAsLong() < 1) {
                throw new IOException("Ferrite did not count the committed Region transfer");
            }

            JsonObject sustained = new JsonObject();
            sustained.addProperty("afterClientTick", moved.get("clientTick").getAsLong() + 40);
            sustained.addProperty("connectionState", "PLAY");
            sustained.addProperty("screenType", "NONE");
            sustained.addProperty("playerAvailable", true);
            sustained.addProperty("maxTicks", 100);
            mcp.call("wait-ferrite-sustained-after-transfer", "wait_for_state", sustained);

            JsonObject lookDown = new JsonObject();
            lookDown.addProperty("yaw", 0.0);
            lookDown.addProperty("pitch", 80.0);
            lookDown.addProperty("relative", false);
            mcp.submitAndAwait("ferrite-look-at-ground", "look", lookDown, 1);
            JsonObject crosshair = mcp.call(
                    "ferrite-block-target-before", "crosshair_state", new JsonObject());
            JsonObject target = requireStoneTarget(crosshair);
            JsonObject attack = new JsonObject();
            attack.addProperty("ticks", 2);
            mcp.submitAndAwait("ferrite-block-interaction", "attack", attack, 2);
            server.awaitStatus(
                    evidence,
                    "ferrite-status-after-block-interaction.json",
                    GameplayScenario::hasCommittedBlockResult,
                    "Ferrite did not publish a committed block result");
            JsonObject converged = mcp.call(
                    "ferrite-nearby-blocks-after-interaction", "nearby_blocks", radius(2));
            requireTargetBlock(converged, target, "minecraft:stone");

            JsonObject chat = new JsonObject();
            chat.addProperty("message", "ferrite goal three unsupported probe");
            mcp.submitAndAwait("ferrite-unsupported-chat", "send_chat", chat, 1);
            JsonObject unsupportedStatus = server.awaitStatus(
                    evidence,
                    "ferrite-status-after-unsupported.json",
                    GameplayScenario::hasExplicitUnsupported,
                    "Ferrite did not expose the unsupported chat disposition");
            requireExplicitUnsupported(unsupportedStatus);

            JsonObject finalFence = new JsonObject();
            finalFence.addProperty("afterClientTick", converged.get("clientTick").getAsLong() + 20);
            finalFence.addProperty("connectionState", "PLAY");
            finalFence.addProperty("screenType", "NONE");
            finalFence.addProperty("playerAvailable", true);
            finalFence.addProperty("maxTicks", 80);
            mcp.call("wait-ferrite-visual-convergence", "wait_for_state", finalFence);
            JsonObject visualLook = new JsonObject();
            visualLook.addProperty("yaw", 0.0);
            visualLook.addProperty("pitch", 20.0);
            visualLook.addProperty("relative", false);
            JsonObject visualReceipt = mcp.submitAndAwait(
                    "ferrite-visual-look", "look", visualLook, 1);
            JsonObject renderFence = new JsonObject();
            renderFence.addProperty(
                    "afterClientTick", visualReceipt.get("completedTick").getAsLong() + 40);
            renderFence.addProperty("connectionState", "PLAY");
            renderFence.addProperty("screenType", "NONE");
            renderFence.addProperty("playerAvailable", true);
            renderFence.addProperty("maxTicks", 100);
            mcp.call("wait-ferrite-render-convergence", "wait_for_state", renderFence);
            mcp.call("ferrite-player-final", "player_state", new JsonObject());
            mcp.call("ferrite-nearby-blocks-final", "nearby_blocks", radius(2));
            JsonObject status = server.captureStatus(evidence, "ferrite-status-in-play.json");
            if (status.get("active_sessions").getAsInt() != 1) {
                throw new IOException("Ferrite did not retain exactly one active client session");
            }
            mcp.call("ferrite-client-errors", "client_errors", new JsonObject());
            mcp.screenshot("ferrite-composite-screenshot", "ferrite-composite-world.png");
        }
    }

    private static boolean compositeSessionReady(JsonObject status) {
        if (!status.has("minecraft") || status.get("minecraft").isJsonNull()) {
            return false;
        }
        JsonObject minecraft = status.getAsJsonObject("minecraft");
        return minecraft.get("committed_tick").getAsLong() > 0
                && minecraft.get("composite_region_commits").getAsInt() == 25
                && minecraft.getAsJsonArray("sessions").size() == 1
                && !onlySession(status).get("region_x").isJsonNull();
    }

    private static boolean transferred(JsonObject status, int initialX, int initialZ) {
        if (!compositeSessionReady(status)) {
            return false;
        }
        JsonObject session = onlySession(status);
        return session.get("region_transfers").getAsLong() > 0
                && (session.get("region_x").getAsInt() != initialX
                        || session.get("region_z").getAsInt() != initialZ);
    }

    private static boolean hasCommittedBlockResult(JsonObject status) {
        return compositeSessionReady(status)
                && !onlySession(status).get("last_block_result").isJsonNull();
    }

    private static boolean hasExplicitUnsupported(JsonObject status) {
        if (!compositeSessionReady(status)) {
            return false;
        }
        var element = onlySession(status).get("last_unsupported_dispatch");
        if (element == null || element.isJsonNull()) {
            return false;
        }
        JsonObject unsupported = element.getAsJsonObject();
        return "unsupported".equals(unsupported.get("disposition").getAsString())
                && "chat-and-command".equals(unsupported.get("responsibility").getAsString());
    }

    private static void requireExplicitUnsupported(JsonObject status) throws IOException {
        if (!hasExplicitUnsupported(status)) {
            throw new IOException("unsupported packet was not classified explicitly");
        }
        String packet = onlySession(status)
                .getAsJsonObject("last_unsupported_dispatch")
                .get("packet")
                .getAsString();
        if (!packet.equals("ChatMessage") && !packet.equals("ChatSessionUpdate")) {
            throw new IOException("unexpected unsupported chat packet: " + packet);
        }
    }

    private static JsonObject onlySession(JsonObject status) {
        return status.getAsJsonObject("minecraft")
                .getAsJsonArray("sessions")
                .get(0)
                .getAsJsonObject();
    }

    private static JsonObject requireStoneTarget(JsonObject observation) throws IOException {
        JsonObject crosshair = observation.getAsJsonObject("crosshair");
        JsonObject block = crosshair == null ? null : crosshair.getAsJsonObject("block");
        if (block == null || !"BLOCK".equals(crosshair.get("kind").getAsString())) {
            throw new IOException("client crosshair did not acquire a block target");
        }
        if (!"minecraft:stone".equals(block.get("blockId").getAsString())) {
            throw new IOException("client crosshair did not target the formal stone terrain");
        }
        return block;
    }

    private static void requireTargetBlock(
            JsonObject observation, JsonObject target, String expectedBlock) throws IOException {
        for (var element : observation
                .getAsJsonObject("nearbyBlocks")
                .getAsJsonArray("blocks")) {
            JsonObject block = element.getAsJsonObject();
            if (block.get("x").getAsInt() == target.get("x").getAsInt()
                    && block.get("y").getAsInt() == target.get("y").getAsInt()
                    && block.get("z").getAsInt() == target.get("z").getAsInt()) {
                if (!expectedBlock.equals(block.get("blockId").getAsString())) {
                    throw new IOException("client block did not converge to the authoritative state");
                }
                return;
            }
        }
        throw new IOException("target block was absent from the converged nearby-block snapshot");
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
