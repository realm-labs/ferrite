package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;

/** Small schema builders shared by responsibility-local observation tools. */
final class ToolSchemas {
    private ToolSchemas() {}

    static JsonObject noArguments(String name, String title, String description) {
        return definition(name, title, description, new JsonObject());
    }

    static JsonObject boundedIntegerArgument(
            String name,
            String title,
            String description,
            String argument,
            String argumentDescription,
            int minimum,
            int maximum) {
        JsonObject integer = new JsonObject();
        integer.addProperty("type", "integer");
        integer.addProperty("description", argumentDescription);
        integer.addProperty("minimum", minimum);
        integer.addProperty("maximum", maximum);

        JsonObject properties = new JsonObject();
        properties.add(argument, integer);
        return definition(name, title, description, properties);
    }

    static boolean hasOnly(JsonObject arguments, String allowed) {
        return arguments.keySet().stream().allMatch(allowed::equals);
    }

    static McpToolResult rejected(String reason) {
        return failure("Rejected", reason);
    }

    static McpToolResult failure(String state, String reason) {
        JsonObject error = new JsonObject();
        error.addProperty("state", state);
        error.addProperty("reason", reason);
        return new McpToolResult(error, reason, true);
    }

    private static JsonObject definition(
            String name, String title, String description, JsonObject properties) {
        JsonObject schema = new JsonObject();
        schema.addProperty("type", "object");
        schema.add("properties", properties);
        schema.addProperty("additionalProperties", false);

        JsonObject definition = new JsonObject();
        definition.addProperty("name", name);
        definition.addProperty("title", title);
        definition.addProperty("description", description);
        definition.add("inputSchema", schema);
        return definition;
    }
}
