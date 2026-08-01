package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;

/** Structured and text-compatible result returned by an MCP tool. */
public record McpToolResult(JsonObject structuredContent, String text, boolean error) {
    public JsonObject toJson() {
        JsonObject contentBlock = new JsonObject();
        contentBlock.addProperty("type", "text");
        contentBlock.addProperty("text", text);

        JsonArray content = new JsonArray();
        content.add(contentBlock);

        JsonObject result = new JsonObject();
        result.add("content", content);
        result.add("structuredContent", structuredContent);
        result.addProperty("isError", error);
        return result;
    }
}
