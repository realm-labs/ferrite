package dev.ferrite.client.mcp.launcher;

import java.io.IOException;
import java.nio.file.Files;
import java.util.concurrent.atomic.AtomicBoolean;

/** Pure-Java supervisor for one exact isolated Quick Play client. */
public final class ClientLauncher {
    private ClientLauncher() {}

    public static void main(String[] arguments) {
        int exitCode;
        try {
            exitCode = run(arguments);
        } catch (IllegalArgumentException error) {
            System.err.println("{\"state\":\"Rejected\",\"reason\":\"invalid launcher arguments\"}");
            exitCode = 2;
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            System.err.println("{\"state\":\"Cancelled\",\"reason\":\"launcher interrupted\"}");
            exitCode = 130;
        } catch (IOException error) {
            System.err.println("{\"state\":\"Rejected\",\"reason\":\"client launch failed\"}");
            exitCode = 1;
        }
        if (exitCode != 0) {
            System.exit(exitCode);
        }
    }

    private static int run(String[] arguments) throws IOException, InterruptedException {
        LauncherConfig config = LauncherConfig.parse(arguments);
        ArtifactVerifier.verifyClient(config.referenceClient());
        IsolatedClientRun run = IsolatedClientRun.create(config.runRoot());
        ManagedClientProcess process = null;
        AtomicBoolean cleaned = new AtomicBoolean();
        try {
            process = ManagedClientProcess.start(config, run);
            ManagedClientProcess ownedProcess = process;
            Thread shutdown = new Thread(
                    () -> cleanup(ownedProcess, run, config.retainRun(), cleaned),
                    "ferrite-client-launcher-shutdown");
            Runtime.getRuntime().addShutdownHook(shutdown);
            process.awaitReady(run, config.readyTimeout());
            String ready = Files.readString(run.readyFile()).strip();
            if (!ready.matches("\\{\"endpoint\":\"http://127\\.0\\.0\\.1:[0-9]{1,5}/mcp\"}")) {
                throw new IOException("MCP ready file has an invalid endpoint");
            }
            System.out.println("{\"state\":\"ready\",\"runId\":\""
                    + run.root().getFileName()
                    + "\",\"mcp\":"
                    + ready
                    + "}");
            if (!process.waitFor(config.maximumRuntime())) {
                System.err.println("{\"state\":\"TimedOut\",\"reason\":\"maximum runtime elapsed\"}");
                return 124;
            }
            if (process.exitValue() != 0) {
                throw new IOException("Minecraft client exited with code " + process.exitValue());
            }
            cleanup(process, run, config.retainRun(), cleaned);
            Runtime.getRuntime().removeShutdownHook(shutdown);
            return 0;
        } finally {
            if (process != null) {
                cleanup(process, run, config.retainRun(), cleaned);
            } else if (!config.retainRun()) {
                run.delete();
            }
        }
    }

    private static void cleanup(
            ManagedClientProcess process,
            IsolatedClientRun run,
            boolean retainRun,
            AtomicBoolean cleaned) {
        if (!cleaned.compareAndSet(false, true)) {
            return;
        }
        process.close();
        if (!retainRun) {
            try {
                run.delete();
            } catch (IOException error) {
                System.err.println("failed to delete isolated client run");
            }
        }
    }
}
