package dev.ferrite.client.mcp.observation;

import dev.ferrite.client.mcp.observation.ClientSnapshot.Block;
import dev.ferrite.client.mcp.observation.ClientSnapshot.BlockPosition;
import dev.ferrite.client.mcp.observation.ClientSnapshot.BlockTarget;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Connection;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Crosshair;
import dev.ferrite.client.mcp.observation.ClientSnapshot.EntityTarget;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Inventory;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Item;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Menu;
import dev.ferrite.client.mcp.observation.ClientSnapshot.NearbyBlocks;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Player;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Point;
import dev.ferrite.client.mcp.observation.ClientSnapshot.Screen;
import java.util.ArrayList;
import java.util.List;
import net.minecraft.client.Minecraft;
import net.minecraft.client.gui.screens.inventory.AbstractContainerScreen;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.client.multiplayer.ServerData;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.DisconnectionDetails;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.player.Abilities;
import net.minecraft.world.inventory.AbstractContainerMenu;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.EntityHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec3;

/** Copies bounded Minecraft state at the end of each client tick. */
public final class MinecraftObservationCollector {
    private static final int NEARBY_RADIUS = 2;

    private final ClientObservationStore store;
    private long clientTick;
    private String recordedDisconnectReason;

    public MinecraftObservationCollector(ClientObservationStore store) {
        this.store = store;
    }

    public void capture(Minecraft client) {
        clientTick++;
        if (!client.isSameThread()) {
            store.recordError(clientTick, "observation", "observation attempted off the client thread");
            return;
        }
        try {
            recordDisconnect(client.getConnection());
            LocalPlayer player = client.player;
            store.publish(new ClientSnapshot(
                    clientTick,
                    connection(client),
                    player == null || client.level == null ? null : player(client, player),
                    inventory(player),
                    crosshair(client),
                    screen(client),
                    nearbyBlocks(client, player)));
        } catch (RuntimeException error) {
            store.recordError(
                    clientTick,
                    "observation",
                    error.getClass().getSimpleName() + ": " + error.getMessage());
        }
    }

    private Connection connection(Minecraft client) {
        ClientPacketListener listener = client.getConnection();
        String state;
        if (listener == null) {
            state = "DISCONNECTED";
        } else if (client.player != null && client.level != null) {
            state = "PLAY";
        } else if (listener.getConnection().isConnecting()) {
            state = "CONNECTING";
        } else if (listener.getConnection().isConnected()) {
            state = "CONNECTED";
        } else {
            state = "DISCONNECTED";
        }

        ServerData serverData = listener == null ? null : listener.getServerData();
        if (serverData == null) {
            serverData = client.getCurrentServer();
        }
        String address = serverData == null ? null : SensitiveText.redact(serverData.ip);
        Long ping = serverData == null || serverData.ping < 0 ? null : serverData.ping;
        Integer protocol = serverData == null ? null : serverData.protocol;
        return new Connection(state, address, client.isLocalServer(), ping, protocol);
    }

    private static Player player(Minecraft client, LocalPlayer player) {
        Vec3 velocity = player.getDeltaMovement();
        Abilities abilities = player.getAbilities();
        return new Player(
                true,
                finite(player.getX(), "player x"),
                finite(player.getY(), "player y"),
                finite(player.getZ(), "player z"),
                finite(player.getYRot(), "player yaw"),
                finite(player.getXRot(), "player pitch"),
                finite(velocity.x(), "player velocity x"),
                finite(velocity.y(), "player velocity y"),
                finite(velocity.z(), "player velocity z"),
                player.onGround(),
                player.isAlive(),
                player.isSprinting(),
                player.isCrouching(),
                finite(player.getHealth(), "player health"),
                finite(player.getMaxHealth(), "player max health"),
                player.getFoodData().getFoodLevel(),
                finite(player.getFoodData().getSaturationLevel(), "player saturation"),
                player.gameMode().getName(),
                client.level.dimension().identifier().toString(),
                abilities.flying,
                abilities.mayfly);
    }

    private static Inventory inventory(LocalPlayer player) {
        if (player == null) {
            return new Inventory(false, -1, List.of());
        }
        net.minecraft.world.entity.player.Inventory inventory = player.getInventory();
        List<Item> items = new ArrayList<>();
        for (int slot = 0; slot < inventory.getContainerSize(); slot++) {
            ItemStack stack = inventory.getItem(slot);
            if (!stack.isEmpty()) {
                items.add(item(slot, stack));
            }
        }
        return new Inventory(true, inventory.getSelectedSlot(), items);
    }

