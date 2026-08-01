package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.capture.ScreenshotCapture;
import dev.ferrite.client.mcp.control.ClientControl;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
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
        return forObservations(new ClientObservationStore());
    }

    public static ToolRegistry forObservations(ClientObservationStore observations) {
        return forObservations(observations, ScreenshotCapture.unavailable());
    }

    public static ToolRegistry forObservations(
            ClientObservationStore observations, ScreenshotCapture screenshotCapture) {
        return forClient(observations, screenshotCapture, ClientControl.unavailable());
    }

    public static ToolRegistry forClient(
            ClientObservationStore observations,
            ScreenshotCapture screenshotCapture,
            ClientControl control) {
        return new ToolRegistry(List.of(
                new ClientStatusTool(observations),
                new WaitForStateTool(observations),
                new ReleaseAllInputsTool(control),
                new ActionStatusTool(control),
                new PlayerStateTool(observations),
                new InventoryStateTool(observations),
                new CrosshairStateTool(observations),
                new ScreenStateTool(observations),
                new NearbyBlocksTool(observations),
                new ClientErrorsTool(observations),
                new TakeScreenshotTool(screenshotCapture),
                new LookTool(control),
                new InputActionTool(control, InputActionTool.Kind.MOVEMENT, "hold_movement"),
                new InputActionTool(control, InputActionTool.Kind.JUMP, "jump"),
                new InputActionTool(control, InputActionTool.Kind.SNEAK, "set_sneaking"),
                new InputActionTool(control, InputActionTool.Kind.SPRINT, "set_sprinting")));
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
