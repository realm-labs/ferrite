package dev.ferrite.client.mcp.mixin;

import net.minecraft.client.KeyMapping;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.gen.Accessor;

/** Narrow bridge for injecting one ordinary key-click into Minecraft's own handler. */
@Mixin(KeyMapping.class)
public interface KeyMappingAccessor {
    @Accessor("clickCount")
    int ferrite$getClickCount();

    @Accessor("clickCount")
    void ferrite$setClickCount(int clickCount);
}
