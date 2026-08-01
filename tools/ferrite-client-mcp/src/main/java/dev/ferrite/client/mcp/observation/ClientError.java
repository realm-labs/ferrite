package dev.ferrite.client.mcp.observation;

/** Redacted client-side failure or connection-loss event. */
public record ClientError(long clientTick, String category, String message) {}
