package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import java.util.Set;

/** Moves the real native cursor using current GUI-scaled coordinates. */
final class MoveCursorTool implements McpTool {
    private static final double MAXIMUM_GUI_COORDINATE = 16_384;
    private static final Set<String> ALLOWED = Set.of("actionId", "x", "y");

    private final ClientControl control;

    MoveCursorTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "move_cursor";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        properties.add("x", ToolSchemas.numberProperty("GUI-scaled X coordinate.", 0, MAXIMUM_GUI_COORDINATE));
        properties.add("y", ToolSchemas.numberProperty("GUI-scaled Y coordinate.", 0, MAXIMUM_GUI_COORDINATE));
        return ToolSchemas.objectArguments(
                name(), "Move cursor", "Move the native cursor within the current screen.", properties, "actionId", "x", "y");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("move_cursor received an unsupported argument");
        }
        try {
            ClientAction action = new ClientAction.MoveCursor(
                    ControlToolSupport.actionId(arguments),
                    ControlToolSupport.finiteDouble(arguments, "x", 0, MAXIMUM_GUI_COORDINATE),
                    ControlToolSupport.finiteDouble(arguments, "y", 0, MAXIMUM_GUI_COORDINATE));
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }
}
