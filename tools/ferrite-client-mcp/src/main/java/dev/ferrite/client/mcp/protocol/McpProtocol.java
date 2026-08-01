package dev.ferrite.client.mcp.protocol;

import com.google.gson.Gson;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParseException;
import com.google.gson.JsonParser;
import dev.ferrite.client.mcp.tools.McpTool;
import dev.ferrite.client.mcp.tools.ToolContext;
import dev.ferrite.client.mcp.tools.ToolRegistry;
import java.util.Set;

/** JSON-RPC and MCP lifecycle dispatcher independent of Minecraft runtime classes. */
public final class McpProtocol implements AutoCloseable {
    public static final String LATEST_PROTOCOL_VERSION = "2025-11-25";
    public static final String LEGACY_PROTOCOL_VERSION = "2025-06-18";

    private static final Set<String> SUPPORTED_PROTOCOL_VERSIONS =
            Set.of(LATEST_PROTOCOL_VERSION, LEGACY_PROTOCOL_VERSION);

    private final Gson gson = new Gson();
    private final McpSessionRegistry sessions = new McpSessionRegistry();
    private final ToolRegistry tools;
    private final String serverVersion;

    public McpProtocol(ToolRegistry tools, String serverVersion) {
        this.tools = tools;
        this.serverVersion = serverVersion;
    }

    public ProtocolReply handle(String body, String sessionId, String protocolVersionHeader) {
        JsonElement parsed;
        try {
            parsed = JsonParser.parseString(body);
        } catch (JsonParseException error) {
            return ProtocolReply.json(400, error(null, -32700, "Parse error"));
        }
        if (!parsed.isJsonObject()) {
            return ProtocolReply.json(400, error(null, -32600, "Batch and non-object messages are not supported"));
        }

        JsonObject request = parsed.getAsJsonObject();
        JsonElement id = request.get("id");
        if (!validEnvelope(request, id)) {
            return ProtocolReply.json(400, error(validId(id) ? id : null, -32600, "Invalid Request"));
        }

        String method = request.get("method").getAsString();
        if ("initialize".equals(method)) {
            return initialize(request, id, sessionId);
        }
        if ("ping".equals(method) && sessionId == null) {
            return requireRequestId(id, emptyResult(id));
        }

        McpSession session = sessions.find(sessionId).orElse(null);
        if (session == null) {
            return ProtocolReply.json(404, error(id, -32001, "Unknown or expired MCP session"));
        }
        if (!session.protocolVersion().equals(protocolVersionHeader)) {
            return ProtocolReply.json(400, error(id, -32600, "Missing or mismatched MCP-Protocol-Version"));
        }

        return switch (method) {
            case "notifications/initialized" -> initialized(request, id, session);
            case "notifications/cancelled" -> notificationOnly(id);
            case "ping" -> requireRequestId(id, emptyResult(id));
            case "tools/list" -> toolsList(request, id, session);
            case "tools/call" -> toolsCall(request, id, session);
            default -> id == null
                    ? ProtocolReply.accepted()
                    : ProtocolReply.json(200, error(id, -32601, "Method not found"));
        };
    }

    public boolean deleteSession(String sessionId) {
        return sessions.remove(sessionId);
    }

    public String toJson(JsonObject value) {
        return gson.toJson(value);
    }

    @Override
    public void close() {
        sessions.clear();
    }

    private ProtocolReply initialize(JsonObject request, JsonElement id, String sessionId) {
        if (id == null) {
            return ProtocolReply.json(400, error(null, -32600, "initialize must be a request"));
        }
        if (sessionId != null) {
            return ProtocolReply.json(400, error(id, -32600, "initialize must not carry a session ID"));
        }
        JsonObject params = objectMember(request, "params");
        if (!validInitializeParams(params)) {
            return ProtocolReply.json(200, error(id, -32602, "Invalid initialize parameters"));
        }

        String requestedVersion = params.get("protocolVersion").getAsString();
        String negotiatedVersion = SUPPORTED_PROTOCOL_VERSIONS.contains(requestedVersion)
                ? requestedVersion
                : LATEST_PROTOCOL_VERSION;
        McpSession session = sessions.create(negotiatedVersion).orElse(null);
        if (session == null) {
            return ProtocolReply.json(409, error(id, -32000, "An MCP control session is already active"));
        }

        JsonObject toolsCapability = new JsonObject();
        toolsCapability.addProperty("listChanged", false);
        JsonObject capabilities = new JsonObject();
        capabilities.add("tools", toolsCapability);

        JsonObject serverInfo = new JsonObject();
        serverInfo.addProperty("name", "ferrite-client-mcp");
        serverInfo.addProperty("title", "Ferrite instrumented Minecraft client");
        serverInfo.addProperty("version", serverVersion);

        JsonObject result = new JsonObject();
        result.addProperty("protocolVersion", negotiatedVersion);
        result.add("capabilities", capabilities);
        result.add("serverInfo", serverInfo);
        result.addProperty(
                "instructions",
                "Use bounded client tools for Ferrite acceptance. Tool success means client-side application, not server acceptance.");
        return ProtocolReply.json(200, success(id, result), session.id());
    }

