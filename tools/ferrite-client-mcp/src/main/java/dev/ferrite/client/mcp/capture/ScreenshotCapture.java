package dev.ferrite.client.mcp.capture;

import java.util.concurrent.CompletableFuture;

/** Render-thread screenshot request boundary used by the synchronous MCP tool. */
public interface ScreenshotCapture extends AutoCloseable {
    CompletableFuture<CapturedScreenshot> request();

    @Override
    void close();

    static ScreenshotCapture unavailable() {
        return new ScreenshotCapture() {
            @Override
            public CompletableFuture<CapturedScreenshot> request() {
                return CompletableFuture.failedFuture(
                        new IllegalStateException("render capture is unavailable"));
            }

            @Override
            public void close() {}
        };
    }
}
