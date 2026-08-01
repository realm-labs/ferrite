package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.observation.ClientError;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.ObservationJson;
import java.util.List;

/** Reads the bounded redacted client failure ring. */
public final class ClientErrorsTool implements McpTool {
    private static final int DEFAULT_LIMIT = 16;
    private static final int MAXIMUM_LIMIT = 64;

    private final ClientObservationStore observations;

    public ClientErrorsTool(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public String name() {
        return "client_errors";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.boundedIntegerArgument(
                name(),
                "Client errors",
                "Read recent redacted instrumentation and connection failures.",
                "limit",
                "Maximum newest events to return; defaults to sixteen.",
                1,
                MAXIMUM_LIMIT);
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ToolSchemas.hasOnly(arguments, "limit")) {
            return ToolSchemas.rejected("client_errors accepts only limit");
        }
        Integer limit = readLimit(arguments);
        if (limit == null) {
            return ToolSchemas.rejected("limit must be an integer between 1 and 64");
        }
        List<ClientError> errors = observations.errors(limit);
        ErrorObservation result =
                new ErrorObservation(observations.latest().clientTick(), errors.size(), errors);
        return new McpToolResult(
                ObservationJson.object(result),
                errors.isEmpty() ? "No client errors recorded" : "Client errors observed",
                false);
    }

    private static Integer readLimit(JsonObject arguments) {
        if (!arguments.has("limit")) {
            return DEFAULT_LIMIT;
        }
        JsonElement limit = arguments.get("limit");
        if (!limit.isJsonPrimitive() || !limit.getAsJsonPrimitive().isNumber()) {
            return null;
        }
        double value = limit.getAsDouble();
        int integer = limit.getAsInt();
        return Double.isFinite(value)
                        && value == integer
                        && integer >= 1
                        && integer <= MAXIMUM_LIMIT
                ? integer
                : null;
    }

    private record ErrorObservation(long clientTick, int count, List<ClientError> errors) {}
}
