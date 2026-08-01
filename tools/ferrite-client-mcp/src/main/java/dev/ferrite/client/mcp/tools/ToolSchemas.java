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

    static JsonObject objectArguments(
            String name,
            String title,
            String description,
            JsonObject properties,
            String... required) {
        JsonObject definition = definition(name, title, description, properties);
        if (required.length > 0) {
            com.google.gson.JsonArray names = new com.google.gson.JsonArray();
            for (String field : required) {
                names.add(field);
            }
            definition.getAsJsonObject("inputSchema").add("required", names);
        }
        return definition;
    }

    static JsonObject stringProperty(String description) {
        return property("string", description);
    }

    static JsonObject booleanProperty(String description) {
        return property("boolean", description);
    }

    static JsonObject numberProperty(String description, double minimum, double maximum) {
        JsonObject property = property("number", description);
        property.addProperty("minimum", minimum);
        property.addProperty("maximum", maximum);
        return property;
    }

    static JsonObject integerProperty(String description, int minimum, int maximum) {
        JsonObject property = property("integer", description);
        property.addProperty("minimum", minimum);
        property.addProperty("maximum", maximum);
        return property;
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

    private static JsonObject property(String type, String description) {
        JsonObject property = new JsonObject();
        property.addProperty("type", type);
        property.addProperty("description", description);
        return property;
    }
}
