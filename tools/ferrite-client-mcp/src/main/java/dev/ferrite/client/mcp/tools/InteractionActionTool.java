package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import dev.ferrite.client.mcp.control.ControlledInput;
import java.util.Set;

/** Normal key-click tools for attack, use, drop, and hand swap. */
final class InteractionActionTool implements McpTool {
    private static final int MAXIMUM_INTERACTION_TICKS = 20;

    private final ClientControl control;
    private final String name;
    private final ControlledInput input;
    private final boolean acceptsDuration;

    InteractionActionTool(
            ClientControl control, String name, ControlledInput input, boolean acceptsDuration) {
        this.control = control;
        this.name = name;
        this.input = input;
        this.acceptsDuration = acceptsDuration;
    }

    @Override
    public String name() {
        return name;
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        if (acceptsDuration) {
            properties.add(
                    "ticks",
                    ToolSchemas.integerProperty(
                            "Client ticks to hold the normal interaction key.",
                            1,
                            MAXIMUM_INTERACTION_TICKS));
        }
        return ToolSchemas.objectArguments(
                name,
                title(),
                description(),
                properties,
                "actionId");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        Set<String> allowed = acceptsDuration ? Set.of("actionId", "ticks") : Set.of("actionId");
        if (!allowed.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected(name + " received an unsupported argument");
        }
        try {
            int ticks = arguments.has("ticks")
                    ? ControlToolSupport.boundedInt(
                            arguments, "ticks", 1, MAXIMUM_INTERACTION_TICKS)
                    : 1;
            ClientAction action = new ClientAction.Inputs(
                    ControlToolSupport.actionId(arguments), name, Set.of(input), true, ticks);
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }

    private String title() {
        return switch (input) {
            case ATTACK -> "Attack";
            case USE -> "Use item";
            case DROP -> "Drop item";
            case SWAP_HANDS -> "Swap hands";
            case INVENTORY -> "Open inventory";
            default -> throw new IllegalStateException("unsupported interaction input");
        };
    }

    private String description() {
        return "Inject one bounded normal " + name + " key click into Minecraft's own handler.";
    }
}
