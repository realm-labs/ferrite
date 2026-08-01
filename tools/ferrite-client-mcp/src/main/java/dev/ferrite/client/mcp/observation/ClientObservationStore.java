package dev.ferrite.client.mcp.observation;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.atomic.AtomicReference;

/** Thread-safe publication boundary between the Minecraft thread and MCP workers. */
public final class ClientObservationStore {
    private static final int MAXIMUM_ERRORS = 64;

    private final AtomicReference<ClientSnapshot> latest =
            new AtomicReference<>(ClientSnapshot.starting());
    private final ArrayDeque<ClientError> errors = new ArrayDeque<>();

    public ClientSnapshot latest() {
        return latest.get();
    }

    public synchronized void publish(ClientSnapshot snapshot) {
        latest.set(Objects.requireNonNull(snapshot, "snapshot"));
        notifyAll();
    }

    /** Waits for a later published client tick, with a wall-clock guard for paused clients. */
    public synchronized ClientSnapshot awaitNext(long afterTick, long timeoutMillis)
            throws InterruptedException {
        if (timeoutMillis < 1) {
            throw new IllegalArgumentException("observation timeout must be positive");
        }
        long remainingNanos = timeoutMillis * 1_000_000L;
        long deadline = System.nanoTime() + remainingNanos;
        ClientSnapshot snapshot = latest.get();
        while (snapshot.clientTick() <= afterTick && remainingNanos > 0) {
            wait(Math.max(1, remainingNanos / 1_000_000L));
            snapshot = latest.get();
            remainingNanos = deadline - System.nanoTime();
        }
        return snapshot;
    }

    public synchronized void recordError(long clientTick, String category, String message) {
        while (errors.size() >= MAXIMUM_ERRORS) {
            errors.removeFirst();
        }
        errors.addLast(new ClientError(clientTick, category, SensitiveText.redact(message)));
    }

    public synchronized List<ClientError> errors(int limit) {
        if (limit < 1 || limit > MAXIMUM_ERRORS) {
            throw new IllegalArgumentException("error limit must be between 1 and 64");
        }
        List<ClientError> snapshot = new ArrayList<>(errors);
        int fromIndex = Math.max(0, snapshot.size() - limit);
        return List.copyOf(snapshot.subList(fromIndex, snapshot.size()));
    }
}
