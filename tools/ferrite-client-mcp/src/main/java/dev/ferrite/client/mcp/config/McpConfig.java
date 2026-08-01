package dev.ferrite.client.mcp.config;

import java.io.IOException;
import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermission;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

/** Validated process configuration for the loopback MCP endpoint. */
public record McpConfig(
        InetAddress bindAddress,
        int port,
        Path secretFile,
        Optional<Path> readyFile,
        int maxBodyBytes,
        int workerThreads,
        int queueCapacity) {
    public static final String SECRET_FILE_ENV = "FERRITE_CLIENT_MCP_SECRET_FILE";
    public static final String READY_FILE_ENV = "FERRITE_CLIENT_MCP_READY_FILE";
    public static final String PORT_ENV = "FERRITE_CLIENT_MCP_PORT";

    private static final int DEFAULT_MAX_BODY_BYTES = 1_048_576;
    private static final int DEFAULT_WORKER_THREADS = 2;
    private static final int DEFAULT_QUEUE_CAPACITY = 32;
    private static final int MINIMUM_SECRET_BYTES = 32;
    private static final int MAXIMUM_SECRET_BYTES = 256;
    private static final Set<PosixFilePermission> UNSAFE_SECRET_PERMISSIONS = Set.of(
            PosixFilePermission.GROUP_READ,
            PosixFilePermission.GROUP_WRITE,
            PosixFilePermission.GROUP_EXECUTE,
            PosixFilePermission.OTHERS_READ,
            PosixFilePermission.OTHERS_WRITE,
            PosixFilePermission.OTHERS_EXECUTE);

    public McpConfig {
        if (bindAddress == null || !bindAddress.isLoopbackAddress()) {
            throw new IllegalArgumentException("MCP bind address must be loopback");
        }
        if (port < 0 || port > 65_535) {
            throw new IllegalArgumentException("MCP port must be between 0 and 65535");
        }
        if (secretFile == null || !secretFile.isAbsolute()) {
            throw new IllegalArgumentException("MCP secret file path must be absolute");
        }
        readyFile = readyFile == null ? Optional.empty() : readyFile;
        if (readyFile.isPresent() && !readyFile.orElseThrow().isAbsolute()) {
            throw new IllegalArgumentException("MCP ready file path must be absolute");
        }
        if (readyFile.isPresent() && readyFile.orElseThrow().equals(secretFile)) {
            throw new IllegalArgumentException("MCP ready file must differ from the secret file");
        }
        if (maxBodyBytes < 1 || workerThreads < 1 || queueCapacity < 1) {
            throw new IllegalArgumentException("MCP resource bounds must be positive");
        }
    }

    public static Optional<McpConfig> fromEnvironment(Map<String, String> environment) {
        String configuredSecret = environment.get(SECRET_FILE_ENV);
        if (configuredSecret == null || configuredSecret.isBlank()) {
            return Optional.empty();
        }

        int configuredPort = parsePort(environment.get(PORT_ENV));
        Optional<Path> configuredReadyFile = Optional.ofNullable(environment.get(READY_FILE_ENV))
                .filter(value -> !value.isBlank())
                .map(value -> Path.of(value).toAbsolutePath().normalize());
        return Optional.of(new McpConfig(
                loopback(),
                configuredPort,
                Path.of(configuredSecret).toAbsolutePath().normalize(),
                configuredReadyFile,
                DEFAULT_MAX_BODY_BYTES,
                DEFAULT_WORKER_THREADS,
                DEFAULT_QUEUE_CAPACITY));
    }

    public byte[] loadSecret() throws IOException {
        if (Files.isSymbolicLink(secretFile)
                || !Files.isRegularFile(secretFile, LinkOption.NOFOLLOW_LINKS)) {
            throw new IOException("MCP secret path must name a regular non-symlink file");
        }
        rejectUnsafePermissions(secretFile);

        String value = Files.readString(secretFile, StandardCharsets.UTF_8).strip();
        byte[] secret = value.getBytes(StandardCharsets.UTF_8);
        if (secret.length < MINIMUM_SECRET_BYTES || secret.length > MAXIMUM_SECRET_BYTES) {
            throw new IOException("MCP secret must contain between 32 and 256 UTF-8 bytes");
        }
        if (value.chars().anyMatch(Character::isWhitespace)) {
            throw new IOException("MCP secret must not contain whitespace");
        }
        return secret;
    }

    private static void rejectUnsafePermissions(Path secretFile) throws IOException {
        try {
            Set<PosixFilePermission> permissions = Files.getPosixFilePermissions(secretFile);
            if (permissions.stream().anyMatch(UNSAFE_SECRET_PERMISSIONS::contains)) {
                throw new IOException("MCP secret file must not be accessible by group or others");
            }
        } catch (UnsupportedOperationException ignored) {
            // Non-POSIX platforms rely on the launcher's owner-only file creation policy.
        }
    }

    private static int parsePort(String value) {
        if (value == null || value.isBlank()) {
            return 0;
        }
        try {
            int parsed = Integer.parseInt(value);
            if (parsed < 0 || parsed > 65_535) {
                throw new IllegalArgumentException("MCP port must be between 0 and 65535");
            }
            return parsed;
        } catch (NumberFormatException error) {
            throw new IllegalArgumentException("MCP port must be an integer", error);
        }
    }

    private static InetAddress loopback() {
        try {
            return InetAddress.getByName("127.0.0.1");
        } catch (UnknownHostException impossible) {
            throw new IllegalStateException("IPv4 loopback address is unavailable", impossible);
        }
    }
}
