package dev.ferrite.client.mcp.launcher;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

/** Verifies the external locked client without copying it into a run directory. */
final class ArtifactVerifier {
    static final long CLIENT_BYTES = 39_193_383;
    static final String CLIENT_SHA1 = "2dc72797acbc1b63fc16a11c4ac393605f453754";

    private ArtifactVerifier() {}

    static void verifyClient(Path client) throws IOException {
        if (!Files.isRegularFile(client) || Files.size(client) != CLIENT_BYTES) {
            throw new IOException("locked Minecraft 26.2 client size mismatch");
        }
        MessageDigest digest;
        try {
            digest = MessageDigest.getInstance("SHA-1");
        } catch (NoSuchAlgorithmException error) {
            throw new IllegalStateException("SHA-1 is unavailable", error);
        }
        try (InputStream input = Files.newInputStream(client)) {
            byte[] buffer = new byte[64 * 1024];
            for (int read; (read = input.read(buffer)) >= 0; ) {
                digest.update(buffer, 0, read);
            }
        }
        if (!HexFormat.of().formatHex(digest.digest()).equals(CLIENT_SHA1)) {
            throw new IOException("locked Minecraft 26.2 client digest mismatch");
        }
    }
}
