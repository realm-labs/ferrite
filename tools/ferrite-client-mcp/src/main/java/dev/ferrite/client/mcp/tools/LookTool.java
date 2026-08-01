package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import java.util.Set;

/** Applies a bounded absolute or relative player-camera rotation. */
final class LookTool implements McpTool {
    private static final Set<String> ALLOWED = Set.of("actionId", "yaw", "pitch", "relative");

    private final ClientControl control;

    LookTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "look";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        properties.add("yaw", ToolSchemas.numberProperty("Yaw degrees.", -360.0, 360.0));
        properties.add("pitch", ToolSchemas.numberProperty("Pitch degrees.", -90.0, 90.0));
        properties.add(
                "relative",
                ToolSchemas.booleanProperty("Apply the supplied degrees relative to the current view."));
        return ToolSchemas.objectArguments(
                name(),
                "Look",
                "Rotate the real local player's view on the client thread.",
                properties,
                "actionId",
                "yaw",
                "pitch");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("look received an unsupported argument");
        }
        try {
            ClientAction action = new ClientAction.Look(
                    ControlToolSupport.actionId(arguments),
                    ControlToolSupport.finiteFloat(arguments, "yaw", -360.0f, 360.0f),
                    ControlToolSupport.finiteFloat(arguments, "pitch", -90.0f, 90.0f),
                    ControlToolSupport.optionalBool(arguments, "relative", false));
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
