package dev.ferrite.client.mcp.protocol;

import com.google.gson.JsonObject;
import java.util.Optional;

/** Transport-neutral result of handling one Streamable HTTP MCP message. */
public record ProtocolReply(int status, Optional<JsonObject> body, Optional<String> sessionId) {
    static ProtocolReply json(int status, JsonObject body) {
        return new ProtocolReply(status, Optional.of(body), Optional.empty());
    }

    static ProtocolReply json(int status, JsonObject body, String sessionId) {
        return new ProtocolReply(status, Optional.of(body), Optional.of(sessionId));
    }

    static ProtocolReply accepted() {
        return new ProtocolReply(202, Optional.empty(), Optional.empty());
    }
}
