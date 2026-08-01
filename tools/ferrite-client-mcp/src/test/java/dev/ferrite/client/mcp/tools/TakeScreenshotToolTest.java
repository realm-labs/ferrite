package dev.ferrite.client.mcp.tools;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.capture.CapturedScreenshot;
import dev.ferrite.client.mcp.capture.ScreenshotCapture;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Base64;
import java.util.HexFormat;
import java.util.concurrent.CompletableFuture;
import org.junit.jupiter.api.Test;

final class TakeScreenshotToolTest {
    private static final byte[] PNG = {
        (byte) 0x89, 'P', 'N', 'G', 0x0D, 0x0A, 0x1A, 0x0A
    };

    @Test
    void successfulCaptureReturnsMetadataTextAndImageContent() {
        ScreenshotCapture capture = completedCapture(
                new CapturedScreenshot(PNG, 320, 240, 42, sha256(PNG)));
        McpToolResult result = new TakeScreenshotTool(capture).call(
                new JsonObject(), new ToolContext("2025-11-25"));

        assertFalse(result.error());
        assertEquals(320, result.structuredContent().get("width").getAsInt());
        assertEquals(42, result.structuredContent().get("clientTick").getAsLong());
        assertEquals(2, result.content().size());
        JsonObject image = result.content().get(1).getAsJsonObject();
        assertEquals("image", image.get("type").getAsString());
        assertEquals("image/png", image.get("mimeType").getAsString());
        assertEquals(Base64.getEncoder().encodeToString(PNG), image.get("data").getAsString());
    }

    @Test
    void failedBusyAndTimedOutCaptureReturnToolErrors() {
        ScreenshotCapture failed = new ScreenshotCapture() {
            @Override
            public CompletableFuture<CapturedScreenshot> request() {
                return CompletableFuture.failedFuture(
                        new IllegalStateException("another framebuffer capture is already pending"));
            }

            @Override
            public void close() {}
        };
        assertTrue(new TakeScreenshotTool(failed)
                .call(new JsonObject(), new ToolContext("2025-11-25"))
                .error());

        CompletableFuture<CapturedScreenshot> pending = new CompletableFuture<>();
        ScreenshotCapture stalled = new ScreenshotCapture() {
            @Override
            public CompletableFuture<CapturedScreenshot> request() {
                return pending;
            }

            @Override
            public void close() {}
        };
        McpToolResult timeout = new TakeScreenshotTool(stalled, 10)
                .call(new JsonObject(), new ToolContext("2025-11-25"));
        assertTrue(timeout.error());
        assertTrue(pending.isCancelled());

        JsonObject arguments = new JsonObject();
        arguments.addProperty("path", "forbidden.png");
        assertTrue(new TakeScreenshotTool(failed)
                .call(arguments, new ToolContext("2025-11-25"))
                .error());
    }

    private static ScreenshotCapture completedCapture(CapturedScreenshot screenshot) {
        return new ScreenshotCapture() {
            @Override
            public CompletableFuture<CapturedScreenshot> request() {
                return CompletableFuture.completedFuture(screenshot);
            }

            @Override
            public void close() {}
        };
    }

    private static String sha256(byte[] bytes) {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        } catch (NoSuchAlgorithmException impossible) {
            throw new IllegalStateException(impossible);
        }
    }
}
