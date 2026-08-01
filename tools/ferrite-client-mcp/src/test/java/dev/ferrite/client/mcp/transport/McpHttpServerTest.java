package dev.ferrite.client.mcp.transport;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import dev.ferrite.client.mcp.config.McpConfig;
import dev.ferrite.client.mcp.protocol.McpProtocol;
import dev.ferrite.client.mcp.tools.ToolRegistry;
import java.io.IOException;
import java.net.InetAddress;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermission;
import java.util.Optional;
import java.util.Set;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class McpHttpServerTest {
    private static final String SECRET = "0123456789abcdef0123456789abcdef0123456789abcdef";
    private static final String ACCEPT = "application/json, text/event-stream";

    @TempDir Path temporaryDirectory;

    private final HttpClient client = HttpClient.newHttpClient();
    private McpHttpServer server;
    private URI endpoint;
    private Path readyFile;

    @BeforeEach
    void startServer() throws IOException {
        Path secretFile = temporaryDirectory.resolve("secret");
        Files.writeString(secretFile, SECRET, StandardCharsets.UTF_8);
        setOwnerOnly(secretFile);
        readyFile = temporaryDirectory.resolve("runtime/ready.json");
        McpConfig config = new McpConfig(
                InetAddress.getByName("127.0.0.1"),
                0,
                secretFile.toAbsolutePath(),
                Optional.of(readyFile.toAbsolutePath()),
                512,
                2,
                4);
        server = new McpHttpServer(
                config, new McpProtocol(ToolRegistry.defaults(), "test"));
        endpoint = URI.create("http://127.0.0.1:" + server.start() + "/mcp");
    }

    @AfterEach
    void stopServer() throws IOException {
        server.close();
    }

    @Test
    void endpointPublishesReadyFileAndCompletesMcpLifecycle() throws Exception {
        assertTrue(Files.readString(readyFile).contains(endpoint.toString()));

        HttpResponse<String> initialize = post(initializeRequest(), null, null, null, SECRET);
        assertEquals(200, initialize.statusCode());
        String session = initialize.headers().firstValue("Mcp-Session-Id").orElseThrow();
        JsonObject initializeBody = parse(initialize);
        assertEquals(
                McpProtocol.LATEST_PROTOCOL_VERSION,
                initializeBody
                        .getAsJsonObject("result")
                        .get("protocolVersion")
                        .getAsString());

        HttpResponse<String> initialized = post(
                "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}",
                session,
                McpProtocol.LATEST_PROTOCOL_VERSION,
                null,
                SECRET);
        assertEquals(202, initialized.statusCode());

        HttpResponse<String> tools = post(
                request(2, "tools/list", "{}"),
                session,
                McpProtocol.LATEST_PROTOCOL_VERSION,
                null,
                SECRET);
        assertEquals("client_status", parse(tools)
                .getAsJsonObject("result")
                .getAsJsonArray("tools")
                .get(0)
                .getAsJsonObject()
                .get("name")
                .getAsString());

        HttpRequest delete = HttpRequest.newBuilder(endpoint)
                .header("Authorization", "Bearer " + SECRET)
                .header("Mcp-Session-Id", session)
                .DELETE()
                .build();
        assertEquals(204, client.send(delete, HttpResponse.BodyHandlers.ofString()).statusCode());
        assertEquals(404, client.send(delete, HttpResponse.BodyHandlers.ofString()).statusCode());
    }

    @Test
    void authenticationOriginMediaBoundsAndMethodsFailClosed() throws Exception {
        assertEquals(401, post(initializeRequest(), null, null, null, "wrong").statusCode());
        assertEquals(
                403,
                post(initializeRequest(), null, null, "https://attacker.example", SECRET)
                        .statusCode());

        HttpRequest get = HttpRequest.newBuilder(endpoint)
                .header("Authorization", "Bearer " + SECRET)
                .GET()
                .build();
        assertEquals(405, client.send(get, HttpResponse.BodyHandlers.ofString()).statusCode());

        HttpRequest wrongType = HttpRequest.newBuilder(endpoint)
                .header("Authorization", "Bearer " + SECRET)
                .header("Accept", ACCEPT)
                .header("Content-Type", "text/plain")
                .POST(HttpRequest.BodyPublishers.ofString(initializeRequest()))
                .build();
        assertEquals(
                415,
                client.send(wrongType, HttpResponse.BodyHandlers.ofString()).statusCode());

        String oversized = "{\"padding\":\"" + "x".repeat(600) + "\"}";
        assertEquals(413, post(oversized, null, null, null, SECRET).statusCode());
    }

    @Test
    void closeRemovesSecretFreeReadyFile() throws IOException {
        String contents = Files.readString(readyFile);
        assertFalse(contents.contains(SECRET));
        server.close();
        assertFalse(Files.exists(readyFile));

        server = new McpHttpServer(
                new McpConfig(
                        InetAddress.getByName("127.0.0.1"),
                        0,
                        temporaryDirectory.resolve("secret").toAbsolutePath(),
                        Optional.empty(),
                        512,
                        1,
                        1),
                new McpProtocol(ToolRegistry.defaults(), "test"));
    }

    private HttpResponse<String> post(
            String body, String session, String version, String origin, String secret)
            throws IOException, InterruptedException {
        HttpRequest.Builder request = HttpRequest.newBuilder(endpoint)
                .header("Authorization", "Bearer " + secret)
                .header("Accept", ACCEPT)
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body));
        if (session != null) {
            request.header("Mcp-Session-Id", session);
        }
        if (version != null) {
            request.header("MCP-Protocol-Version", version);
        }
        if (origin != null) {
            request.header("Origin", origin);
        }
        return client.send(request.build(), HttpResponse.BodyHandlers.ofString());
    }

    private static JsonObject parse(HttpResponse<String> response) {
        return JsonParser.parseString(response.body()).getAsJsonObject();
    }

    private static String initializeRequest() {
        return request(
                1,
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

    private static void setOwnerOnly(Path path) throws IOException {
        try {
            Files.setPosixFilePermissions(
                    path,
                    Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE));
        } catch (UnsupportedOperationException ignored) {
            // The production configuration applies the available platform policy.
        }
    }
}
