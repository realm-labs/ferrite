package dev.ferrite.client.mcp.acceptance;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.HashMap;
import java.util.Map;

/** Closed command-line contract for local unattended gameplay acceptance. */
record AcceptanceConfig(Path workspace, Path javaHome, Path ferriteBinary, Path outputRoot, Mode mode) {
    enum Mode {
        REFERENCE,
        FERRITE,
        FERRITE_PORTAL,
        ALL
    }

    static AcceptanceConfig parse(String[] arguments) {
        Map<String, String> values = new HashMap<>();
        for (int index = 0; index < arguments.length; index++) {
            String name = arguments[index];
            if (!name.startsWith("--") || index + 1 >= arguments.length) {
                throw new IllegalArgumentException("expected --name value arguments");
            }
            if (values.putIfAbsent(name, arguments[++index]) != null) {
                throw new IllegalArgumentException("duplicate acceptance argument: " + name);
            }
        }
        for (String name : values.keySet()) {
            if (!name.equals("--workspace")
                    && !name.equals("--java-home")
                    && !name.equals("--ferrite-bin")
                    && !name.equals("--output-root")
                    && !name.equals("--mode")) {
                throw new IllegalArgumentException("unknown acceptance argument: " + name);
            }
        }
        Path workspace = path(values, "--workspace");
        Path javaHome = path(values, "--java-home");
        Path ferrite = values.containsKey("--ferrite-bin")
                ? Path.of(values.get("--ferrite-bin")).toAbsolutePath().normalize()
                : workspace.resolve("target/debug/ferrite-server");
        Path output = values.containsKey("--output-root")
                ? Path.of(values.get("--output-root")).toAbsolutePath().normalize()
                : workspace.resolve("target/client-mcp-evidence");
        Path target = workspace.resolve("target").normalize();
        if (!output.startsWith(target)) {
            throw new IllegalArgumentException("output root must be below the workspace target directory");
        }
        Mode mode = values.containsKey("--mode")
                ? Mode.valueOf(values.get("--mode").toUpperCase(java.util.Locale.ROOT))
                : Mode.ALL;
        if (!Files.isRegularFile(javaHome.resolve("bin/java"))) {
            throw new IllegalArgumentException("Java executable is missing");
        }
        return new AcceptanceConfig(workspace, javaHome, ferrite, output, mode);
    }

    Path clientJar() {
        return workspace.resolve("target/mc-reference/26.2/client.jar");
    }

    Path serverJar() {
        return workspace.resolve("target/mc-reference/26.2/server.jar");
    }

    Path registryReport() {
        return workspace.resolve("target/mc-reference/26.2/generated/reports/registries.json");
    }

    Path launcherJar() {
        Path directory = workspace.resolve("tools/ferrite-client-mcp/build/libs");
        try (var files = Files.list(directory)) {
            return files.filter(path -> path.getFileName().toString().endsWith("-launcher.jar"))
                    .findFirst()
                    .orElseThrow(() -> new IllegalArgumentException("launcher JAR is missing"));
        } catch (java.io.IOException error) {
            throw new IllegalArgumentException("launcher JAR cannot be resolved", error);
        }
    }

    private static Path path(Map<String, String> values, String name) {
        String value = values.get(name);
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException(name + " is required");
        }
        return Path.of(value).toAbsolutePath().normalize();
    }
}
