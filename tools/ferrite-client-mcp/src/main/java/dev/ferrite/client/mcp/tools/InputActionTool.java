package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import dev.ferrite.client.mcp.control.ControlledInput;
import java.util.EnumSet;
import java.util.Set;

/** Strict schemas for movement, jump, sneak, and sprint key ownership. */
final class InputActionTool implements McpTool {
    enum Kind {
        MOVEMENT,
        JUMP,
        SNEAK,
        SPRINT
    }

    private final ClientControl control;
    private final Kind kind;
    private final String name;

    InputActionTool(ClientControl control, Kind kind, String name) {
        this.control = control;
        this.kind = kind;
        this.name = name;
    }

    @Override
    public String name() {
        return name;
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        switch (kind) {
            case MOVEMENT -> {
                properties.add("forward", ToolSchemas.booleanProperty("Hold forward."));
                properties.add("backward", ToolSchemas.booleanProperty("Hold backward."));
                properties.add("left", ToolSchemas.booleanProperty("Hold left."));
                properties.add("right", ToolSchemas.booleanProperty("Hold right."));
                properties.add(
                        "ticks",
                        ToolSchemas.integerProperty(
                                "Client ticks to hold the selected keys.",
                                1,
                                ControlToolSupport.MAXIMUM_HELD_TICKS));
            }
            case JUMP -> properties.add(
                    "ticks", ToolSchemas.integerProperty("Client ticks to hold jump.", 1, 20));
            case SNEAK, SPRINT -> {
                properties.add("enabled", ToolSchemas.booleanProperty("Whether to hold the key."));
                properties.add(
                        "ticks",
                        ToolSchemas.integerProperty(
                                "Bounded hold duration when enabled.",
                                1,
                                ControlToolSupport.MAXIMUM_HELD_TICKS));
            }
        }
        return ToolSchemas.objectArguments(
                name,
                title(),
                description(),
                properties,
                requiredFields());
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        try {
            ClientAction.Inputs action = switch (kind) {
                case MOVEMENT -> movement(arguments);
                case JUMP -> singlePulse(arguments, ControlledInput.JUMP, 20);
                case SNEAK -> toggled(arguments, ControlledInput.SNEAK);
                case SPRINT -> toggled(arguments, ControlledInput.SPRINT);
            };
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }

    private ClientAction.Inputs movement(JsonObject arguments) {
        requireOnly(arguments, "actionId", "forward", "backward", "left", "right", "ticks");
        EnumSet<ControlledInput> inputs = EnumSet.noneOf(ControlledInput.class);
        addIf(arguments, "forward", ControlledInput.FORWARD, inputs);
        addIf(arguments, "backward", ControlledInput.BACKWARD, inputs);
        addIf(arguments, "left", ControlledInput.LEFT, inputs);
        addIf(arguments, "right", ControlledInput.RIGHT, inputs);
        if (inputs.containsAll(Set.of(ControlledInput.FORWARD, ControlledInput.BACKWARD))
                || inputs.containsAll(Set.of(ControlledInput.LEFT, ControlledInput.RIGHT))) {
            throw new IllegalArgumentException("opposing movement directions are not allowed");
        }
        if (inputs.isEmpty()) {
            throw new IllegalArgumentException("at least one movement direction must be true");
        }
        return new ClientAction.Inputs(
                ControlToolSupport.actionId(arguments),
                name,
                inputs,
                true,
                ControlToolSupport.boundedInt(
                        arguments, "ticks", 1, ControlToolSupport.MAXIMUM_HELD_TICKS));
    }

    private ClientAction.Inputs singlePulse(
            JsonObject arguments, ControlledInput input, int maximumTicks) {
        requireOnly(arguments, "actionId", "ticks");
        int ticks = arguments.has("ticks")
                ? ControlToolSupport.boundedInt(arguments, "ticks", 1, maximumTicks)
                : 1;
        return new ClientAction.Inputs(
                ControlToolSupport.actionId(arguments), name, Set.of(input), true, ticks);
    }

    private ClientAction.Inputs toggled(JsonObject arguments, ControlledInput input) {
        requireOnly(arguments, "actionId", "enabled", "ticks");
        boolean enabled = ControlToolSupport.bool(arguments, "enabled");
        if (!enabled && arguments.has("ticks")) {
            throw new IllegalArgumentException("ticks is only valid when enabled is true");
        }
        int ticks = enabled
                ? ControlToolSupport.boundedInt(
                        arguments, "ticks", 1, ControlToolSupport.MAXIMUM_HELD_TICKS)
                : 0;
        return new ClientAction.Inputs(
                ControlToolSupport.actionId(arguments), name, Set.of(input), enabled, ticks);
    }

    private static void addIf(
            JsonObject arguments,
            String field,
            ControlledInput input,
            EnumSet<ControlledInput> inputs) {
        if (arguments.has(field) && ControlToolSupport.bool(arguments, field)) {
            inputs.add(input);
        }
    }

    private static void requireOnly(JsonObject arguments, String... allowed) {
        Set<String> names = Set.of(allowed);
        if (!names.containsAll(arguments.keySet())) {
            throw new IllegalArgumentException("tool received an unsupported argument");
        }
    }

    private String[] requiredFields() {
        return switch (kind) {
            case MOVEMENT -> new String[] {"actionId", "ticks"};
            case JUMP -> new String[] {"actionId"};
            case SNEAK, SPRINT -> new String[] {"actionId", "enabled"};
        };
    }

    private String title() {
        return switch (kind) {
            case MOVEMENT -> "Hold movement";
            case JUMP -> "Jump";
            case SNEAK -> "Set sneaking";
            case SPRINT -> "Set sprinting";
        };
    }

    private String description() {
        return switch (kind) {
            case MOVEMENT -> "Hold bounded normal movement keys for client ticks.";
            case JUMP -> "Pulse the normal jump key for bounded client ticks.";
            case SNEAK -> "Acquire or release bounded sneak-key ownership.";
            case SPRINT -> "Acquire or release bounded sprint-key ownership.";
        };
    }
}
