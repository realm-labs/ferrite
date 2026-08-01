package dev.ferrite.client.mcp.acceptance;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import java.io.BufferedReader;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.Executors;
import java.util.concurrent.TimeUnit;

/** Starts the committed launcher and resolves its secret-bearing run only inside this process. */
final class ManagedLauncher implements AutoCloseable {
    private static final Duration READY_TIMEOUT = Duration.ofSeconds(100);

    private final AcceptanceConfig config;
    private final EvidenceBundle evidence;
    private final Process process;
    private final Path runDirectory;
    private final String mcpEndpoint;
    private final BufferedReader output;

    private ManagedLauncher(
            AcceptanceConfig config,
            EvidenceBundle evidence,
            Process process,
            Path runDirectory,
            String mcpEndpoint,
            BufferedReader output) {
        this.config = config;
        this.evidence = evidence;
        this.process = process;
        this.runDirectory = runDirectory;
        this.mcpEndpoint = mcpEndpoint;
        this.output = output;
    }

    static ManagedLauncher start(
            AcceptanceConfig config, EvidenceBundle evidence, String serverEndpoint)
            throws IOException, InterruptedException {
        List<String> command = List.of(
                config.javaHome().resolve("bin/java").toString(),
                "-jar",
                config.launcherJar().toString(),
                "--workspace",
                config.workspace().toString(),
                "--java-home",
                config.javaHome().toString(),
                "--endpoint",
                serverEndpoint,
                "--ready-timeout-seconds",
                "90",
                "--max-runtime-seconds",
                "300",
                "--retain-run");
        Process process = new ProcessBuilder(command)
                .directory(config.workspace().resolve("tools/ferrite-client-mcp").toFile())
                .redirectErrorStream(true)
                .start();
        BufferedReader output = process.inputReader(StandardCharsets.UTF_8);
        String line;
        var executor = Executors.newVirtualThreadPerTaskExecutor();
        try {
            line = executor.submit(output::readLine).get(READY_TIMEOUT.toMillis(), TimeUnit.MILLISECONDS);
        } catch (java.util.concurrent.TimeoutException error) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw new IOException("client launcher readiness timed out", error);
        } catch (java.util.concurrent.ExecutionException error) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw new IOException("client launcher output failed", error.getCause());
        } finally {
            executor.shutdownNow();
        }
        if (line == null) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw new IOException("client launcher exited before readiness");
        }
        try {
            evidence.writeText("launcher-output.log", line + System.lineSeparator());
            JsonObject ready = JsonParser.parseString(line).getAsJsonObject();
            if (!"ready".equals(ready.get("state").getAsString())) {
                throw new IOException("client launcher rejected startup");
            }
            String runId = ready.get("runId").getAsString();
            if (!runId.matches("run-[0-9a-f-]{36}")) {
                throw new IOException("client launcher returned an invalid run ID");
            }
            Path run = config.workspace().resolve("target/client-mcp-runs").resolve(runId).normalize();
            String endpoint = ready.getAsJsonObject("mcp").get("endpoint").getAsString();
            if (!run.startsWith(config.workspace().resolve("target/client-mcp-runs"))
                    || !endpoint.matches("http://127\\.0\\.0\\.1:[0-9]{1,5}/mcp")) {
                throw new IOException("client launcher returned unsafe readiness data");
            }
            return new ManagedLauncher(config, evidence, process, run, endpoint, output);
        } catch (IOException | RuntimeException error) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw new IOException("client launcher returned invalid readiness data", error);
        }
    }

    McpClient connectMcp() throws IOException, InterruptedException {
        return McpClient.initialize(
                java.net.URI.create(mcpEndpoint), runDirectory.resolve("mcp.secret"), evidence);
    }

    @Override
    public void close() {
        ProcessTree.terminate(process, Duration.ofSeconds(10));
        try {
            for (String line; (line = output.readLine()) != null; ) {
                Files.writeString(
                        evidence.root().resolve("launcher-output.log"),
                        line + System.lineSeparator(),
                        StandardCharsets.UTF_8,
                        java.nio.file.StandardOpenOption.APPEND);
            }
            evidence.copyIfPresent(runDirectory.resolve("client.log"), "client-process.log");
            evidence.copyIfPresent(runDirectory.resolve("game/logs/latest.log"), "minecraft-latest.log");
            deleteRun();
        } catch (IOException error) {
            System.err.println("failed to collect or remove isolated client run");
        }
    }

    private void deleteRun() throws IOException {
        Path permitted = config.workspace().resolve("target/client-mcp-runs").normalize();
        if (!runDirectory.startsWith(permitted) || !runDirectory.getFileName().toString().startsWith("run-")) {
            throw new IOException("refusing to delete an unowned client run");
        }
        if (!Files.exists(runDirectory)) {
            return;
        }
        try (var paths = Files.walk(runDirectory)) {
            for (Path path : paths.sorted(java.util.Comparator.reverseOrder()).toList()) {
                Files.deleteIfExists(path);
            }
        }
    }
}
