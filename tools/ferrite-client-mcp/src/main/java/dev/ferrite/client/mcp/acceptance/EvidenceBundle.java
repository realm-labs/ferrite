package dev.ferrite.client.mcp.acceptance;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.time.Instant;
import java.util.UUID;

/** Secret-free local artifacts for one deterministic client scenario. */
final class EvidenceBundle {
    private static final Gson JSON = new GsonBuilder().setPrettyPrinting().create();

    private final String scenario;
    private final Path root;
    private int sequence;

    private EvidenceBundle(String scenario, Path root) {
        this.scenario = scenario;
        this.root = root;
    }

    static EvidenceBundle create(Path outputRoot, String scenario) throws IOException {
        Files.createDirectories(outputRoot);
        Path root = outputRoot.resolve(scenario + "-" + UUID.randomUUID());
        Files.createDirectory(root);
        return new EvidenceBundle(scenario, root);
    }

    Path root() {
        return root;
    }

    synchronized void response(String operation, JsonElement response) throws IOException {
        sequence++;
        writeJson("%02d-%s.json".formatted(sequence, safe(operation)), response);
    }

    void writeJson(String name, JsonElement value) throws IOException {
        Files.writeString(
                root.resolve(name), JSON.toJson(value) + System.lineSeparator(), StandardCharsets.UTF_8);
    }

    void writeText(String name, String value) throws IOException {
        Files.writeString(root.resolve(name), value, StandardCharsets.UTF_8);
    }

    void copyIfPresent(Path source, String name) throws IOException {
        if (Files.isRegularFile(source)) {
            Files.copy(source, root.resolve(name), StandardCopyOption.REPLACE_EXISTING);
        }
    }

    void finish(String state, String detail) throws IOException {
        JsonObject summary = new JsonObject();
        summary.addProperty("scenario", scenario);
        summary.addProperty("state", state);
        summary.addProperty("detail", detail);
        summary.addProperty("completedAt", Instant.now().toString());
        writeJson("summary.json", summary);
    }

    private static String safe(String value) {
        return value.toLowerCase(java.util.Locale.ROOT).replaceAll("[^a-z0-9-]+", "-");
    }
}
