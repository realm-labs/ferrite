package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;

/** One stable MCP tool definition and its bounded invocation. */
public interface McpTool {
    String name();

    JsonObject definition();

    McpToolResult call(JsonObject arguments, ToolContext context);
}
