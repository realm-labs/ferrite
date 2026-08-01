package dev.ferrite.client.mcp.acceptance;

import com.google.gson.JsonObject;
import java.io.IOException;
import java.util.TreeSet;

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
        JsonObject worldBeforeRestart;
        JsonObject terrainBeforeRestart;
        try (ManagedServer server = ManagedServer.startFerrite(config, evidence);
                ManagedLauncher launcher = ManagedLauncher.start(config, evidence, server.endpoint());
                McpClient mcp = launcher.connectMcp()) {
            server.captureStatus(evidence, "ferrite-status-before-client.json");
            try {
                waitForPlayableTerrain(mcp, "ferrite-initial");
            } catch (IOException error) {
                server.captureStatus(evidence, "ferrite-status-after-disconnect.json");
                mcp.call("ferrite-client-errors", "client_errors", new JsonObject());
                throw error;
            }
            JsonObject initialPlayer =
                    mcp.call("ferrite-player-before", "player_state", new JsonObject());
            requireAvailable(initialPlayer);
            terrainBeforeRestart =
                    mcp.call("ferrite-generated-terrain-before", "nearby_blocks", radius(2));
            requireGeneratedTerrain(terrainBeforeRestart);
            worldBeforeRestart =
                    mcp.call("ferrite-world-before", "world_state", new JsonObject());
            requireWorldObservable(worldBeforeRestart);
            server.awaitStatus(
                    evidence,
                    "ferrite-status-composite-ready.json",
                    GameplayScenario::compositeSessionReady,
                    "Ferrite did not publish one committed world session");
            server.awaitStatus(
                    evidence,
                    "ferrite-status-full-view.json",
                    GameplayScenario::fullViewStreamed,
                    "Ferrite did not finish the initial authoritative chunk view");

            JsonObject lookSouth = new JsonObject();
            lookSouth.addProperty("yaw", 0.0);
            lookSouth.addProperty("pitch", 0.0);
            lookSouth.addProperty("relative", false);
            mcp.submitAndAwait("ferrite-look-south", "look", lookSouth, 1);
            JsonObject movement = new JsonObject();
            movement.addProperty("forward", true);
            movement.addProperty("ticks", 40);
            mcp.submitAndAwait("ferrite-explore", "hold_movement", movement, 40);
            JsonObject movementStatus = server.awaitStatus(
                    evidence,
                    "ferrite-status-after-exploration.json",
                    status -> movedAndGrounded(status, initialPlayer),
                    "normal input did not produce authoritative grounded exploration");
            requireServerCollision(movementStatus);
            JsonObject moved =
                    mcp.call("ferrite-player-after-exploration", "player_state", new JsonObject());
            requireMoved(initialPlayer, moved);
            JsonObject terrainAfterMovement =
                    mcp.call("ferrite-generated-terrain-after", "nearby_blocks", radius(2));
            requireGeneratedTerrain(terrainAfterMovement);

            JsonObject jump = new JsonObject();
            jump.addProperty("ticks", 2);
            JsonObject jumpReceipt = mcp.submitAndAwait("ferrite-jump", "jump", jump, 2);
            JsonObject landed = new JsonObject();
            landed.addProperty("afterClientTick", jumpReceipt.get("completedTick").getAsLong() + 20);
            landed.addProperty("connectionState", "PLAY");
            landed.addProperty("playerAvailable", true);
            landed.addProperty("onGround", true);
            landed.addProperty("maxTicks", 200);
            waitForState(mcp, "ferrite-land-after-jump", landed, 2);

            JsonObject clockFence = new JsonObject();
            clockFence.addProperty(
                    "afterClientTick", worldBeforeRestart.get("clientTick").getAsLong() + 40);
            clockFence.addProperty("connectionState", "PLAY");
            clockFence.addProperty("playerAvailable", true);
            clockFence.addProperty("maxTicks", 120);
            mcp.call("wait-ferrite-clock", "wait_for_state", clockFence);
            JsonObject worldAfter =
                    mcp.call("ferrite-world-after", "world_state", new JsonObject());
            requireClockAdvanced(worldBeforeRestart, worldAfter);
            JsonObject status = server.captureStatus(evidence, "ferrite-status-in-play.json");
            if (status.get("active_sessions").getAsInt() != 1) {
                throw new IOException("Ferrite did not retain exactly one active client session");
            }
            mcp.call("ferrite-client-errors", "client_errors", new JsonObject());
            prepareVisual(mcp, "ferrite-world-visual");
            mcp.screenshot("ferrite-generated-world-screenshot", "ferrite-generated-world.png");
        }

        try (ManagedServer server = ManagedServer.restartFerrite(config, evidence);
                ManagedLauncher launcher = ManagedLauncher.start(config, evidence, server.endpoint());
                McpClient mcp = launcher.connectMcp()) {
            waitForPlayableTerrain(mcp, "ferrite-restart");
            JsonObject restartedWorld =
                    mcp.call("ferrite-world-after-restart", "world_state", new JsonObject());
            requireRestartClock(worldBeforeRestart, restartedWorld);
            JsonObject restartedTerrain =
                    mcp.call("ferrite-terrain-after-restart", "nearby_blocks", radius(2));
            requireGeneratedTerrain(restartedTerrain);
            requireSameTerrain(terrainBeforeRestart, restartedTerrain);
            server.awaitStatus(
                    evidence,
                    "ferrite-status-after-restart.json",
                    GameplayScenario::compositeSessionReady,
                    "restarted Ferrite world did not admit the exact client");
            server.awaitStatus(
                    evidence,
                    "ferrite-status-full-view-after-restart.json",
                    GameplayScenario::fullViewStreamed,
                    "restarted Ferrite world did not finish the authoritative chunk view");
            mcp.call("ferrite-client-errors-after-restart", "client_errors", new JsonObject());
            prepareVisual(mcp, "ferrite-restart-visual");
            mcp.screenshot(
                    "ferrite-restarted-world-screenshot", "ferrite-restarted-world.png");
        }
    }

    static void runFerritePortal(AcceptanceConfig config, EvidenceBundle evidence) throws Exception {
        try (ManagedServer server = ManagedServer.startPortalFerrite(config, evidence);
                ManagedLauncher launcher = ManagedLauncher.start(config, evidence, server.endpoint());
                McpClient mcp = launcher.connectMcp()) {
            waitForPlayableTerrain(mcp, "ferrite-portal-source");
            server.awaitStatus(
                    evidence,
                    "ferrite-portal-status-full-source-view.json",
                    GameplayScenario::fullViewStreamed,
                    "portal acceptance world did not finish the source chunk view");
            JsonObject look = new JsonObject();
            look.addProperty("yaw", 0.0);
            look.addProperty("pitch", 0.0);
            look.addProperty("relative", false);
            mcp.submitAndAwait("ferrite-portal-look", "look", look, 1);
            JsonObject movement = new JsonObject();
            movement.addProperty("forward", true);
            movement.addProperty("ticks", 22);
            mcp.submitAndAwait("ferrite-enter-portal", "hold_movement", movement, 22);
            JsonObject sourceBlocks =
                    mcp.call("ferrite-portal-source-blocks", "nearby_blocks", radius(2));
            requirePortalBlock(sourceBlocks);
            JsonObject destination = waitForDimension(
                    mcp, "ferrite-wait-nether", "minecraft:the_nether", 360);
            requirePlayerDimension(destination, "minecraft:the_nether");
            server.awaitStatus(
                    evidence,
                    "ferrite-portal-status-nether.json",
                    status -> sessionDimension(status, "minecraft:the_nether"),
                    "authoritative portal transfer did not commit the Nether dimension");
            JsonObject netherWorld =
                    mcp.call("ferrite-nether-world", "world_state", new JsonObject());
            requireWorldDimension(netherWorld, "minecraft:the_nether");
            server.awaitStatus(
                    evidence,
                    "ferrite-portal-status-full-nether-view.json",
                    GameplayScenario::fullViewStreamed,
                    "portal destination did not finish its authoritative chunk view");
            waitForPlayableTerrain(mcp, "ferrite-nether-ready");
            prepareVisual(mcp, "ferrite-nether-visual");
            mcp.call("ferrite-portal-client-errors", "client_errors", new JsonObject());
            mcp.screenshot("ferrite-nether-screenshot", "ferrite-nether-after-portal.png");
        }
    }

    private static void waitForPlayableTerrain(McpClient mcp, String operation)
            throws IOException, InterruptedException {
        JsonObject play = new JsonObject();
        play.addProperty("connectionState", "PLAY");
        play.addProperty("playerAvailable", true);
        play.addProperty("maxTicks", 300);
        mcp.call(operation + "-play", "wait_for_state", play);
        JsonObject terrain = new JsonObject();
        terrain.addProperty("connectionState", "PLAY");
        terrain.addProperty("screenType", "NONE");
        terrain.addProperty("playerAvailable", true);
        terrain.addProperty("maxTicks", 400);
        waitForState(mcp, operation + "-terrain", terrain, 3);
    }

    private static void prepareVisual(McpClient mcp, String operation)
            throws IOException, InterruptedException {
        JsonObject look = new JsonObject();
        look.addProperty("yaw", 0.0);
        look.addProperty("pitch", 20.0);
        look.addProperty("relative", false);
        JsonObject receipt = mcp.submitAndAwait(operation, "look", look, 1);
        JsonObject render = new JsonObject();
        render.addProperty("afterClientTick", receipt.get("completedTick").getAsLong() + 40);
        render.addProperty("connectionState", "PLAY");
        render.addProperty("screenType", "NONE");
        render.addProperty("playerAvailable", true);
        render.addProperty("maxTicks", 100);
        mcp.call(operation + "-fence", "wait_for_state", render);
    }

    private static void requireGeneratedTerrain(JsonObject observation) throws IOException {
        JsonObject nearby = observation.getAsJsonObject("nearbyBlocks");
        if (nearby == null
                || !nearby.get("available").getAsBoolean()
                || !nearby.get("complete").getAsBoolean()) {
            throw new IOException("generated terrain observation is incomplete");
        }
        boolean stone = false;
        boolean surface = false;
        for (var element : nearby.getAsJsonArray("blocks")) {
            String block = element.getAsJsonObject().get("blockId").getAsString();
            stone |= "minecraft:stone".equals(block);
            surface |= "minecraft:grass_block".equals(block)
                    || "minecraft:dirt".equals(block)
                    || "minecraft:sand".equals(block);
        }
        if (!stone || !surface) {
            throw new IOException("client did not observe generated subsurface and surface blocks");
        }
    }

    private static void requirePortalBlock(JsonObject observation) throws IOException {
        for (var element : observation
                .getAsJsonObject("nearbyBlocks")
                .getAsJsonArray("blocks")) {
            if ("minecraft:nether_portal"
                    .equals(element.getAsJsonObject().get("blockId").getAsString())) {
                return;
            }
        }
        throw new IOException("exact client did not observe the generated source portal block");
    }

    private static JsonObject waitForDimension(
            McpClient mcp, String operation, String dimension, int attempts)
            throws IOException, InterruptedException {
        JsonObject latest = null;
        for (int attempt = 1; attempt <= attempts; attempt++) {
            latest = mcp.call(operation + "-" + attempt, "player_state", new JsonObject());
            if (latest.has("available")
                    && latest.get("available").getAsBoolean()
                    && dimension.equals(
                            latest.getAsJsonObject("player").get("dimension").getAsString())) {
                return latest;
            }
            Thread.sleep(250);
        }
        throw new IOException("client did not enter " + dimension + ": " + latest);
    }

    private static void requirePlayerDimension(JsonObject observation, String dimension)
            throws IOException {
        if (!observation.get("available").getAsBoolean()
                || !dimension.equals(
                        observation.getAsJsonObject("player").get("dimension").getAsString())) {
            throw new IOException("client player dimension did not converge to " + dimension);
        }
    }

    private static void requireWorldDimension(JsonObject observation, String dimension)
            throws IOException {
        JsonObject world = observation.getAsJsonObject("world");
        if (world == null
                || !world.get("available").getAsBoolean()
                || !dimension.equals(world.get("dimension").getAsString())) {
            throw new IOException("client world dimension did not converge to " + dimension);
        }
    }

    private static void requireWorldObservable(JsonObject observation) throws IOException {
        JsonObject world = observation.getAsJsonObject("world");
        if (world == null
                || !world.get("available").getAsBoolean()
                || !"minecraft:overworld".equals(world.get("dimension").getAsString())) {
            throw new IOException("client did not observe the formal overworld environment");
        }
        float rain = world.get("rainLevel").getAsFloat();
        float thunder = world.get("thunderLevel").getAsFloat();
        if (!Float.isFinite(rain)
                || !Float.isFinite(thunder)
                || rain < 0.0f
                || rain > 1.0f
                || thunder < 0.0f
                || thunder > 1.0f) {
            throw new IOException("client weather projection is outside its valid range");
        }
    }

    private static boolean movedAndGrounded(JsonObject status, JsonObject initialPlayer) {
        if (!compositeSessionReady(status)) {
            return false;
        }
        JsonObject initial = initialPlayer.getAsJsonObject("player");
        JsonObject session = onlySession(status);
        double horizontal = Math.abs(session.get("x").getAsDouble() - initial.get("x").getAsDouble())
                + Math.abs(session.get("z").getAsDouble() - initial.get("z").getAsDouble());
        return horizontal > 1.0 && session.get("on_ground").getAsBoolean();
    }

    private static void requireServerCollision(JsonObject status) throws IOException {
        JsonObject session = onlySession(status);
        double y = session.get("y").getAsDouble();
        if (!session.get("on_ground").getAsBoolean() || !Double.isFinite(y) || y < -64 || y > 320) {
            throw new IOException("authoritative collision did not retain a grounded world position");
        }
        if (!status.getAsJsonObject("minecraft").get("last_session_error").isJsonNull()) {
            throw new IOException("world exploration produced a server session failure");
        }
    }

    private static void requireClockAdvanced(JsonObject before, JsonObject after)
            throws IOException {
        requireWorldObservable(after);
        long left = before.getAsJsonObject("world").get("defaultClockTime").getAsLong();
        long right = after.getAsJsonObject("world").get("defaultClockTime").getAsLong();
        if (right <= left) {
            throw new IOException("client world clock did not advance during sustained Play");
        }
    }

    private static void requireRestartClock(JsonObject before, JsonObject after)
            throws IOException {
        requireWorldObservable(after);
        long left = before.getAsJsonObject("world").get("defaultClockTime").getAsLong();
        long right = after.getAsJsonObject("world").get("defaultClockTime").getAsLong();
        if (right < left) {
            throw new IOException("client world clock regressed across formal restart");
        }
    }

    private static void requireSameTerrain(JsonObject before, JsonObject after) throws IOException {
        TreeSet<String> left = terrainSignature(before);
        TreeSet<String> right = terrainSignature(after);
        if (!left.equals(right)) {
            throw new IOException("spawn terrain did not converge to the same blocks after restart");
        }
    }

    private static TreeSet<String> terrainSignature(JsonObject observation) {
        TreeSet<String> signature = new TreeSet<>();
        for (var element : observation
                .getAsJsonObject("nearbyBlocks")
                .getAsJsonArray("blocks")) {
            JsonObject block = element.getAsJsonObject();
            signature.add(block.get("x").getAsInt()
                    + ":"
                    + block.get("y").getAsInt()
                    + ":"
                    + block.get("z").getAsInt()
                    + ":"
                    + block.get("blockId").getAsString());
        }
        return signature;
    }

    private static boolean compositeSessionReady(JsonObject status) {
        if (!status.has("minecraft") || status.get("minecraft").isJsonNull()) {
            return false;
        }
        JsonObject minecraft = status.getAsJsonObject("minecraft");
        return minecraft.get("committed_tick").getAsLong() > 0
                && minecraft.get("composite_region_commits").getAsInt() > 0
                && minecraft.getAsJsonArray("sessions").size() == 1
                && !onlySession(status).get("region_x").isJsonNull()
                && onlySession(status).get("sent_chunks").getAsInt() > 0;
    }

    private static boolean fullViewStreamed(JsonObject status) {
        if (!compositeSessionReady(status)) {
            return false;
        }
        JsonObject session = onlySession(status);
        return session.get("pending_chunks").getAsInt() == 0
                && session.get("sent_chunks").getAsInt()
                        == session.get("view_chunks").getAsInt();
    }

    private static boolean sessionDimension(JsonObject status, String dimension) {
        return compositeSessionReady(status)
                && dimension.equals(onlySession(status).get("dimension").getAsString());
    }

    private static JsonObject onlySession(JsonObject status) {
        return status.getAsJsonObject("minecraft")
                .getAsJsonArray("sessions")
                .get(0)
                .getAsJsonObject();
    }

    private static JsonObject radius(int radius) {
        JsonObject arguments = new JsonObject();
        arguments.addProperty("radius", radius);
        return arguments;
    }

    private static JsonObject waitForState(
            McpClient mcp, String operation, JsonObject criteria, int attempts)
            throws IOException, InterruptedException {
        IOException timeout = null;
        for (int attempt = 1; attempt <= attempts; attempt++) {
            try {
                return mcp.call(operation + "-" + attempt, "wait_for_state", criteria);
            } catch (IOException error) {
                if (!error.getMessage().contains("TimedOut")) {
                    throw error;
                }
                timeout = error;
            }
        }
        throw timeout;
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
