package dev.ferrite.client.mcp.launcher;

import java.net.InetAddress;
import java.net.UnknownHostException;
import java.nio.file.Path;
import java.time.Duration;
import java.util.HashMap;
import java.util.Map;

/** Validated command-line contract for one isolated client process. */
record LauncherConfig(
        Path workspace,
        Path javaHome,
        Path referenceClient,
        Path runRoot,
        String endpoint,
        Duration readyTimeout,
        Duration maximumRuntime,
        boolean retainRun) {
    static LauncherConfig parse(String[] arguments) {
        Map<String, String> values = new HashMap<>();
        boolean retain = false;
        for (int index = 0; index < arguments.length; index++) {
            String argument = arguments[index];
            if (argument.equals("--retain-run")) {
                retain = true;
                continue;
            }
            if (!argument.startsWith("--") || index + 1 >= arguments.length) {
                throw new IllegalArgumentException("expected --name value arguments");
            }
            if (values.putIfAbsent(argument, arguments[++index]) != null) {
                throw new IllegalArgumentException("duplicate launcher argument: " + argument);
            }
        }
        rejectUnknown(values);
        Path workspace = requiredPath(values, "--workspace");
        Path javaHome = requiredPath(values, "--java-home");
        Path referenceClient = values.containsKey("--reference-client")
                ? Path.of(values.get("--reference-client")).toAbsolutePath().normalize()
                : workspace.resolve("target/mc-reference/26.2/client.jar");
        Path runRoot = values.containsKey("--run-root")
                ? Path.of(values.get("--run-root")).toAbsolutePath().normalize()
                : workspace.resolve("target/client-mcp-runs");
        Path permittedRoot = workspace.resolve("target").normalize();
        if (!runRoot.startsWith(permittedRoot)) {
            throw new IllegalArgumentException("run root must be below the workspace target directory");
        }
        String endpoint = required(values, "--endpoint");
        validateEndpoint(endpoint);
        Duration ready = seconds(values, "--ready-timeout-seconds", 90, 5, 300);
        Duration runtime = seconds(values, "--max-runtime-seconds", 300, 10, 3600);
        return new LauncherConfig(
                workspace, javaHome, referenceClient, runRoot, endpoint, ready, runtime, retain);
    }

    private static void rejectUnknown(Map<String, String> values) {
        for (String name : values.keySet()) {
            if (!name.equals("--workspace")
                    && !name.equals("--java-home")
                    && !name.equals("--reference-client")
                    && !name.equals("--run-root")
                    && !name.equals("--endpoint")
                    && !name.equals("--ready-timeout-seconds")
                    && !name.equals("--max-runtime-seconds")) {
                throw new IllegalArgumentException("unknown launcher argument: " + name);
            }
        }
    }

    private static Path requiredPath(Map<String, String> values, String name) {
        return Path.of(required(values, name)).toAbsolutePath().normalize();
    }

    private static String required(Map<String, String> values, String name) {
        String value = values.get(name);
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException(name + " is required");
        }
        return value;
    }

    private static Duration seconds(
            Map<String, String> values, String name, int fallback, int minimum, int maximum) {
        int number = values.containsKey(name) ? Integer.parseInt(values.get(name)) : fallback;
        if (number < minimum || number > maximum) {
            throw new IllegalArgumentException(name + " must be between " + minimum + " and " + maximum);
        }
        return Duration.ofSeconds(number);
    }

    private static void validateEndpoint(String endpoint) {
        int separator = endpoint.lastIndexOf(':');
        if (separator < 1 || separator == endpoint.length() - 1) {
            throw new IllegalArgumentException("endpoint must be loopback-host:port");
        }
        String host = endpoint.substring(0, separator);
        int port = Integer.parseInt(endpoint.substring(separator + 1));
        try {
            if (!InetAddress.getByName(host).isLoopbackAddress() || port < 1 || port > 65535) {
                throw new IllegalArgumentException("endpoint must resolve to loopback with a valid port");
            }
        } catch (UnknownHostException error) {
            throw new IllegalArgumentException("endpoint host cannot be resolved", error);
        }
    }
}
