package dev.ferrite.client.mcp.launcher;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.attribute.PosixFilePermission;
import java.security.SecureRandom;
import java.util.HexFormat;
import java.util.Set;
import java.util.UUID;

/** Owner-only runtime files that never reuse a normal Minecraft installation. */
final class IsolatedClientRun {
    private static final Set<PosixFilePermission> OWNER_ONLY =
            Set.of(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE);

    private final Path root;
    private final Path gameDirectory;
    private final Path secretFile;
    private final Path readyFile;
    private final Path clientLog;

    private IsolatedClientRun(Path root) {
        this.root = root;
        gameDirectory = root.resolve("game");
        secretFile = root.resolve("mcp.secret");
        readyFile = root.resolve("mcp-ready.json");
        clientLog = root.resolve("client.log");
    }

    static IsolatedClientRun create(Path runRoot) throws IOException {
        Files.createDirectories(runRoot);
        IsolatedClientRun run = new IsolatedClientRun(
                runRoot.resolve("run-" + UUID.randomUUID()));
        Files.createDirectory(run.root);
        Files.createDirectory(run.gameDirectory);
        byte[] secret = new byte[32];
        new SecureRandom().nextBytes(secret);
        Files.writeString(
                run.secretFile,
                HexFormat.of().formatHex(secret) + System.lineSeparator(),
                StandardCharsets.UTF_8);
        setOwnerOnly(run.secretFile);
        Files.writeString(
                run.gameDirectory.resolve("options.txt"),
                "version:4903\nskipMultiplayerWarning:true\njoinedFirstServer:true\nonboardAccessibility:false\n",
                StandardCharsets.UTF_8);
        return run;
    }

    Path root() {
        return root;
    }

    Path gameDirectory() {
        return gameDirectory;
    }

    Path secretFile() {
        return secretFile;
    }

    Path readyFile() {
        return readyFile;
    }

    Path clientLog() {
        return clientLog;
    }

    void delete() throws IOException {
        if (!Files.exists(root)) {
            return;
        }
        try (var paths = Files.walk(root)) {
            for (Path path : paths.sorted(java.util.Comparator.reverseOrder()).toList()) {
                Files.deleteIfExists(path);
            }
        }
    }

    private static void setOwnerOnly(Path file) throws IOException {
        try {
            Files.setPosixFilePermissions(file, OWNER_ONLY);
        } catch (UnsupportedOperationException error) {
            if (!file.toFile().setReadable(false, false)
                    || !file.toFile().setReadable(true, true)
                    || !file.toFile().setWritable(false, false)
                    || !file.toFile().setWritable(true, true)) {
                throw new IOException("failed to restrict secret file permissions", error);
            }
        }
    }
}
