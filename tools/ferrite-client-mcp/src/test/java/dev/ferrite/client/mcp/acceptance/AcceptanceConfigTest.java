package dev.ferrite.client.mcp.acceptance;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class AcceptanceConfigTest {
    @TempDir
    Path temporary;

    @Test
    void parsesAWorkspaceOwnedReferenceRun() throws Exception {
        Path workspace = temporary.resolve("workspace");
        Path javaHome = temporary.resolve("jdk");
        Files.createDirectories(workspace.resolve("target"));
        Files.createDirectories(javaHome.resolve("bin"));
        Files.createFile(javaHome.resolve("bin/java"));

        AcceptanceConfig config = AcceptanceConfig.parse(new String[] {
            "--workspace", workspace.toString(),
            "--java-home", javaHome.toString(),
            "--mode", "reference"
        });

        assertEquals(AcceptanceConfig.Mode.REFERENCE, config.mode());
        assertEquals(workspace.resolve("target/client-mcp-evidence"), config.outputRoot());
    }

    @Test
    void rejectsEvidenceOutsideTheWorkspaceTarget() throws Exception {
        Path workspace = temporary.resolve("workspace");
        Path javaHome = temporary.resolve("jdk");
        Files.createDirectories(workspace.resolve("target"));
        Files.createDirectories(javaHome.resolve("bin"));
        Files.createFile(javaHome.resolve("bin/java"));

        assertThrows(
                IllegalArgumentException.class,
                () -> AcceptanceConfig.parse(new String[] {
                    "--workspace", workspace.toString(),
                    "--java-home", javaHome.toString(),
                    "--output-root", temporary.resolve("outside").toString()
                }));
    }
}