    private ProtocolReply initialized(JsonObject request, JsonElement id, McpSession session) {
        if (id != null || request.has("params")) {
            return ProtocolReply.json(400, error(id, -32600, "initialized must be a parameterless notification"));
        }
        session.markInitialized();
        return ProtocolReply.accepted();
    }

    private ProtocolReply toolsList(JsonObject request, JsonElement id, McpSession session) {
        ProtocolReply lifecycleError = requireOperationalRequest(id, session);
        if (lifecycleError != null) {
            return lifecycleError;
        }
        JsonObject params = objectMember(request, "params");
        if (params != null && params.has("cursor")) {
            return ProtocolReply.json(200, error(id, -32602, "Pagination cursors are not supported"));
        }
        return ProtocolReply.json(200, success(id, tools.listResponse()));
    }

    private ProtocolReply toolsCall(JsonObject request, JsonElement id, McpSession session) {
        ProtocolReply lifecycleError = requireOperationalRequest(id, session);
        if (lifecycleError != null) {
            return lifecycleError;
        }
        JsonObject params = objectMember(request, "params");
        if (params == null || !stringMember(params, "name")) {
            return ProtocolReply.json(200, error(id, -32602, "tools/call requires a tool name"));
        }
        JsonObject arguments = objectMember(params, "arguments");
        if (params.has("arguments") && arguments == null) {
            return ProtocolReply.json(200, error(id, -32602, "Tool arguments must be an object"));
        }

        String name = params.get("name").getAsString();
        McpTool tool = tools.find(name).orElse(null);
        if (tool == null) {
            return ProtocolReply.json(200, error(id, -32602, "Unknown tool: " + name));
        }
        JsonObject result;
        try {
            result = tool.call(
                            arguments == null ? new JsonObject() : arguments,
                            new ToolContext(session.protocolVersion()))
                    .toJson();
        } catch (RuntimeException error) {
            result = failedToolResult();
        }
        return ProtocolReply.json(200, success(id, result));
    }

    private static ProtocolReply requireOperationalRequest(JsonElement id, McpSession session) {
        if (id == null) {
            return ProtocolReply.json(400, error(null, -32600, "Operation must be a request"));
        }
        if (!session.initialized()) {
            return ProtocolReply.json(400, error(id, -32002, "MCP session is not initialized"));
        }
        return null;
    }

    private static ProtocolReply requireRequestId(JsonElement id, JsonObject response) {
        return id == null
                ? ProtocolReply.json(400, error(null, -32600, "ping must be a request"))
                : ProtocolReply.json(200, response);
    }

    private static ProtocolReply notificationOnly(JsonElement id) {
        return id == null
                ? ProtocolReply.accepted()
                : ProtocolReply.json(400, error(id, -32600, "Expected a notification"));
    }

    private static boolean validEnvelope(JsonObject request, JsonElement id) {
        return stringValue(request, "jsonrpc", "2.0")
                && stringMember(request, "method")
                && validId(id);
    }

    private static boolean validId(JsonElement id) {
        return id == null
                || id.isJsonNull()
                || (id.isJsonPrimitive()
                        && (id.getAsJsonPrimitive().isString()
                                || id.getAsJsonPrimitive().isNumber()));
    }

    private static boolean validInitializeParams(JsonObject params) {
        if (params == null
                || !stringMember(params, "protocolVersion")
                || objectMember(params, "capabilities") == null) {
            return false;
        }
        JsonObject clientInfo = objectMember(params, "clientInfo");
        return clientInfo != null
                && stringMember(clientInfo, "name")
                && stringMember(clientInfo, "version");
    }

    private static JsonObject objectMember(JsonObject object, String name) {
        if (object == null || !object.has(name) || !object.get(name).isJsonObject()) {
            return null;
        }
        return object.getAsJsonObject(name);
    }

    private static boolean stringMember(JsonObject object, String name) {
        return object.has(name)
                && object.get(name).isJsonPrimitive()
                && object.get(name).getAsJsonPrimitive().isString();
    }

    private static boolean stringValue(JsonObject object, String name, String expected) {
        return stringMember(object, name) && expected.equals(object.get(name).getAsString());
    }

    private static JsonObject emptyResult(JsonElement id) {
        return success(id, new JsonObject());
    }

    private static JsonObject success(JsonElement id, JsonObject result) {
        JsonObject response = new JsonObject();
        response.addProperty("jsonrpc", "2.0");
        response.add("id", id.deepCopy());
        response.add("result", result);
        return response;
    }

    private static JsonObject error(JsonElement id, int code, String message) {
        JsonObject detail = new JsonObject();
        detail.addProperty("code", code);
        detail.addProperty("message", message);

        JsonObject response = new JsonObject();
        response.addProperty("jsonrpc", "2.0");
        response.add("id", id == null ? null : id.deepCopy());
        response.add("error", detail);
        return response;
    }

    private static JsonObject failedToolResult() {
        JsonObject structured = new JsonObject();
        structured.addProperty("state", "Rejected");
        structured.addProperty("reason", "tool failed safely");

        JsonObject text = new JsonObject();
        text.addProperty("type", "text");
        text.addProperty("text", "tool failed safely");
        JsonArray content = new JsonArray();
        content.add(text);

        JsonObject result = new JsonObject();
        result.add("content", content);
        result.add("structuredContent", structured);
        result.addProperty("isError", true);
        return result;
    }
}
