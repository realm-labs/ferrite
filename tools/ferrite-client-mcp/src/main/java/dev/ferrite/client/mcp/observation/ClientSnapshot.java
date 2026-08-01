package dev.ferrite.client.mcp.observation;

import java.util.List;
import java.util.Objects;

/** Immutable state copied from Minecraft on the client thread at one tick boundary. */
public record ClientSnapshot(
        long clientTick,
        Connection connection,
        Player player,
        World world,
        Inventory inventory,
        Crosshair crosshair,
        Screen screen,
        NearbyBlocks nearbyBlocks) {
    public ClientSnapshot {
        if (clientTick < 0) {
            throw new IllegalArgumentException("client tick must be non-negative");
        }
        Objects.requireNonNull(connection, "connection");
        Objects.requireNonNull(world, "world");
        Objects.requireNonNull(inventory, "inventory");
        Objects.requireNonNull(crosshair, "crosshair");
        Objects.requireNonNull(screen, "screen");
        Objects.requireNonNull(nearbyBlocks, "nearbyBlocks");
    }

    public static ClientSnapshot starting() {
        return new ClientSnapshot(
                0,
                new Connection("STARTING", null, false, null, null),
                null,
                new World(false, null, 0, 0, 0.0f, 0.0f, false, false),
                new Inventory(false, -1, List.of()),
                new Crosshair("NONE", null, null, null),
                new Screen("NONE", null, false, 0, 0, null, null),
                new NearbyBlocks(false, 2, null, false, List.of()));
    }

    public record Connection(
            String state,
            String serverAddress,
            boolean localServer,
            Long pingMillis,
            Integer serverProtocol) {}

    public record Player(
            boolean available,
            double x,
            double y,
            double z,
            float yaw,
            float pitch,
            double velocityX,
            double velocityY,
            double velocityZ,
            boolean onGround,
            boolean alive,
            boolean sprinting,
            boolean crouching,
            float health,
            float maxHealth,
            int food,
            float saturation,
            String gameMode,
            String dimension,
            boolean flying,
            boolean mayFly) {}

    public record World(
            boolean available,
            String dimension,
            long overworldClockTime,
            long defaultClockTime,
            float rainLevel,
            float thunderLevel,
            boolean raining,
            boolean thundering) {}

    public record Inventory(boolean available, int selectedHotbarSlot, List<Item> items) {
        public Inventory {
            items = List.copyOf(items);
        }
    }

    public record Item(int slot, String itemId, int count, int damage, int maxDamage) {}

    public record Crosshair(String kind, Point location, BlockTarget block, EntityTarget entity) {}

    public record Point(double x, double y, double z) {}

    public record BlockTarget(int x, int y, int z, String face, String blockId) {}

    public record EntityTarget(int entityId, String entityType) {}

    public record Screen(
            String type,
            String title,
            boolean pausesGame,
            int width,
            int height,
            String overlayType,
            Menu menu) {}

    public record Menu(
            int containerId,
            int stateId,
            int slotCount,
            Item carried,
            List<Item> populatedSlots) {
        public Menu {
            populatedSlots = List.copyOf(populatedSlots);
        }
    }

    public record NearbyBlocks(
            boolean available,
            int radius,
            BlockPosition center,
            boolean complete,
            List<Block> blocks) {
        public NearbyBlocks {
            blocks = List.copyOf(blocks);
        }
    }

    public record BlockPosition(int x, int y, int z) {}

    public record Block(int x, int y, int z, String blockId, String state) {}
}
