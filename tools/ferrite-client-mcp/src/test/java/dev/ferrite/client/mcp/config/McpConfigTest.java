package dev.ferrite.client.mcp.config;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermission;
import java.util.Map;
import java.util.Set;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class McpConfigTest {
    private static final String SECRET = "0123456789abcdef0123456789abcdef0123456789abcdef";

    @TempDir Path temporaryDirectory;

    @Test
    void absentSecretDisablesInstrumentation() {
        assertTrue(McpConfig.fromEnvironment(Map.of()).isEmpty());
        assertTrue(McpConfig.fromEnvironment(Map.of(McpConfig.SECRET_FILE_ENV, " ")).isEmpty());
    }

    @Test
    void environmentUsesLoopbackAndNormalizesPaths() throws IOException {
        Path secret = createSecret(SECRET);
        Path ready = temporaryDirectory.resolve("state/ready.json");
        McpConfig config = McpConfig.fromEnvironment(Map.of(
                        McpConfig.SECRET_FILE_ENV,
                        secret.toString(),
                        McpConfig.READY_FILE_ENV,
                        ready.toString(),
                        McpConfig.PORT_ENV,
                        "25580"))
                .orElseThrow();

        assertTrue(config.bindAddress().isLoopbackAddress());
        assertTrue(config.secretFile().isAbsolute());
        assertTrue(config.readyFile().orElseThrow().isAbsolute());
        assertArrayEquals(SECRET.getBytes(StandardCharsets.UTF_8), config.loadSecret());
    }

    @Test
    void malformedPortsAndSecretsFailClosed() throws IOException {
        assertThrows(
                IllegalArgumentException.class,
                () -> McpConfig.fromEnvironment(Map.of(
                        McpConfig.SECRET_FILE_ENV, "/tmp/secret", McpConfig.PORT_ENV, "65536")));
        assertThrows(
                IllegalArgumentException.class,
                () -> McpConfig.fromEnvironment(Map.of(
                        McpConfig.SECRET_FILE_ENV, "/tmp/secret", McpConfig.PORT_ENV, "nope")));

        Path shortSecret = createSecret("too-short");
        McpConfig config = McpConfig.fromEnvironment(
                        Map.of(McpConfig.SECRET_FILE_ENV, shortSecret.toString()))
                .orElseThrow();
        assertThrows(IOException.class, config::loadSecret);

        Path whitespaceSecret = createSecret(SECRET + " bad");
        McpConfig whitespaceConfig = McpConfig.fromEnvironment(
                        Map.of(McpConfig.SECRET_FILE_ENV, whitespaceSecret.toString()))
                .orElseThrow();
        assertThrows(IOException.class, whitespaceConfig::loadSecret);
    }

    @Test
    void groupReadableSecretIsRejectedOnPosixFileSystems() throws IOException {
        Path secret = createSecret(SECRET);
        try {
            Files.setPosixFilePermissions(
                    secret,
                    Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.GROUP_READ));
        } catch (UnsupportedOperationException unsupported) {
            assertFalse(Files.getFileStore(secret).supportsFileAttributeView("posix"));
            return;
        }

        McpConfig config = McpConfig.fromEnvironment(
                        Map.of(McpConfig.SECRET_FILE_ENV, secret.toString()))
                .orElseThrow();
        assertThrows(IOException.class, config::loadSecret);
    }

    private Path createSecret(String value) throws IOException {
        Path secret = temporaryDirectory.resolve("secret-" + Math.abs(value.hashCode()));
        Files.writeString(secret, value, StandardCharsets.UTF_8);
        try {
            Files.setPosixFilePermissions(
                    secret,
                    Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE));
        } catch (UnsupportedOperationException ignored) {
            // The production loader applies the platform-specific check available to it.
        }
        return secret;
    }
}
