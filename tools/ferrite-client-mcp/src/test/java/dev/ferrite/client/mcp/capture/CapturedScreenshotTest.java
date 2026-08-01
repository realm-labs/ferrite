package dev.ferrite.client.mcp.capture;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import org.junit.jupiter.api.Test;

final class CapturedScreenshotTest {
    private static final byte[] PNG = {
        (byte) 0x89, 'P', 'N', 'G', 0x0D, 0x0A, 0x1A, 0x0A, 1
    };

    @Test
    void pngBytesAreDefensivelyCopied() {
        byte[] source = PNG.clone();
        CapturedScreenshot screenshot = new CapturedScreenshot(source, 2, 1, 7, sha256(source));
        source[8] = 9;
        byte[] returned = screenshot.png();
        returned[8] = 8;

        assertArrayEquals(PNG, screenshot.png());
        assertEquals(PNG.length, screenshot.byteLength());
        assertNotEquals(source[8], screenshot.png()[8]);
    }

    @Test
    void invalidGeometryTickAndDigestFailClosed() {
        assertThrows(
                IllegalArgumentException.class,
                () -> new CapturedScreenshot(PNG, 0, 1, 0, sha256(PNG)));
        assertThrows(
                IllegalArgumentException.class,
                () -> new CapturedScreenshot(PNG, 1, 1, -1, sha256(PNG)));
        assertThrows(
                IllegalArgumentException.class,
                () -> new CapturedScreenshot(PNG, 1, 1, 0, "ABC"));
        assertThrows(
                IllegalArgumentException.class,
                () -> new CapturedScreenshot(PNG, 1, 1, 0, "0".repeat(64)));
    }

    private static String sha256(byte[] bytes) {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        } catch (NoSuchAlgorithmException impossible) {
            throw new IllegalStateException(impossible);
        }
    }
}
