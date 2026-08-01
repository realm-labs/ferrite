package dev.ferrite.client.mcp;

import net.fabricmc.api.ClientModInitializer;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/** Entry point for Ferrite's instrumented Minecraft acceptance client. */
public final class FerriteClientMcp implements ClientModInitializer {
    private static final Logger LOGGER = LoggerFactory.getLogger(FerriteClientMcp.class);

    @Override
    public void onInitializeClient() {
        LOGGER.info("Ferrite client MCP instrumentation loaded");
    }
}
