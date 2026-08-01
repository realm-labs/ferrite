package dev.ferrite.client.mcp.control;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertInstanceOf;

import java.util.Set;
import org.junit.jupiter.api.Test;

final class ClientActionQueueTest {
    @Test
    void ownsInputsUntilBoundedActionCompletes() {
        ClientActionQueue queue = new ClientActionQueue();
        ClientAction.Inputs first = movement("walk-1");

        assertEquals(ActionState.QUEUED, queue.submit(first).state());
        assertEquals(
                ActionState.REJECTED,
                queue.submit(movement("walk-2")).state(),
                "the same held input must have only one MCP owner");
        assertInstanceOf(ClientAction.Inputs.class, queue.poll(12));
        assertEquals(ActionState.APPLIED, queue.markApplied("walk-1").state());
        assertEquals(
                ActionState.SATISFIED,
                queue.complete("walk-1", ActionState.SATISFIED, "elapsed").state());
        assertEquals(ActionState.QUEUED, queue.submit(movement("walk-3")).state());
    }

    @Test
    void rejectsDuplicateAndInvalidActionIdentifiers() {
        ClientActionQueue queue = new ClientActionQueue();
        assertEquals(ActionState.QUEUED, queue.submit(new ClientAction.ReleaseAll("release.1")).state());
        assertEquals(
                ActionState.REJECTED,
                queue.submit(new ClientAction.ReleaseAll("release.1")).state());
        assertEquals(
                ActionState.REJECTED,
                queue.submit(new ClientAction.ReleaseAll("bad action id")).state());
    }

    @Test
    void priorityReleaseBypassesAFullQueueAndCancelsOutstandingWork() {
        ClientActionQueue queue = new ClientActionQueue();
        for (int index = 0; index < ClientActionQueue.MAXIMUM_PENDING_ACTIONS; index++) {
            assertEquals(
                    ActionState.QUEUED,
                    queue.submit(new ClientAction.Look("look-" + index, 0, 0, false)).state());
        }
        assertEquals(
                ActionState.REJECTED,
                queue.submit(new ClientAction.Look("overflow", 0, 0, false)).state());

        assertEquals(
                ActionState.QUEUED,
                queue.submit(new ClientAction.ReleaseAll("release-priority")).state());
        assertInstanceOf(ClientAction.ReleaseAll.class, queue.poll(20));
        queue.markApplied("release-priority");
        queue.cancelOutstandingExcept("release-priority", "released");
        queue.complete("release-priority", ActionState.SATISFIED, "released");

        assertEquals(ActionState.CANCELLED, queue.status("look-0").state());
        assertEquals(ActionState.SATISFIED, queue.status("release-priority").state());
        assertEquals(0, queue.pendingCount());
    }

    @Test
    void closeCancelsQueuedAndRejectsLaterActions() {
        ClientActionQueue queue = new ClientActionQueue();
        queue.submit(new ClientAction.Look("look-before-close", 0, 0, false));
        queue.close("shutdown");

        assertEquals(ActionState.CANCELLED, queue.status("look-before-close").state());
        assertEquals(
                ActionState.REJECTED,
                queue.submit(new ClientAction.Look("look-after-close", 0, 0, false)).state());
    }

    @Test
    void disconnectCancellationReleasesAStuckInputReservation() {
        ClientActionQueue queue = new ClientActionQueue();
        assertEquals(ActionState.QUEUED, queue.submit(movement("stuck-before-disconnect")).state());
        queue.poll(4);
        queue.markApplied("stuck-before-disconnect");

        queue.cancelOutstanding("world or connection changed");

        assertEquals(ActionState.CANCELLED, queue.status("stuck-before-disconnect").state());
        assertEquals(ActionState.QUEUED, queue.submit(movement("after-reconnect")).state());
    }

    private static ClientAction.Inputs movement(String actionId) {
        return new ClientAction.Inputs(
                actionId,
                "hold_movement",
                Set.of(ControlledInput.FORWARD),
                true,
                10);
    }
}
