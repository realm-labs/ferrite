package dev.ferrite.client.mcp.protocol;

import java.util.concurrent.atomic.AtomicBoolean;

/** Lifecycle state negotiated by one bounded Streamable HTTP session. */
final class McpSession {
    private final String id;
    private final String protocolVersion;
    private final AtomicBoolean initialized = new AtomicBoolean();

    McpSession(String id, String protocolVersion) {
        this.id = id;
        this.protocolVersion = protocolVersion;
    }

    String id() {
        return id;
    }

    String protocolVersion() {
        return protocolVersion;
    }

    boolean initialized() {
        return initialized.get();
    }

    void markInitialized() {
        initialized.set(true);
    }
}
