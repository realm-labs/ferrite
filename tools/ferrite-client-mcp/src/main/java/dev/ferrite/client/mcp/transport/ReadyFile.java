package dev.ferrite.client.mcp.transport;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.attribute.PosixFilePermission;
import java.util.Set;

/** Atomic, secret-free endpoint discovery file owned by the isolated launcher run. */
final class ReadyFile implements AutoCloseable {
    private static final Set<PosixFilePermission> OWNER_ONLY = Set.of(
            PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE);

    private final Path path;

    ReadyFile(Path path) {
        this.path = path;
    }

    void publish(int port) throws IOException {
        Path parent = path.getParent();
        if (parent == null) {
            throw new IOException("MCP ready file must have a parent directory");
        }
        Files.createDirectories(parent);
        Path temporary = Files.createTempFile(parent, ".ferrite-client-mcp-", ".tmp");
        try {
            setOwnerOnly(temporary);
            String body = "{\"endpoint\":\"http://127.0.0.1:" + port + "/mcp\"}\n";
            Files.writeString(temporary, body, StandardCharsets.UTF_8);
            moveAtomically(temporary, path);
            setOwnerOnly(path);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    @Override
    public void close() throws IOException {
        Files.deleteIfExists(path);
    }

    private static void moveAtomically(Path source, Path target) throws IOException {
        try {
            Files.move(
                    source,
                    target,
                    StandardCopyOption.ATOMIC_MOVE,
                    StandardCopyOption.REPLACE_EXISTING);
        } catch (AtomicMoveNotSupportedException unsupported) {
            Files.move(source, target, StandardCopyOption.REPLACE_EXISTING);
        }
    }

    private static void setOwnerOnly(Path path) throws IOException {
        try {
            Files.setPosixFilePermissions(path, OWNER_ONLY);
        } catch (UnsupportedOperationException ignored) {
            // The launcher applies the closest platform-specific owner-only policy.
        }
    }
}
