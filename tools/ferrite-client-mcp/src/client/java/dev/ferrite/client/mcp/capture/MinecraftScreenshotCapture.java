package dev.ferrite.client.mcp.capture;

import com.mojang.blaze3d.pipeline.RenderTarget;
import com.mojang.blaze3d.platform.NativeImage;
import dev.ferrite.client.mcp.observation.ClientObservationStore;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.HexFormat;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicReference;
import net.minecraft.client.Minecraft;
import net.minecraft.client.Screenshot;

/** Single-flight render-thread capture of Minecraft's main framebuffer. */
public final class MinecraftScreenshotCapture implements ScreenshotCapture {
    private static final long MAXIMUM_PIXELS = 16_777_216;
    private static final long MAXIMUM_PNG_BYTES = 16L * 1024 * 1024;

    private final AtomicReference<CompletableFuture<CapturedScreenshot>> pending =
            new AtomicReference<>();
    private final AtomicBoolean closed = new AtomicBoolean();
    private final ClientObservationStore observations;

    public MinecraftScreenshotCapture(ClientObservationStore observations) {
        this.observations = observations;
    }

    @Override
    public CompletableFuture<CapturedScreenshot> request() {
        if (closed.get()) {
            return CompletableFuture.failedFuture(
                    new IllegalStateException("render capture is shut down"));
        }

        CompletableFuture<CapturedScreenshot> future = new CompletableFuture<>();
        if (!pending.compareAndSet(null, future)) {
            return CompletableFuture.failedFuture(
                    new IllegalStateException("another framebuffer capture is already pending"));
        }
        future.whenComplete((result, error) -> pending.compareAndSet(future, null));

        Minecraft client = Minecraft.getInstance();
        try {
            client.execute(() -> captureOnRenderThread(client, observations.latest().clientTick(), future));
        } catch (RuntimeException error) {
            future.completeExceptionally(error);
        }
        return future;
    }

    @Override
    public void close() {
        closed.set(true);
        CompletableFuture<CapturedScreenshot> future = pending.getAndSet(null);
        if (future != null) {
            future.completeExceptionally(new IllegalStateException("render capture shut down"));
        }
    }

    private static void captureOnRenderThread(
            Minecraft client,
            long clientTick,
            CompletableFuture<CapturedScreenshot> future) {
        if (future.isDone()) {
            return;
        }
        if (!client.isSameThread()) {
            future.completeExceptionally(
                    new IllegalStateException("framebuffer capture ran off the render thread"));
            return;
        }
        try {
            RenderTarget target = client.gameRenderer.mainRenderTarget();
            validateGeometry(target.width, target.height);
            Screenshot.takeScreenshot(
                    target, image -> encodeScreenshot(image, clientTick, future));
        } catch (RuntimeException error) {
            future.completeExceptionally(error);
        }
    }

    private static void encodeScreenshot(
            NativeImage image, long clientTick, CompletableFuture<CapturedScreenshot> future) {
        Path temporary = null;
        try (image) {
            if (future.isDone()) {
                return;
            }
            validateGeometry(image.getWidth(), image.getHeight());
            temporary = Files.createTempFile("ferrite-client-mcp-", ".png");
            image.writeToFile(temporary);
            long byteLength = Files.size(temporary);
            if (byteLength < 1 || byteLength > MAXIMUM_PNG_BYTES) {
                throw new IOException("encoded PNG exceeds the 16 MiB response bound");
            }
            byte[] png = Files.readAllBytes(temporary);
            future.complete(new CapturedScreenshot(
                    png,
                    image.getWidth(),
                    image.getHeight(),
                    clientTick,
                    sha256(png)));
        } catch (IOException | RuntimeException error) {
            future.completeExceptionally(error);
        } finally {
            if (temporary != null) {
                try {
                    Files.deleteIfExists(temporary);
                } catch (IOException ignored) {
                    // The OS temporary directory remains outside the evidence boundary.
                }
            }
        }
    }

    private static void validateGeometry(int width, int height) {
        long pixels = (long) width * height;
        if (width < 1 || height < 1 || pixels > MAXIMUM_PIXELS) {
            throw new IllegalStateException("framebuffer geometry exceeds the 16 Mi-pixel bound");
        }
    }

    private static String sha256(byte[] bytes) {
        try {
            return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
        } catch (NoSuchAlgorithmException impossible) {
            throw new IllegalStateException("SHA-256 is unavailable", impossible);
        }
    }
}
