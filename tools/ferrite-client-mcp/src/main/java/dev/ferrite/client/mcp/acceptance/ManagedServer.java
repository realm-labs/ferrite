package dev.ferrite.client.mcp.acceptance;

import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import dev.ferrite.client.mcp.launcher.ArtifactVerifier;
import java.io.IOException;
import java.net.InetAddress;
import java.net.InetSocketAddress;
import java.net.ServerSocket;
import java.net.URI;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.time.Duration;
import java.util.List;
import java.util.concurrent.TimeUnit;

/** Isolated reference or Ferrite server with bounded readiness and shutdown. */
final class ManagedServer implements AutoCloseable {
    private enum Kind {
        REFERENCE,
        FERRITE
    }

    private static final Duration READY_TIMEOUT = Duration.ofSeconds(60);
    private static final Duration STOP_TIMEOUT = Duration.ofSeconds(20);

    private final Kind kind;
    private final Process process;
    private final String endpoint;
    private final URI management;
    private final Path log;

    private ManagedServer(
            Kind kind, Process process, String endpoint, URI management, Path log) {
        this.kind = kind;
        this.process = process;
        this.endpoint = endpoint;
        this.management = management;
        this.log = log;
    }

    static ManagedServer startReference(AcceptanceConfig config, EvidenceBundle evidence)
            throws IOException, InterruptedException {
        ArtifactVerifier.verifyServer(config.serverJar());
        int port = freePort();
        Path directory = evidence.root().resolve("reference-server");
        Files.createDirectory(directory);
        Files.writeString(directory.resolve("eula.txt"), "eula=true\n", StandardCharsets.UTF_8);
        Files.writeString(
                directory.resolve("server.properties"),
                "server-ip=127.0.0.1\nserver-port="
                        + port
                        + "\nonline-mode=false\nforce-gamemode=true\ngamemode=survival\n"
                        + "difficulty=peaceful\nlevel-type=minecraft:flat\nview-distance=5\n"
                        + "simulation-distance=5\nenable-rcon=false\nenable-command-block=false\n"
                        + "level-seed=FerriteMcp26.2\ngenerate-structures=false\n"
                        + "spawn-npcs=false\nspawn-animals=false\nspawn-monsters=false\n"
                        + "spawn-protection=0\nmotd=Ferrite client MCP reference\n",
                StandardCharsets.UTF_8);
        Path log = evidence.root().resolve("reference-server-process.log");
        Process process = start(
                List.of(
                        config.javaHome().resolve("bin/java").toString(),
                        "-Xms256M",
                        "-Xmx768M",
                        "-jar",
                        config.serverJar().toString(),
                        "nogui"),
                directory,
                log);
        ManagedServer server = new ManagedServer(
                Kind.REFERENCE, process, "127.0.0.1:" + port, null, log);
        try {
            server.awaitLog("Done (", READY_TIMEOUT);
            evidence.writeText("reference-endpoint.txt", server.endpoint + System.lineSeparator());
            return server;
        } catch (IOException | InterruptedException error) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw error;
        }
    }

    static ManagedServer startFerrite(AcceptanceConfig config, EvidenceBundle evidence)
            throws IOException, InterruptedException {
        if (!Files.isRegularFile(config.ferriteBinary()) || !Files.isRegularFile(config.registryReport())) {
            throw new IOException("Ferrite binary or locked registry report is missing");
        }
        int remoting = freePort();
        int managementPort = distinctPort(remoting);
        int minecraft = distinctPort(remoting, managementPort);
        Path directory = evidence.root().resolve("ferrite-server");
        Files.createDirectory(directory);
        Path configFile = directory.resolve("server.toml");
        Files.writeString(
                configFile,
                ferriteConfig(config, directory, remoting, managementPort, minecraft),
                StandardCharsets.UTF_8);
        Path log = evidence.root().resolve("ferrite-server-process.log");
        Process process = start(
                List.of(config.ferriteBinary().toString(), "--config", configFile.toString()),
                config.workspace(),
                log);
        URI management = URI.create("http://127.0.0.1:" + managementPort);
        ManagedServer server = new ManagedServer(
                Kind.FERRITE, process, "127.0.0.1:" + minecraft, management, log);
        try {
            server.awaitLog("minecraft=127.0.0.1:" + minecraft, READY_TIMEOUT);
            server.awaitFerriteReady();
            evidence.writeText("ferrite-endpoint.txt", server.endpoint + System.lineSeparator());
            return server;
        } catch (IOException | InterruptedException error) {
            ProcessTree.terminate(process, Duration.ofSeconds(5));
            throw error;
        }
    }

    String endpoint() {
        return endpoint;
    }

    JsonObject captureStatus(EvidenceBundle evidence, String name)
            throws IOException, InterruptedException {
        if (management == null) {
            throw new IOException("reference server has no management status");
        }
        HttpResponse<String> response = HttpClient.newHttpClient().send(
                HttpRequest.newBuilder(management.resolve("/status"))
                        .timeout(Duration.ofSeconds(5))
                        .GET()
                        .build(),
                HttpResponse.BodyHandlers.ofString());
        evidence.writeText(name, response.body() + System.lineSeparator());
        if (response.statusCode() != 200) {
            throw new IOException("Ferrite status failed with HTTP " + response.statusCode());
        }
        return JsonParser.parseString(response.body()).getAsJsonObject();
    }

    @Override
    public void close() {
        if (!process.isAlive()) {
            return;
        }
        try {
            if (kind == Kind.REFERENCE) {
                process.getOutputStream().write("stop\n".getBytes(StandardCharsets.UTF_8));
                process.getOutputStream().flush();
            } else {
                HttpClient.newHttpClient().send(
                        HttpRequest.newBuilder(management.resolve("/drain"))
                                .timeout(Duration.ofSeconds(5))
                                .POST(HttpRequest.BodyPublishers.noBody())
                                .build(),
                        HttpResponse.BodyHandlers.discarding());
            }
            if (process.waitFor(STOP_TIMEOUT.toMillis(), TimeUnit.MILLISECONDS)) {
                return;
            }
        } catch (IOException error) {
            System.err.println("server shutdown request failed");
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
        }
        ProcessTree.terminate(process, Duration.ofSeconds(5));
    }

    private void awaitFerriteReady() throws IOException, InterruptedException {
        HttpClient client = HttpClient.newHttpClient();
        long deadline = System.nanoTime() + READY_TIMEOUT.toNanos();
        while (System.nanoTime() < deadline) {
            if (!process.isAlive()) {
                throw new IOException("Ferrite exited before readiness");
            }
            try {
                HttpResponse<String> response = client.send(
                        HttpRequest.newBuilder(management.resolve("/readyz"))
                                .timeout(Duration.ofSeconds(2))
                                .GET()
                                .build(),
                        HttpResponse.BodyHandlers.ofString());
                if (response.statusCode() == 200 && response.body().contains("\"ready\":true")) {
                    return;
                }
            } catch (IOException ignored) {
                // Listener creation and readiness publication are separate bounded states.
            }
            Thread.sleep(25);
        }
        throw new IOException("Ferrite readiness timed out");
    }

    private void awaitLog(String marker, Duration timeout) throws IOException, InterruptedException {
        long deadline = System.nanoTime() + timeout.toNanos();
        while (System.nanoTime() < deadline) {
            if (Files.isRegularFile(log) && Files.readString(log).contains(marker)) {
                return;
            }
            if (!process.isAlive()) {
                throw new IOException("server exited before readiness: " + Files.readString(log));
            }
            Thread.sleep(25);
        }
        throw new IOException("server readiness timed out");
    }

    private static Process start(List<String> command, Path directory, Path log) throws IOException {
        return new ProcessBuilder(command)
                .directory(directory.toFile())
                .redirectErrorStream(true)
                .redirectOutput(log.toFile())
                .start();
    }

    private static int freePort() throws IOException {
        try (ServerSocket socket = new ServerSocket()) {
            socket.bind(new InetSocketAddress(InetAddress.getLoopbackAddress(), 0));
            return socket.getLocalPort();
        }
    }

    private static int distinctPort(int... excluded) throws IOException {
        while (true) {
            int candidate = freePort();
            if (java.util.Arrays.stream(excluded).noneMatch(port -> port == candidate)) {
                return candidate;
            }
        }
    }

    private static String ferriteConfig(
            AcceptanceConfig config, Path directory, int remoting, int management, int minecraft) {
        return """
                schema_version = 1
                [cluster]
                name = "ferrite-goal02"
                [node]
                id = "goal02-node"
                roles = ["gateway", "region-worker", "coordinator-candidate", "administration"]
                [remoting]
                bind = "127.0.0.1:%d"
                advertise = { host = "127.0.0.1", port = %d }
                [discovery]
                provider = "development-static"
                minimum_members = 1
                peers = [{ host = "127.0.0.1", port = %d }]
                [placement]
                capacity_regions = 256
                required_domains = ["ferrite-region-v1"]
                [storage]
                root = "%s"
                [management]
                bind = "127.0.0.1:%d"
                allow_remote_drain = false
                [minecraft]
                enabled = true
                bind = "127.0.0.1:%d"
                registry_report = "%s"
                [limits]
                max_sessions = 16
                max_region_mailbox = 1024
                max_management_request_bytes = 4096
                [shutdown]
                drain_timeout_millis = 10000
                """
                .formatted(
                        remoting,
                        remoting,
                        remoting,
                        directory.resolve("state").toString(),
                        management,
                        minecraft,
                        config.registryReport());
    }
}