    private static Crosshair crosshair(Minecraft client) {
        HitResult hit = client.hitResult;
        if (hit == null) {
            return new Crosshair("NONE", null, null, null);
        }
        Vec3 location = hit.getLocation();
        Point point = new Point(
                finite(location.x(), "crosshair x"),
                finite(location.y(), "crosshair y"),
                finite(location.z(), "crosshair z"));
        if (hit instanceof BlockHitResult blockHit) {
            BlockPos position = blockHit.getBlockPos();
            String blockId = blockIdIfLoaded(client, position);
            return new Crosshair(
                    hit.getType().name(),
                    point,
                    new BlockTarget(
                            position.getX(),
                            position.getY(),
                            position.getZ(),
                            blockHit.getDirection().getName(),
                            blockId),
                    null);
        }
        if (hit instanceof EntityHitResult entityHit) {
            Entity entity = entityHit.getEntity();
            return new Crosshair(
                    hit.getType().name(),
                    point,
                    null,
                    new EntityTarget(
                            entity.getId(),
                            BuiltInRegistries.ENTITY_TYPE.getKey(entity.getType()).toString()));
        }
        return new Crosshair(hit.getType().name(), point, null, null);
    }

    private static Screen screen(Minecraft client) {
        net.minecraft.client.gui.screens.Screen current = client.gui.screen();
        String overlay = client.gui.overlay() == null
                ? null
                : client.gui.overlay().getClass().getSimpleName();
        if (current == null) {
            return new Screen("NONE", null, false, 0, 0, overlay, null);
        }

        Menu menu = null;
        if (current instanceof AbstractContainerScreen<?> containerScreen) {
            menu = menu(containerScreen.getMenu());
        }
        return new Screen(
                current.getClass().getSimpleName(),
                SensitiveText.redact(current.getTitle().getString()),
                current.isPauseScreen(),
                current.width,
                current.height,
                overlay,
                menu);
    }

    private static Menu menu(AbstractContainerMenu menu) {
        List<Item> populated = new ArrayList<>();
        for (int slot = 0; slot < menu.slots.size(); slot++) {
            ItemStack stack = menu.getSlot(slot).getItem();
            if (!stack.isEmpty()) {
                populated.add(item(slot, stack));
            }
        }
        Item carried = menu.getCarried().isEmpty() ? null : item(-1, menu.getCarried());
        return new Menu(
                menu.containerId, menu.getStateId(), menu.slots.size(), carried, populated);
    }

    private static NearbyBlocks nearbyBlocks(Minecraft client, LocalPlayer player) {
        if (player == null || client.level == null) {
            return new NearbyBlocks(false, NEARBY_RADIUS, null, false, List.of());
        }

        BlockPos center = player.blockPosition();
        List<Block> blocks = new ArrayList<>();
        boolean complete = true;
        for (int y = center.getY() - NEARBY_RADIUS;
                y <= center.getY() + NEARBY_RADIUS;
                y++) {
            for (int z = center.getZ() - NEARBY_RADIUS;
                    z <= center.getZ() + NEARBY_RADIUS;
                    z++) {
                for (int x = center.getX() - NEARBY_RADIUS;
                        x <= center.getX() + NEARBY_RADIUS;
                        x++) {
                    BlockPos position = new BlockPos(x, y, z);
                    if (client.level.isOutsideBuildHeight(position)
                            || !hasLoadedChunk(client, x, z)) {
                        complete = false;
                        continue;
                    }
                    BlockState state = client.level.getBlockState(position);
                    if (!state.isAir()) {
                        blocks.add(new Block(
                                x,
                                y,
                                z,
                                BuiltInRegistries.BLOCK.getKey(state.getBlock()).toString(),
                                state.toString()));
                    }
                }
            }
        }
        return new NearbyBlocks(
                true,
                NEARBY_RADIUS,
                new BlockPosition(center.getX(), center.getY(), center.getZ()),
                complete,
                blocks);
    }

    private void recordDisconnect(ClientPacketListener listener) {
        if (listener == null || listener.getConnection().isConnected()) {
            recordedDisconnectReason = null;
            return;
        }
        DisconnectionDetails details = listener.getConnection().getDisconnectionDetails();
        if (details == null) {
            return;
        }
        String reason = SensitiveText.redact(details.reason().getString());
        if (!reason.equals(recordedDisconnectReason)) {
            store.recordError(clientTick, "connection", reason);
            recordedDisconnectReason = reason;
        }
    }

    private static Item item(int slot, ItemStack stack) {
        return new Item(
                slot,
                BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(),
                stack.getCount(),
                stack.getDamageValue(),
                stack.getMaxDamage());
    }

    private static String blockIdIfLoaded(Minecraft client, BlockPos position) {
        if (client.level == null
                || client.level.isOutsideBuildHeight(position)
                || !hasLoadedChunk(client, position.getX(), position.getZ())) {
            return null;
        }
        return BuiltInRegistries.BLOCK
                .getKey(client.level.getBlockState(position).getBlock())
                .toString();
    }

    private static boolean hasLoadedChunk(Minecraft client, int blockX, int blockZ) {
        return client.level.hasChunk(
                SectionPos.blockToSectionCoord(blockX), SectionPos.blockToSectionCoord(blockZ));
    }

    private static double finite(double value, String field) {
        if (!Double.isFinite(value)) {
            throw new IllegalStateException(field + " is not finite");
        }
        return value;
    }

    private static float finite(float value, String field) {
        if (!Float.isFinite(value)) {
            throw new IllegalStateException(field + " is not finite");
        }
        return value;
    }
}
