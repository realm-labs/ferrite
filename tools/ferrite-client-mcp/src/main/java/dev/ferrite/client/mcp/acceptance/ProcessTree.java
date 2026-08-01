package dev.ferrite.client.mcp.acceptance;

import java.time.Duration;
import java.util.concurrent.TimeUnit;

/** Bounded termination shared by acceptance-owned child processes. */
final class ProcessTree {
    private ProcessTree() {}

    static void terminate(Process process, Duration grace) {
        ProcessHandle handle = process.toHandle();
        handle.descendants().forEach(ProcessHandle::destroy);
        handle.destroy();
        try {
            if (!process.waitFor(grace.toMillis(), TimeUnit.MILLISECONDS)) {
                handle.descendants().forEach(ProcessHandle::destroyForcibly);
                handle.destroyForcibly();
                process.waitFor(grace.toMillis(), TimeUnit.MILLISECONDS);
            }
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            handle.descendants().forEach(ProcessHandle::destroyForcibly);
            handle.destroyForcibly();
        }
    }
}
