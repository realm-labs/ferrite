package dev.ferrite.client.mcp.launcher;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.StandardWatchEventKinds;
import java.nio.file.WatchKey;
import java.nio.file.WatchService;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.TimeUnit;

/** Process-tree ownership and event-driven MCP readiness waiting. */
final class ManagedClientProcess implements AutoCloseable {
    private final Process process;

    private ManagedClientProcess(Process process) {
        this.process = process;
    }

    static ManagedClientProcess start(LauncherConfig config, IsolatedClientRun run)
            throws IOException {
        var project = config.workspace().resolve("tools/ferrite-client-mcp");
        var wrapper = project.resolve("gradlew");
        if (!Files.isRegularFile(wrapper)
                || !Files.isRegularFile(config.javaHome().resolve("bin/java"))) {
            throw new IOException("Gradle wrapper or Java executable is missing");
        }
        List<String> command = List.of(
                wrapper.toString(),
                "--no-daemon",
                "-PferriteGameDir=" + run.gameDirectory(),
                "runClient",
                "--args=--quickPlayMultiplayer " + config.endpoint());
        ProcessBuilder builder = new ProcessBuilder(command)
                .directory(project.toFile())
                .redirectErrorStream(true)
                .redirectOutput(run.clientLog().toFile());
        builder.environment().put("JAVA_HOME", config.javaHome().toString());
        builder.environment().put("FERRITE_CLIENT_MCP_SECRET_FILE", run.secretFile().toString());
        builder.environment().put("FERRITE_CLIENT_MCP_READY_FILE", run.readyFile().toString());
        return new ManagedClientProcess(builder.start());
    }

    void awaitReady(IsolatedClientRun run, Duration timeout) throws IOException, InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        try (WatchService watcher = run.root().getFileSystem().newWatchService()) {
            run.root().register(
                    watcher,
                    StandardWatchEventKinds.ENTRY_CREATE,
                    StandardWatchEventKinds.ENTRY_MODIFY);
            while (!Files.isRegularFile(run.readyFile())) {
                if (!process.isAlive()) {
                    throw new IOException("Minecraft client exited before MCP readiness");
                }
                long remaining = deadline - System.nanoTime();
                if (remaining <= 0) {
                    throw new IOException("Minecraft client MCP readiness timed out");
                }
                WatchKey key = watcher.poll(remaining, TimeUnit.NANOSECONDS);
                if (key == null) {
                    throw new IOException("Minecraft client MCP readiness timed out");
                }
                key.pollEvents();
                if (!key.reset()) {
                    throw new IOException("client run directory became unavailable");
                }
            }
        }
    }

    boolean waitFor(Duration timeout) throws InterruptedException {
        return process.waitFor(timeout.toMillis(), TimeUnit.MILLISECONDS);
    }

    int exitValue() {
        return process.exitValue();
    }

    @Override
    public void close() {
        ProcessHandle handle = process.toHandle();
        handle.descendants().forEach(child -> child.destroy());
        handle.destroy();
        try {
            if (!process.waitFor(5, TimeUnit.SECONDS)) {
                handle.descendants().forEach(child -> child.destroyForcibly());
                handle.destroyForcibly();
                process.waitFor(5, TimeUnit.SECONDS);
            }
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            handle.descendants().forEach(child -> child.destroyForcibly());
            handle.destroyForcibly();
        }
    }
}
