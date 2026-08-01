package dev.ferrite.client.mcp.tools;

import com.google.gson.JsonObject;
import dev.ferrite.client.mcp.capture.CapturedScreenshot;
import dev.ferrite.client.mcp.capture.ScreenshotCapture;
import dev.ferrite.client.mcp.observation.SensitiveText;
import java.util.Base64;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

/** Requests one real framebuffer capture and returns MCP image content. */
public final class TakeScreenshotTool implements McpTool {
    private static final long DEFAULT_TIMEOUT_MILLIS = 10_000;

    private final ScreenshotCapture capture;
    private final long timeoutMillis;

    public TakeScreenshotTool(ScreenshotCapture capture) {
        this(capture, DEFAULT_TIMEOUT_MILLIS);
    }

    TakeScreenshotTool(ScreenshotCapture capture, long timeoutMillis) {
        if (timeoutMillis < 1) {
            throw new IllegalArgumentException("screenshot timeout must be positive");
        }
        this.capture = capture;
        this.timeoutMillis = timeoutMillis;
    }

    @Override
    public String name() {
        return "take_screenshot";
    }

    @Override
    public JsonObject definition() {
        return ToolSchemas.noArguments(
                name(),
                "Take framebuffer screenshot",
                "Capture the current rendered Minecraft framebuffer as bounded PNG image content.");
    }

    @Override
    public McpToolResult call(JsonObject arguments, ToolContext context) {
        if (!arguments.isEmpty()) {
            return ToolSchemas.rejected("take_screenshot does not accept arguments");
        }

        CompletableFuture<CapturedScreenshot> future;
        try {
            future = capture.request();
        } catch (RuntimeException error) {
            return ToolSchemas.rejected("framebuffer capture failed safely");
        }
        try {
            CapturedScreenshot screenshot = future.get(timeoutMillis, TimeUnit.MILLISECONDS);
            JsonObject metadata = new JsonObject();
            metadata.addProperty("state", "Satisfied");
            metadata.addProperty("mimeType", "image/png");
            metadata.addProperty("width", screenshot.width());
            metadata.addProperty("height", screenshot.height());
            metadata.addProperty("clientTick", screenshot.clientTick());
            metadata.addProperty("byteLength", screenshot.byteLength());
            metadata.addProperty("sha256", screenshot.sha256());
            return McpToolResult.image(
                    metadata,
                    "Minecraft framebuffer captured",
                    Base64.getEncoder().encodeToString(screenshot.png()));
        } catch (TimeoutException error) {
            future.cancel(false);
            return ToolSchemas.failure(
                    "TimedOut",
                    "framebuffer capture timed out after " + timeoutMillis + " milliseconds");
        } catch (InterruptedException error) {
            future.cancel(false);
            Thread.currentThread().interrupt();
            return ToolSchemas.failure("Cancelled", "framebuffer capture was interrupted");
        } catch (ExecutionException error) {
            Throwable cause = error.getCause() == null ? error : error.getCause();
            return ToolSchemas.rejected(
                    "framebuffer capture failed: " + SensitiveText.redact(cause.getMessage()));
        }
    }
}
