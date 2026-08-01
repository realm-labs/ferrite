package dev.ferrite.client.mcp.transport;

import com.sun.net.httpserver.HttpExchange;
import com.sun.net.httpserver.HttpServer;
import dev.ferrite.client.mcp.config.McpConfig;
import dev.ferrite.client.mcp.protocol.McpProtocol;
import dev.ferrite.client.mcp.protocol.ProtocolReply;
import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Optional;
import java.util.concurrent.ArrayBlockingQueue;
import java.util.concurrent.ThreadPoolExecutor;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicInteger;

/** Bounded loopback-only Streamable HTTP server without an SSE stream. */
public final class McpHttpServer implements AutoCloseable {
    private final McpConfig config;
    private final McpProtocol protocol;
    private final byte[] secret;
    private final ThreadPoolExecutor executor;
    private final Optional<ReadyFile> readyFile;
    private HttpServer server;

    public McpHttpServer(McpConfig config, McpProtocol protocol) throws IOException {
        this.config = config;
        this.protocol = protocol;
        this.secret = config.loadSecret();
        this.executor = new ThreadPoolExecutor(
                config.workerThreads(),
                config.workerThreads(),
                0,
                TimeUnit.MILLISECONDS,
                new ArrayBlockingQueue<>(config.queueCapacity()),
                new McpThreadFactory(),
                new ThreadPoolExecutor.AbortPolicy());
        this.readyFile = config.readyFile().map(ReadyFile::new);
    }

    public synchronized int start() throws IOException {
        if (server != null) {
            throw new IllegalStateException("MCP HTTP server is already started");
        }
        HttpServer created = HttpServer.create(
                new InetSocketAddress(config.bindAddress(), config.port()), 0);
        created.createContext("/mcp", this::handle);
        created.setExecutor(executor);
        created.start();
        server = created;
        int port = created.getAddress().getPort();
        try {
            if (readyFile.isPresent()) {
                readyFile.orElseThrow().publish(port);
            }
        } catch (IOException error) {
            close();
            throw error;
        }
        return port;
    }

    public synchronized int port() {
        if (server == null) {
            throw new IllegalStateException("MCP HTTP server is not started");
        }
        return server.getAddress().getPort();
    }

    @Override
    public synchronized void close() throws IOException {
        IOException closeFailure = null;
        if (server != null) {
            server.stop(0);
            server = null;
        }
        executor.shutdownNow();
        protocol.close();
        if (readyFile.isPresent()) {
            try {
                readyFile.orElseThrow().close();
            } catch (IOException error) {
                closeFailure = error;
            }
        }
        Arrays.fill(secret, (byte) 0);
        if (closeFailure != null) {
            throw closeFailure;
        }
    }

    private void handle(HttpExchange exchange) throws IOException {
        try (exchange) {
            addCommonHeaders(exchange);
            if (!exchange.getRemoteAddress().getAddress().isLoopbackAddress()) {
                sendEmpty(exchange, 403);
                return;
            }
            if (!HttpSecurity.validOrigin(exchange.getRequestHeaders())) {
                sendEmpty(exchange, 403);
                return;
            }
            if (!HttpSecurity.authorized(exchange.getRequestHeaders(), secret)) {
                exchange.getResponseHeaders().set("WWW-Authenticate", "Bearer");
                sendEmpty(exchange, 401);
                return;
            }

            switch (exchange.getRequestMethod()) {
                case "POST" -> handlePost(exchange);
                case "DELETE" -> handleDelete(exchange);
                default -> {
                    exchange.getResponseHeaders().set("Allow", "POST, DELETE");
                    sendEmpty(exchange, 405);
                }
            }
        }
    }

    private void handlePost(HttpExchange exchange) throws IOException {
        if (!HttpSecurity.isJsonRequest(exchange.getRequestHeaders())) {
            sendEmpty(exchange, 415);
            return;
        }
        if (!HttpSecurity.acceptsMcpResponse(exchange.getRequestHeaders())) {
            sendEmpty(exchange, 406);
            return;
        }
        byte[] body = exchange.getRequestBody().readNBytes(config.maxBodyBytes() + 1);
        if (body.length > config.maxBodyBytes()) {
            sendEmpty(exchange, 413);
            return;
        }

        ProtocolReply reply = protocol.handle(
                new String(body, StandardCharsets.UTF_8),
                exchange.getRequestHeaders().getFirst("Mcp-Session-Id"),
                exchange.getRequestHeaders().getFirst("MCP-Protocol-Version"));
        reply.sessionId().ifPresent(id -> exchange.getResponseHeaders().set("Mcp-Session-Id", id));
        if (reply.body().isEmpty()) {
            sendEmpty(exchange, reply.status());
            return;
        }
        byte[] response = protocol.toJson(reply.body().orElseThrow()).getBytes(StandardCharsets.UTF_8);
        exchange.getResponseHeaders().set("Content-Type", "application/json; charset=utf-8");
        exchange.sendResponseHeaders(reply.status(), response.length);
        exchange.getResponseBody().write(response);
    }

    private void handleDelete(HttpExchange exchange) throws IOException {
        String sessionId = exchange.getRequestHeaders().getFirst("Mcp-Session-Id");
        if (protocol.deleteSession(sessionId)) {
            sendEmpty(exchange, 204);
        } else {
            sendEmpty(exchange, 404);
        }
    }

    private static void addCommonHeaders(HttpExchange exchange) {
        exchange.getResponseHeaders().set("Cache-Control", "no-store");
        exchange.getResponseHeaders().set("X-Content-Type-Options", "nosniff");
    }

    private static void sendEmpty(HttpExchange exchange, int status) throws IOException {
        exchange.sendResponseHeaders(status, -1);
    }

    private static final class McpThreadFactory implements java.util.concurrent.ThreadFactory {
        private final AtomicInteger sequence = new AtomicInteger();

        @Override
        public Thread newThread(Runnable runnable) {
            Thread thread = new Thread(
                    runnable, "ferrite-client-mcp-http-" + sequence.incrementAndGet());
            thread.setDaemon(true);
            return thread;
        }
    }
}
