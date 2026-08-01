package dev.ferrite.client.mcp.acceptance;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.IOException;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.Base64;
import java.util.UUID;

/** Minimal authenticated MCP client that records responses but never its bearer secret. */
final class McpClient implements AutoCloseable {
    private static final String PROTOCOL = "2025-11-25";

    private final URI endpoint;
    private final String secret;
    private final String session;
    private final HttpClient client;
    private final EvidenceBundle evidence;
    private int requestId;

    private McpClient(
            URI endpoint,
            String secret,
            String session,
            HttpClient client,
            EvidenceBundle evidence,
            int requestId) {
        this.endpoint = endpoint;
        this.secret = secret;
        this.session = session;
        this.client = client;
        this.evidence = evidence;
        this.requestId = requestId;
    }

    static McpClient initialize(URI endpoint, Path secretFile, EvidenceBundle evidence)
            throws IOException, InterruptedException {
        String secret = Files.readString(secretFile, StandardCharsets.UTF_8).strip();
        if (!secret.matches("[0-9a-f]{64}")) {
            throw new IOException("MCP secret file is malformed");
        }
        HttpClient client = HttpClient.newBuilder()
                .connectTimeout(Duration.ofSeconds(5))
                .build();
        JsonObject params = new JsonObject();
        params.addProperty("protocolVersion", PROTOCOL);
        params.add("capabilities", new JsonObject());
        JsonObject clientInfo = new JsonObject();
        clientInfo.addProperty("name", "ferrite-client-acceptance");
        clientInfo.addProperty("version", "0.1.0");
        params.add("clientInfo", clientInfo);
        JsonObject request = request(1, "initialize", params);
        HttpResponse<String> response = send(client, endpoint, secret, null, request);
        JsonObject body = parseSuccess(response, "initialize");
        evidence.response("initialize", body);
        String session = response.headers()
                .firstValue("Mcp-Session-Id")
                .orElseThrow(() -> new IOException("initialize omitted MCP session ID"));
        McpClient mcp = new McpClient(endpoint, secret, session, client, evidence, 1);
        JsonObject initialized = new JsonObject();
        initialized.addProperty("jsonrpc", "2.0");
        initialized.addProperty("method", "notifications/initialized");
        HttpResponse<String> notification = mcp.send(initialized);
        if (notification.statusCode() != 202 && notification.statusCode() != 200) {
            throw new IOException("initialized notification failed with HTTP " + notification.statusCode());
        }
        return mcp;
    }

    JsonObject call(String operation, String tool, JsonObject arguments)
            throws IOException, InterruptedException {
        requestId++;
        JsonObject params = new JsonObject();
        params.addProperty("name", tool);
        params.add("arguments", arguments);
        JsonObject response = parseSuccess(
                send(request(requestId, "tools/call", params)), operation);
        evidence.response(operation, response);
        JsonObject result = response.getAsJsonObject("result");
        if (result.has("isError") && result.get("isError").getAsBoolean()) {
            throw new IOException(tool + " failed: " + result);
        }
        return result.getAsJsonObject("structuredContent");
    }

    JsonObject submitAndAwait(String operation, String tool, JsonObject arguments, int durationTicks)
            throws IOException, InterruptedException {
        String actionId = operation + "-" + UUID.randomUUID();
        arguments.addProperty("actionId", actionId);
        JsonObject queued = call(operation + "-queued", tool, arguments);
        long acceptedTick = queued.get("acceptedTick").getAsLong();
        JsonObject wait = new JsonObject();
        wait.addProperty("afterClientTick", acceptedTick + Math.max(1, durationTicks));
        wait.addProperty("maxTicks", Math.max(20, durationTicks + 20));
        call(operation + "-tick-fence", "wait_for_state", wait);
        JsonObject statusArgs = new JsonObject();
        statusArgs.addProperty("actionId", actionId);
        JsonObject status = call(operation + "-status", "action_status", statusArgs);
        String state = status.get("state").getAsString();
        if (!state.equals("Satisfied")) {
            throw new IOException(tool + " did not satisfy: " + state);
        }
        return status;
    }

    void screenshot(String operation, String fileName) throws IOException, InterruptedException {
        requestId++;
        JsonObject params = new JsonObject();
        params.addProperty("name", "take_screenshot");
        params.add("arguments", new JsonObject());
        JsonObject response = parseSuccess(
                send(request(requestId, "tools/call", params)), operation);
        evidence.response(operation, response);
        JsonObject result = response.getAsJsonObject("result");
        if (result.get("isError").getAsBoolean()) {
            throw new IOException("screenshot failed");
        }
        JsonArray content = result.getAsJsonArray("content");
        for (JsonElement element : content) {
            JsonObject item = element.getAsJsonObject();
            if (item.has("type") && "image".equals(item.get("type").getAsString())) {
                byte[] png = Base64.getDecoder().decode(item.get("data").getAsString());
                Files.write(evidence.root().resolve(fileName), png);
                return;
            }
        }
        throw new IOException("screenshot response omitted image content");
    }

    @Override
    public void close() {
        try {
            HttpRequest request = HttpRequest.newBuilder(endpoint)
                    .timeout(Duration.ofSeconds(5))
                    .header("Authorization", "Bearer " + secret)
                    .header("Mcp-Session-Id", session)
                    .header("MCP-Protocol-Version", PROTOCOL)
                    .header("Accept", "application/json, text/event-stream")
                    .DELETE()
                    .build();
            client.send(request, HttpResponse.BodyHandlers.discarding());
        } catch (IOException error) {
            System.err.println("MCP session shutdown failed");
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
        }
    }

    private HttpResponse<String> send(JsonObject body) throws IOException, InterruptedException {
        return send(client, endpoint, secret, session, body);
    }

    private static HttpResponse<String> send(
            HttpClient client, URI endpoint, String secret, String session, JsonObject body)
            throws IOException, InterruptedException {
        HttpRequest.Builder builder = HttpRequest.newBuilder(endpoint)
                .timeout(Duration.ofSeconds(35))
                .header("Authorization", "Bearer " + secret)
                .header("Accept", "application/json, text/event-stream")
                .header("Content-Type", "application/json")
                .POST(HttpRequest.BodyPublishers.ofString(body.toString(), StandardCharsets.UTF_8));
        if (session != null) {
            builder.header("Mcp-Session-Id", session)
                    .header("MCP-Protocol-Version", PROTOCOL);
        }
        return client.send(builder.build(), HttpResponse.BodyHandlers.ofString(StandardCharsets.UTF_8));
    }

    private static JsonObject parseSuccess(HttpResponse<String> response, String operation)
            throws IOException {
        if (response.statusCode() != 200) {
            throw new IOException(operation + " failed with HTTP " + response.statusCode());
        }
        JsonObject body = JsonParser.parseString(response.body()).getAsJsonObject();
        if (body.has("error")) {
            throw new IOException(operation + " returned JSON-RPC error: " + body.get("error"));
        }
        return body;
    }

    private static JsonObject request(int id, String method, JsonObject params) {
        JsonObject request = new JsonObject();
        request.addProperty("jsonrpc", "2.0");
        request.addProperty("id", id);
        request.addProperty("method", method);
        request.add("params", params);
        return request;
    }
}
