package dev.ferrite.client.mcp.acceptance;

import java.time.Duration;
import java.util.Comparator;
import java.util.List;
import java.util.concurrent.TimeUnit;

/** Bounded termination shared by acceptance-owned child processes. */
final class ProcessTree {
    private ProcessTree() {}

    static void terminate(Process process, Duration grace) {
        ProcessHandle handle = process.toHandle();
        List<ProcessHandle> descendants = handle.descendants()
                .sorted(Comparator.comparingLong(ProcessHandle::pid).reversed())
                .toList();
        descendants.forEach(ProcessHandle::destroy);
        handle.destroy();
        try {
            if (!process.waitFor(grace.toMillis(), TimeUnit.MILLISECONDS)) {
                descendants.stream()
                        .filter(ProcessHandle::isAlive)
                        .forEach(ProcessHandle::destroyForcibly);
                handle.destroyForcibly();
                process.waitFor(grace.toMillis(), TimeUnit.MILLISECONDS);
            }
            descendants.stream()
                    .filter(ProcessHandle::isAlive)
                    .forEach(ProcessHandle::destroyForcibly);
        } catch (InterruptedException error) {
            Thread.currentThread().interrupt();
            descendants.stream()
                    .filter(ProcessHandle::isAlive)
                    .forEach(ProcessHandle::destroyForcibly);
            handle.destroyForcibly();
        }
    }
}
