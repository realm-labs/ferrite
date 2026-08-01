package dev.ferrite.client.mcp.capture;

import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.Locale;

/** Bounded immutable PNG plus the client tick and framebuffer geometry it represents. */
public record CapturedScreenshot(
        byte[] png, int width, int height, long clientTick, String sha256) {
    private static final long MAXIMUM_PIXELS = 16_777_216;
    private static final int MAXIMUM_PNG_BYTES = 16 * 1024 * 1024;
    private static final byte[] PNG_SIGNATURE = {
        (byte) 0x89, 'P', 'N', 'G', 0x0D, 0x0A, 0x1A, 0x0A
    };

    public CapturedScreenshot {
        png = png.clone();
        if (width < 1
                || height < 1
                || (long) width * height > MAXIMUM_PIXELS
                || clientTick < 0) {
            throw new IllegalArgumentException("screenshot dimensions and client tick are invalid");
        }
        if (png.length < PNG_SIGNATURE.length || png.length > MAXIMUM_PNG_BYTES) {
            throw new IllegalArgumentException("screenshot PNG length is invalid");
        }
        for (int index = 0; index < PNG_SIGNATURE.length; index++) {
            if (png[index] != PNG_SIGNATURE[index]) {
                throw new IllegalArgumentException("screenshot does not have a PNG signature");
            }
        }
        if (sha256 == null
                || sha256.length() != 64
                || !sha256.equals(sha256.toLowerCase(Locale.ROOT))) {
            throw new IllegalArgumentException("screenshot SHA-256 must be canonical hex");
        }
        HexFormat.of().parseHex(sha256);
        if (!sha256.equals(computeSha256(png))) {
            throw new IllegalArgumentException("screenshot SHA-256 does not match PNG bytes");
        }
    }

    @Override
    public byte[] png() {
        return png.clone();
    }

    public int byteLength() {
        return png.length;
    }

    private static String computeSha256(byte[] bytes) {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        } catch (NoSuchAlgorithmException impossible) {
            throw new IllegalStateException("SHA-256 is unavailable", impossible);
        }
    }
}
