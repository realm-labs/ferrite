package dev.ferrite.client.mcp.observation;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import dev.ferrite.client.mcp.observation.ClientSnapshot.Inventory;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Item;
import java.util.ArrayList;
import java.util.List;
import org.junit.jupiter.api.Test;

final class ClientObservationStoreTest {
    @Test
    void awaitNextReturnsOnlyAfterALaterTickIsPublished() throws Exception {
        ClientObservationStore store = new ClientObservationStore();
        ClientSnapshot later = snapshot(2);
        java.util.concurrent.CompletableFuture<ClientSnapshot> waiting =
                java.util.concurrent.CompletableFuture.supplyAsync(() -> {
                    try {
                        return store.awaitNext(0, 1_000);
                    } catch (InterruptedException error) {
                        throw new IllegalStateException(error);
                    }
                });
        store.publish(later);
        assertEquals(later, waiting.get(1, java.util.concurrent.TimeUnit.SECONDS));
    }

    @Test
    void snapshotsDefensivelyCopyListsAndRejectNullPublication() {
        List<Item> mutable = new ArrayList<>();
        mutable.add(new Item(0, "minecraft:stone", 1, 0, 0));
        Inventory inventory = new Inventory(true, 0, mutable);
        mutable.clear();

        assertEquals(1, inventory.items().size());
        assertThrows(
                UnsupportedOperationException.class,
                () -> inventory.items().add(new Item(1, "minecraft:dirt", 1, 0, 0)));

        ClientObservationStore store = new ClientObservationStore();
        assertThrows(NullPointerException.class, () -> store.publish(null));
    }

    @Test
    void errorRingIsBoundedNewestFirstAndRedacted() {
        ClientObservationStore store = new ClientObservationStore();
        String home = System.getProperty("user.home");
        store.recordError(
                1,
                "connection",
                "path=" + home + "/game authorization: secret-value access_token=another");
        for (int index = 2; index <= 70; index++) {
            store.recordError(index, "test", "error-" + index);
        }

        List<ClientError> errors = store.errors(64);
        assertEquals(64, errors.size());
        assertEquals(7, errors.get(0).clientTick());
        assertEquals(70, errors.get(63).clientTick());

        ClientObservationStore redactionStore = new ClientObservationStore();
        redactionStore.recordError(
                1,
                "connection",
                "path=" + home + "/game authorization: secret-value access_token=another");
        String redacted = redactionStore.errors(1).get(0).message();
        assertTrue(redacted.contains("<user-home>"));
        assertFalse(redacted.contains("secret-value"));
        assertFalse(redacted.contains("another"));
        assertThrows(IllegalArgumentException.class, () -> store.errors(0));
        assertThrows(IllegalArgumentException.class, () -> store.errors(65));
    }

    private static ClientSnapshot snapshot(long tick) {
        ClientSnapshot starting = ClientSnapshot.starting();
        return new ClientSnapshot(
                tick,
                starting.connection(),
                starting.player(),
                starting.inventory(),
                starting.crosshair(),
                starting.screen(),
                starting.nearbyBlocks());
    }
}
