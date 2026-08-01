package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/** Immutable name-indexed MCP tool catalog. */
public final class ToolRegistry {
    private final Map<String, McpTool> tools;

    public ToolRegistry(List<McpTool> tools) {
        Map<String, McpTool> indexed = new LinkedHashMap<>();
        for (McpTool tool : tools) {
            if (indexed.putIfAbsent(tool.name(), tool) != null) {
                throw new IllegalArgumentException("duplicate MCP tool: " + tool.name());
            }
        }
        this.tools = Collections.unmodifiableMap(indexed);
    }

    public static ToolRegistry defaults() {
        return new ToolRegistry(List.of(new ClientStatusTool()));
    }

    public JsonObject listResponse() {
        JsonArray definitions = new JsonArray();
        tools.values().stream().map(McpTool::definition).forEach(definitions::add);

        JsonObject result = new JsonObject();
        result.add("tools", definitions);
        return result;
    }

    public Optional<McpTool> find(String name) {
        return Optional.ofNullable(tools.get(name));
    }
}
