package dev.ferrite.client.mcp.protocol;

import java.util.Optional;
import java.util.UUID;

/** Single-client session registry that keeps client control ownership unambiguous. */
final class McpSessionRegistry {
    private McpSession active;

    synchronized Optional<McpSession> create(String protocolVersion) {
        if (active != null) {
            return Optional.empty();
        }
        active = new McpSession(UUID.randomUUID().toString(), protocolVersion);
        return Optional.of(active);
    }

    synchronized Optional<McpSession> find(String id) {
        if (active == null || id == null || !active.id().equals(id)) {
            return Optional.empty();
        }
        return Optional.of(active);
    }

    synchronized boolean remove(String id) {
        if (active == null || id == null || !active.id().equals(id)) {
            return false;
        }
        active = null;
        return true;
    }

    synchronized void clear() {
        active = null;
    }
}
