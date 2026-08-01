package dev.ferrite.client.mcp;

import dev.ferrite.client.mcp.capture.MinecraftScreenshotCapture;
import dev.ferrite.client.mcp.capture.ScreenshotCapture;
import dev.ferrite.client.mcp.config.McpConfig;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import dev.ferrite.client.mcp.observation.MinecraftObservationCollector;
import dev.ferrite.client.mcp.protocol.McpProtocol;
import dev.ferrite.client.mcp.tools.ToolRegistry;
import dev.ferrite.client.mcp.transport.McpHttpServer;
import java.io.IOException;
import java.util.Optional;
import net.fabricmc.api.ClientModInitializer;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientLifecycleEvents;
import net.fabricmc.fabric.api.client.event.lifecycle.v1.ClientTickEvents;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Entry point for Ferrite's instrumented Minecraft acceptance client. */
public final class FerriteClientMcp implements ClientModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger(FerriteClientMcp.class);
    private static final String VERSION = "0.1.0-SNAPSHOT";

    private McpHttpServer server;
    private ScreenshotCapture screenshotCapture;

    @Override
    public void onInitializeClient() {
        Optional<McpConfig> configured = McpConfig.fromEnvironment(System.getenv());
        if (configured.isEmpty()) {
            LOGGER.info("Ferrite client MCP instrumentation is disabled");
            return;
        }

        try {
            ClientObservationStore observations = new ClientObservationStore();
            MinecraftObservationCollector collector = new MinecraftObservationCollector(observations);
            ClientTickEvents.END_CLIENT_TICK.register(collector::capture);
            screenshotCapture = new MinecraftScreenshotCapture(observations);
            McpProtocol protocol = new McpProtocol(
                    ToolRegistry.forObservations(observations, screenshotCapture), VERSION);
            server = new McpHttpServer(configured.orElseThrow(), protocol);
            int port = server.start();
            LOGGER.info("Ferrite client MCP instrumentation listening on loopback port {}", port);
            ClientLifecycleEvents.CLIENT_STOPPING.register(client -> closeServer());
            Runtime.getRuntime()
                    .addShutdownHook(
                            new Thread(this::closeServer, "ferrite-client-mcp-shutdown"));
        } catch (IOException | RuntimeException error) {
            closeServer();
            throw new IllegalStateException("failed to start Ferrite client MCP instrumentation", error);
        }
    }

    private synchronized void closeServer() {
        McpHttpServer activeServer = server;
        server = null;
        if (activeServer != null) {
            try {
                activeServer.close();
            } catch (IOException error) {
                LOGGER.error("Failed to cleanly stop Ferrite client MCP instrumentation", error);
            }
        }

        ScreenshotCapture activeCapture = screenshotCapture;
        screenshotCapture = null;
        if (activeCapture != null) {
            activeCapture.close();
        }
    }
}
