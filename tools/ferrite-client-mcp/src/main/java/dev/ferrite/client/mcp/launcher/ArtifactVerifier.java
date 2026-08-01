package dev.ferrite.client.mcp.launcher;

import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;

/** Verifies the external locked client without copying it into a run directory. */
public final class ArtifactVerifier {
    static final long CLIENT_BYTES = 39_193_383;
    static final String CLIENT_SHA1 = "2dc72797acbc1b63fc16a11c4ac393605f453754";
    private static final long SERVER_BYTES = 60_894_273;
    private static final String SERVER_SHA1 = "823e2250d24b3ddac457a60c92a6a941943fcd6a";

    private ArtifactVerifier() {}

    static void verifyClient(Path client) throws IOException {
        verify(client, CLIENT_BYTES, CLIENT_SHA1, "client");
    }

    public static void verifyServer(Path server) throws IOException {
        verify(server, SERVER_BYTES, SERVER_SHA1, "server");
    }

    private static void verify(Path artifact, long expectedBytes, String expectedSha1, String label)
            throws IOException {
        if (!Files.isRegularFile(artifact) || Files.size(artifact) != expectedBytes) {
            throw new IOException("locked Minecraft 26.2 " + label + " size mismatch");
        }
        MessageDigest digest;
        try {
            digest = MessageDigest.getInstance("SHA-1");
        } catch (NoSuchAlgorithmException error) {
            throw new IllegalStateException("SHA-1 is unavailable", error);
        }
        try (InputStream input = Files.newInputStream(artifact)) {
            byte[] buffer = new byte[64 * 1024];
            for (int read; (read = input.read(buffer)) >= 0; ) {
                digest.update(buffer, 0, read);
            }
        }
        if (!HexFormat.of().formatHex(digest.digest()).equals(expectedSha1)) {
            throw new IOException("locked Minecraft 26.2 " + label + " digest mismatch");
        }
    }
}
