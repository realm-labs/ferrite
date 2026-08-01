package dev.ferrite.client.mcp.protocol;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.tools.ToolRegistry;
import org.junit.jupiter.api.Test;

final class McpProtocolTest {
    @Test
    void lifecycleNegotiatesListsCallsPingsAndDeletes() {
        try (McpProtocol protocol = new McpProtocol(ToolRegistry.defaults(), "test")) {
            ProtocolReply initializedResponse = protocol.handle(initializeRequest(1), null, null);
            assertEquals(200, initializedResponse.status());
            String session = initializedResponse.sessionId().orElseThrow();
            assertEquals(
                    McpProtocol.LATEST_PROTOCOL_VERSION,
                    initializedResponse
                            .body()
                            .orElseThrow()
                            .getAsJsonObject("result")
                            .get("protocolVersion")
                            .getAsString());

            ProtocolReply premature = protocol.handle(
                    request(2, "tools/list", "{}"), session, McpProtocol.LATEST_PROTOCOL_VERSION);
            assertEquals(-32002, errorCode(premature));

            ProtocolReply notification = protocol.handle(
                    "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}",
                    session,
                    McpProtocol.LATEST_PROTOCOL_VERSION);
            assertEquals(202, notification.status());
            assertTrue(notification.body().isEmpty());

            ProtocolReply list = protocol.handle(
                    request(3, "tools/list", "{}"), session, McpProtocol.LATEST_PROTOCOL_VERSION);
            JsonObject tool = list.body()
                    .orElseThrow()
                    .getAsJsonObject("result")
                    .getAsJsonArray("tools")
                    .get(0)
                    .getAsJsonObject();
            assertEquals("client_status", tool.get("name").getAsString());

            ProtocolReply call = protocol.handle(
                    request(
                            4,
                            "tools/call",
                            "{\"name\":\"client_status\",\"arguments\":{}}"),
                    session,
                    McpProtocol.LATEST_PROTOCOL_VERSION);
            JsonObject structured = call.body()
                    .orElseThrow()
                    .getAsJsonObject("result")
                    .getAsJsonObject("structuredContent");
            assertEquals("Ready", structured.get("state").getAsString());
            assertFalse(structured.get("gameObservationAvailable").getAsBoolean());

            ProtocolReply ping = protocol.handle(
                    request(5, "ping", "{}"), session, McpProtocol.LATEST_PROTOCOL_VERSION);
            assertTrue(ping.body().orElseThrow().getAsJsonObject("result").isEmpty());
            assertTrue(protocol.deleteSession(session));
            assertFalse(protocol.deleteSession(session));
        }
    }

    @Test
    void malformedBatchWrongVersionAndConcurrentSessionFailClosed() {
        try (McpProtocol protocol = new McpProtocol(ToolRegistry.defaults(), "test")) {
            assertEquals(-32700, errorCode(protocol.handle("{", null, null)));
            assertEquals(-32600, errorCode(protocol.handle("[]", null, null)));

            ProtocolReply first = protocol.handle(initializeRequest(1), null, null);
            String session = first.sessionId().orElseThrow();
            ProtocolReply second = protocol.handle(initializeRequest(2), null, null);
            assertEquals(409, second.status());
            assertEquals(-32000, errorCode(second));

            ProtocolReply wrongVersion = protocol.handle(request(3, "ping", "{}"), session, "wrong");
            assertEquals(400, wrongVersion.status());
            assertEquals(-32600, errorCode(wrongVersion));
        }
    }

    private static String initializeRequest(int id) {
        return request(
                id,
                "initialize",
                "{\"protocolVersion\":\"2025-11-25\",\"capabilities\":{},"
                        + "\"clientInfo\":{\"name\":\"test\",\"version\":\"1\"}}");
    }

    private static String request(int id, String method, String params) {
        return "{\"jsonrpc\":\"2.0\",\"id\":"
                + id
                + ",\"method\":\""
                + method
                + "\",\"params\":"
                + params
                + "}";
    }

    private static int errorCode(ProtocolReply reply) {
        return reply.body().orElseThrow().getAsJsonObject("error").get("code").getAsInt();
    }
}
