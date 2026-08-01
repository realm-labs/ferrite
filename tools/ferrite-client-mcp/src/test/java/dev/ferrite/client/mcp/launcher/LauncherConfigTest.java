package dev.ferrite.client.mcp.launcher;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.file.Files;
import java.nio.file.Path;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

final class LauncherConfigTest {
    @TempDir
    Path temporary;

    @Test
    void acceptsOnlyLoopbackAndWorkspaceOwnedRunRoots() throws Exception {
        Path workspace = temporary.resolve("workspace").toAbsolutePath();
        Files.createDirectories(workspace.resolve("target"));
        LauncherConfig config = LauncherConfig.parse(new String[] {
            "--workspace", workspace.toString(),
            "--java-home", temporary.resolve("jdk").toString(),
            "--endpoint", "127.0.0.1:25565",
            "--run-root", workspace.resolve("target/runs").toString()
        });
        assertEquals("127.0.0.1:25565", config.endpoint());
        assertTrue(config.runRoot().startsWith(workspace.resolve("target")));

        assertThrows(
                IllegalArgumentException.class,
                () -> LauncherConfig.parse(new String[] {
                    "--workspace", workspace.toString(),
                    "--java-home", temporary.resolve("jdk").toString(),
                    "--endpoint", "example.com:25565"
                }));
        assertThrows(
                IllegalArgumentException.class,
                () -> LauncherConfig.parse(new String[] {
                    "--workspace", workspace.toString(),
                    "--java-home", temporary.resolve("jdk").toString(),
                    "--endpoint", "127.0.0.1:25565",
                    "--run-root", temporary.resolve("outside").toString()
                }));
        assertThrows(
                IllegalArgumentException.class,
                () -> LauncherConfig.parse(new String[] {
                    "--workspace", workspace.toString(),
                    "--java-home", temporary.resolve("jdk").toString(),
                    "--endpoint", "127.0.0.1:25565",
                    "--max-runtime-seconds", "9"
                }));
    }

    @Test
    void rejectsAnArtifactThatDoesNotMatchTheLockedClient() throws Exception {
        Path impostor = temporary.resolve("client.jar");
        Files.writeString(impostor, "not the locked client");

        assertThrows(java.io.IOException.class, () -> ArtifactVerifier.verifyClient(impostor));
    }

    @Test
    void createsAndDeletesAnOwnerOnlyIsolatedRun() throws Exception {
        IsolatedClientRun run = IsolatedClientRun.create(temporary.resolve("runs"));
        assertTrue(Files.readString(run.gameDirectory().resolve("options.txt"))
                .contains("onboardAccessibility:false"));
        assertEquals(65, Files.size(run.secretFile()));
        run.delete();
        assertTrue(Files.notExists(run.root()));
    }
}
