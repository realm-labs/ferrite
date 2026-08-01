package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ClientSnapshot;
import java.util.Set;

/** Waits on immutable observation publication and client-tick bounds without polling sleeps. */
final class WaitForStateTool implements McpTool {
    private static final int DEFAULT_MAXIMUM_TICKS = 100;
    private static final int MAXIMUM_TICKS = 400;
    private static final long MAXIMUM_WALL_MILLIS = 30_000;
    private static final Set<String> ALLOWED = Set.of(
            "afterClientTick",
            "connectionState",
            "screenType",
            "playerAvailable",
            "onGround",
            "maxTicks");

    private final ClientObservationStore observations;

    WaitForStateTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "wait_for_state";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add(
                "afterClientTick",
                ToolSchemas.integerProperty(
                        "Require an observation strictly after this client tick.", 0, Integer.MAX_VALUE));
        properties.add(
                "connectionState",
                ToolSchemas.stringProperty("Exact connection state, such as PLAY or DISCONNECTED."));
        properties.add(
                "screenType",
                ToolSchemas.stringProperty("Exact observed screen class, or NONE for gameplay."));
        properties.add(
                "playerAvailable",
                ToolSchemas.booleanProperty("Whether a local player must be available."));
        properties.add(
                "onGround", ToolSchemas.booleanProperty("Required local-player on-ground state."));
        properties.add(
                "maxTicks",
                ToolSchemas.integerProperty(
                        "Maximum observed client ticks before timeout.", 1, MAXIMUM_TICKS));
        return ToolSchemas.objectArguments(
                name(),
                "Wait for client state",
                "Wait for structured state across published client ticks with a hard wall guard.",
                properties);
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("wait_for_state received an unsupported argument");
        }
        try {
            Criteria criteria = Criteria.parse(arguments);
            int maxTicks = arguments.has("maxTicks")
                    ? ControlToolSupport.boundedInt(arguments, "maxTicks", 1, MAXIMUM_TICKS)
                    : DEFAULT_MAXIMUM_TICKS;
            return waitFor(criteria, maxTicks);
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            return ToolSchemas.failure("Cancelled", "state wait was interrupted");
        }
    }

    private McpToolResult waitFor(Criteria criteria, int maxTicks) throws InterruptedException {
        ClientSnapshot snapshot = observations.latest();
        long startTick = snapshot.clientTick();
        long deadlineNanos = System.nanoTime()
                + Math.min(MAXIMUM_WALL_MILLIS, Math.max(1_000L, maxTicks * 100L))
                        * 1_000_000L;
        while (!criteria.matches(snapshot)
                && snapshot.clientTick() - startTick < maxTicks
                && System.nanoTime() < deadlineNanos) {
            long remainingMillis = Math.max(1, (deadlineNanos - System.nanoTime()) / 1_000_000L);
            snapshot = observations.awaitNext(snapshot.clientTick(), remainingMillis);
        }

        boolean satisfied = criteria.matches(snapshot);
        JsonObject result = new JsonObject();
        result.addProperty("state", satisfied ? "Satisfied" : "TimedOut");
        result.addProperty("startTick", startTick);
        result.addProperty("clientTick", snapshot.clientTick());
        result.addProperty("elapsedTicks", Math.max(0, snapshot.clientTick() - startTick));
        result.add("criteria", criteria.toJson());
        result.addProperty("connectionState", snapshot.connection().state());
        result.addProperty("screenType", snapshot.screen().type());
        result.addProperty("playerAvailable", snapshot.player() != null);
        if (snapshot.player() != null) {
            result.addProperty("onGround", snapshot.player().onGround());
        }
        return new McpToolResult(
                result,
                satisfied ? "Client state condition satisfied" : "Client state condition timed out",
                !satisfied);
    }

    private record Criteria(
            Long afterClientTick,
            String connectionState,
            String screenType,
            Boolean playerAvailable,
            Boolean onGround) {
        static Criteria parse(JsonObject arguments) {
            Long afterTick = arguments.has("afterClientTick")
                    ? ControlToolSupport.nonNegativeLong(arguments, "afterClientTick")
                    : null;
            String connection = arguments.has("connectionState")
                    ? ControlToolSupport.string(arguments, "connectionState")
                    : null;
            String screen = arguments.has("screenType")
                    ? ControlToolSupport.string(arguments, "screenType")
                    : null;
            Boolean player = arguments.has("playerAvailable")
                    ? ControlToolSupport.bool(arguments, "playerAvailable")
                    : null;
            Boolean ground = arguments.has("onGround")
                    ? ControlToolSupport.bool(arguments, "onGround")
                    : null;
            if (afterTick == null
                    && connection == null
                    && screen == null
                    && player == null
                    && ground == null) {
                throw new IllegalArgumentException("wait_for_state requires at least one condition");
            }
            if (connection != null && (connection.isBlank() || connection.length() > 32)) {
                throw new IllegalArgumentException("connectionState must contain 1 to 32 characters");
            }
            if (screen != null && (screen.isBlank() || screen.length() > 128)) {
                throw new IllegalArgumentException("screenType must contain 1 to 128 characters");
            }
            return new Criteria(afterTick, connection, screen, player, ground);
        }

        boolean matches(ClientSnapshot snapshot) {
            if (afterClientTick != null && snapshot.clientTick() <= afterClientTick) {
                return false;
            }
            if (connectionState != null
                    && !connectionState.equals(snapshot.connection().state())) {
                return false;
            }
            if (screenType != null && !screenType.equals(snapshot.screen().type())) {
                return false;
            }
            boolean available = snapshot.player() != null;
            if (playerAvailable != null && playerAvailable != available) {
                return false;
            }
            return onGround == null
                    || (snapshot.player() != null && onGround == snapshot.player().onGround());
        }

        JsonObject toJson() {
            JsonObject json = new JsonObject();
            if (afterClientTick != null) {
                json.addProperty("afterClientTick", afterClientTick);
            }
            if (connectionState != null) {
                json.addProperty("connectionState", connectionState);
            }
            if (screenType != null) {
                json.addProperty("screenType", screenType);
            }
            if (playerAvailable != null) {
                json.addProperty("playerAvailable", playerAvailable);
            }
            if (onGround != null) {
                json.addProperty("onGround", onGround);
            }
            return json;
        }
    }
}
