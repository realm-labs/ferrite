package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Structured and text-compatible result returned by an MCP tool. */
public record McpToolResult(JsonObject structuredContent, JsonArray content, boolean error) {
    public McpToolResult(JsonObject structuredContent, String text, boolean error) {
        this(structuredContent, textContent(text), error);
    }

    public static McpToolResult image(
            JsonObject structuredContent, String text, String base64Png) {
        JsonArray content = textContent(text);
        JsonObject image = new JsonObject();
        image.addProperty("type", "image");
        image.addProperty("data", base64Png);
        image.addProperty("mimeType", "image/png");
        content.add(image);
        return new McpToolResult(structuredContent, content, false);
    }

    public JsonObject toJson() {
        JsonObject result = new JsonObject();
        result.add("content", content);
        result.add("structuredContent", structuredContent);
        result.addProperty("isError", error);
        return result;
    }

    private static JsonArray textContent(String text) {
        JsonObject contentBlock = new JsonObject();
        contentBlock.addProperty("type", "text");
        contentBlock.addProperty("text", text);

        JsonArray content = new JsonArray();
        content.add(contentBlock);
        return content;
    }
}
