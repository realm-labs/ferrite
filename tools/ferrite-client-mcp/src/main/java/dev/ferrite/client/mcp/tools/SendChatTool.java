package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.control.ClientAction;
import dev.ferrite.client.mcp.control.ClientControl;
import java.util.Set;

/** Sends bounded ordinary chat while explicitly rejecting the command path. */
final class SendChatTool implements McpTool {
    private static final int MAXIMUM_CHAT_CODE_POINTS = 256;
    private static final Set<String> ALLOWED = Set.of("actionId", "message");

    private final ClientControl control;

    SendChatTool(ClientControl control) {
        this.control = control;
    }

    @Override
    public String name() {
        return "send_chat";
    }

    @Override
    public JsonObject definition() {
        JsonObject properties = new JsonObject();
        properties.add("actionId", ToolSchemas.stringProperty("Unique action identifier."));
        JsonObject message = ToolSchemas.stringProperty(
                "Ordinary chat text. Slash commands and control characters are rejected.");
        message.addProperty("minLength", 1);
        message.addProperty("maxLength", MAXIMUM_CHAT_CODE_POINTS);
        properties.add("message", message);
        return ToolSchemas.objectArguments(
                name(),
                "Send chat",
                "Send bounded ordinary chat through the active Minecraft client connection.",
                properties,
                "actionId",
                "message");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!ALLOWED.containsAll(arguments.keySet())) {
            return ToolSchemas.rejected("send_chat received an unsupported argument");
        }
        try {
            String message = ControlToolSupport.string(arguments, "message");
            validateMessage(message);
            ClientAction action = new ClientAction.SendChat(
                    ControlToolSupport.actionId(arguments), message);
            return ControlToolSupport.receipt(control.submit(action));
        } catch (IllegalArgumentException error) {
            return ToolSchemas.rejected(error.getMessage());
        }
    }

    private static void validateMessage(String message) {
        if (message.isBlank()) {
            throw new IllegalArgumentException("chat message must not be blank");
        }
        if (message.startsWith("/")) {
            throw new IllegalArgumentException("server commands are not permitted");
        }
        if (message.codePointCount(0, message.length()) > MAXIMUM_CHAT_CODE_POINTS) {
            throw new IllegalArgumentException("chat message exceeds 256 code points");
        }
        if (message.codePoints().anyMatch(Character::isISOControl)) {
            throw new IllegalArgumentException("chat message contains a control character");
        }
    }
}
