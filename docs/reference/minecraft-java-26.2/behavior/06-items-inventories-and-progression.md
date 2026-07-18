# 06 — Items, Containers, Processing, and Progression

Concrete items, recipes, loot tables, enchantments, advancements, and component defaults come from `OFF-DATA-001` / `OFF-REPORT-001`. This page specifies their shared state machines.

## `ITM-001` ItemStack is a value of item type, count, and data components

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.item.ItemStack`; `net.minecraft.core.component.DataComponentMap`; `net.minecraft.core.component.DataComponents`; `COM-WIKI-ITM-001`
- **Applies when:** An item appears in a slot, world entity, machine input/output, or interaction context.
- **Behavior and timing:** A nonempty stack contains at least a registered item, positive count, and valid data-component patch. Components carry composable semantics such as damage, custom name, food, tool, and equippable data. Stacking compares more than item ID: components relevant to stacking must agree and the maximum count must be respected.
- **Boundaries and quirks:** A zero-count stack normalizes to empty semantics. Mutable `ItemStack` operations must dirty and synchronize their owning container/entity. Do not keep a second NBT-like field copy that can diverge.
- **Verification owner (`ITM-USE-001`; `EXP-ITM-*`):** Generate default-component and max-stack conformance tables from per-item reports, and lock component-patch equality/serialization boundaries.

## `ITM-002` The server replays menu actions and corrects clients with slot snapshots

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-CLIENT-001`; `net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClick(net.minecraft.network.protocol.game.ServerboundContainerClickPacket)`; `net.minecraft.world.inventory.AbstractContainerMenu#clicked(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`; `net.minecraft.world.inventory.AbstractContainerMenu#doClick(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`; `net.minecraft.world.inventory.AbstractContainerMenu#moveItemStackTo(net.minecraft.world.item.ItemStack,int,int,boolean)`; `net.minecraft.world.inventory.AbstractContainerMenu#broadcastChanges()`; `net.minecraft.client.multiplayer.MultiPlayerGameMode#handleContainerInput(int,int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`
- **Applies when:** A player clicks, shift-clicks, drags, swaps, throws, or picks up a slot in an open menu.
- **Behavior and timing:** The client immediately replays the generic click and sends hashes of predicted changed slots/cursor with the pre-click 15-bit state ID. After identity, player-state, validity and slot admission, the server suppresses remote comparisons, replays the click on current authoritative state, installs the submitted hashes only as remote comparison baselines, then resumes synchronization. A current state ID produces per-slot/cursor/data deltas; a stale ID still commits the click and then produces a full authoritative snapshot. Slot deltas increment the state ID independently. Quick-move uses the menu's locked half-open ranges and may repeat for a replenishing result slot.
- **Boundaries and quirks:** The cursor is menu state, not an inventory slot. Quick-craft is a three-header state machine spanning packets. Merge and empty-placement are separate passes: occupied same-component stacks can merge without `mayPlace`, while empty destinations require it. Closing resolves cursor and transient inputs before transferring matching remote snapshots back to the inventory menu; the server does not validate the close packet's container ID.
- **Verification owner (`ITM-CONTAINER-001`, `ITM-CONTAINER-CLICK-001`, `ITM-CONTAINER-MOVE-001`, `ITM-CONTAINER-CONTROL-001`, `ITM-CONTAINER-CLOSE-001`; `EXP-ITM-002`):** The generic algorithms and all 25 registered menu layouts are source-specified in the container leaf page. Keep the experiment as a latency/concurrency regression probe, especially for stale-state commit-plus-full-correction and delayed old-menu close.

## `ITM-003` Use, consumption, durability, and cooldown form server-committed item transactions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-SERVER-001`; `net.minecraft.world.item.ItemStack#useOn(net.minecraft.world.item.context.UseOnContext)`; `net.minecraft.world.item.ItemStack#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`; `net.minecraft.world.item.ItemStack#consume(int,net.minecraft.world.entity.LivingEntity)`; `net.minecraft.world.item.ItemStack#hurtAndBreak(int,net.minecraft.server.level.ServerLevel,net.minecraft.server.level.ServerPlayer,java.util.function.Consumer)`; `net.minecraft.world.item.ItemCooldowns#isOnCooldown(net.minecraft.world.item.ItemStack)`
- **Applies when:** Interaction priority reaches item use, or an action consumes count/durability and may set cooldown.
- **Behavior and timing:** `InteractionResult` selects success, fallback, and swing behavior. Continuous use enters a using-item state, advances against use duration each tick, then invokes finish/release/interruption logic. `consume` changes count. `hurtAndBreak` changes the damage component and, at its limit, invokes a break callback and shrinks/replaces the stack. Cooldown advances on the player tick by item/cooldown group and blocks corresponding use.
- **Boundaries and quirks:** Creative abilities may skip consumption without skipping every side effect. Enchantments such as Unbreaking may alter actual durability loss. Hand changes, death, or component mutation while using can interrupt or resynchronize.
- **Verification owner (`ITM-USE-001`; `EXP-ITM-*`):** Lock continuous-use completion tick, release point, damage RNG, and post-break container synchronization order.

## `ITM-004` Crafting matches a recipe, then atomically consumes input and creates remainders

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.item.crafting.RecipeManager#prepare(net.minecraft.server.packs.resources.ResourceManager,net.minecraft.util.profiling.ProfilerFiller)`; `net.minecraft.world.item.crafting.RecipeManager#getRecipeFor(net.minecraft.world.item.crafting.RecipeType,net.minecraft.world.item.crafting.RecipeInput,net.minecraft.world.level.Level,net.minecraft.world.item.crafting.RecipeHolder)`; `net.minecraft.world.item.crafting.RecipeSerializers#bootstrap(net.minecraft.core.Registry)`; `net.minecraft.world.inventory.CraftingMenu#slotChangedCraftingGrid(net.minecraft.world.inventory.AbstractContainerMenu,net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.player.Player,net.minecraft.world.inventory.CraftingContainer,net.minecraft.world.inventory.ResultContainer,net.minecraft.world.item.crafting.RecipeHolder)`; `net.minecraft.world.inventory.ResultSlot#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`
- **Applies when:** A crafting grid changes or a player takes/shift-clicks its result.
- **Behavior and timing:** Reload decodes recipes in key order and partitions them into seven type domains. A retained recipe that still matches wins before the first key-ordered match. The 21 serializers then apply their audited shape/allocation/component rules. Grid change assembles a non-consuming preview after limited-crafting and feature gates. Taking credits that stored recipe, freshly resolves take-time remainders, consumes one item in every cropped participating cell, and places each remainder into its cell, inventory, or a non-random drop.
- **Boundaries and quirks:** The credited preview recipe and freshly matched remainder recipe can differ after mutation. Shaped input is cropped before mirrored/ordinary matching; shapeless allocation ignores components unless a special serializer adds a component gate. Client recipe displays and recipe-book categories cannot authorize crafting.
- **Verification owner (`ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`; `EXP-ITM-003`):** All 21 serializer IDs and the manual commit are source-specified. Keep the experiment for callback mutation and shift-craft regression while ticked processors remain separately owned.

## `ITM-005` Furnaces and brewing stands advance separate timers only in valid input states

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-SERVER-001`; `net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#serverTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity)`; `net.minecraft.world.level.block.entity.BrewingStandBlockEntity#serverTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BrewingStandBlockEntity)`; `COM-WIKI-ITM-001`
- **Applies when:** The block entity is in a ticking chunk and has input, fuel/fuel value, or an in-progress operation.
- **Behavior and timing:** A furnace validates recipe and output room each block-entity tick, managing burn time and cooking progress. Fuel may ignite when required; progress advances only in a processable state. Completion atomically consumes input, creates output, and records experience. A brewing stand validates three bottle slots, ingredient, and fuel, then counts down. At completion it applies the brewing recipe to still-matching slots, consumes ingredient/remainder, and updates state.
- **Boundaries and quirks:** Mid-process input/output changes can pause, regress, or cancel concrete timers. Unloaded chunks do not perform wall-time offline catch-up. Hoppers interact with machine slots on their own block-entity timing.
- **Verification owner (`ITM-CRAFT-001`; `EXP-ITM-003`):** Extract per-machine progress regression, fuel-consumption point, XP rounding, and same-tick hopper order into a table from source/data.

## `ITM-006` Enchanting and loot use registry data plus contextual random selection

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`
- **Primary evidence:** `OFF-DATA-001`; `OFF-SERVER-001`; `net.minecraft.world.item.enchantment.EnchantmentHelper#selectEnchantment(net.minecraft.util.RandomSource,net.minecraft.world.item.ItemStack,int,java.util.stream.Stream)`; `net.minecraft.world.item.enchantment.EnchantmentHelper#modifyDamage(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.Entity,net.minecraft.world.damagesource.DamageSource,float)`; `net.minecraft.world.level.storage.loot.LootTable#getRandomItems(net.minecraft.world.level.storage.loot.LootParams,net.minecraft.util.RandomSource)`; `net.minecraft.world.level.storage.loot.LootTable#fill(net.minecraft.world.Container,net.minecraft.world.level.storage.loot.LootParams,long)`; `COM-WIKI-ITM-001`
- **Applies when:** An enchanting table, loot source, entity drop, or related system computes effects/results from data.
- **Behavior and timing:** Enchantment selection takes item, level, enchantability, compatibility, registry candidates, and an explicit `RandomSource`; resulting equipment components and enchantment-effect data join gameplay hooks. A loot table builds context from a `LootParams` set such as luck, origin, tool, or killer, evaluates pools/entries/conditions/functions into stacks, then applies container fill rules when needed.
- **Boundaries and quirks:** A missing required context parameter should fail rather than silently become zero. The same table can legitimately differ by caller context. Explicit seed overrides and function order are observable.
- **Verification owner (`ITM-LOOT-001`, `ITM-ENCHANT-001`; `EXP-ITM-004`, `EXP-ITM-005`):** Add data-driven tests for every context set/table type, explicit seed, and enchantment compatibility conflict without copying table contents here.

## `ITM-007` Hunger, experience, and advancements are three independent server progression systems

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`
- **Primary evidence:** `OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.food.FoodData#eat(int,float)`; `net.minecraft.world.food.FoodData#tick(net.minecraft.server.level.ServerPlayer)`; `net.minecraft.world.entity.player.Player#giveExperiencePoints(int)`; `net.minecraft.server.PlayerAdvancements#award(net.minecraft.advancements.AdvancementHolder,java.lang.String)`; `net.minecraft.server.PlayerAdvancements#flushDirty(net.minecraft.server.level.ServerPlayer,boolean)`; `COM-WIKI-ITM-001`
- **Applies when:** A player eats or exerts, gains/loses experience, or satisfies an advancement criterion.
- **Behavior and timing:** `FoodData` separately stores food level, saturation, and exhaustion. Activity adds exhaustion; thresholds convert it into saturation/food loss, after which difficulty and health decide regeneration or starvation damage. Experience points, level, and progress convert across total-point boundaries. Server triggers award individual advancement criteria; completing requirements finishes the advancement, synchronizes it, and grants rewards.
- **Boundaries and quirks:** Peaceful difficulty, `naturalRegeneration`, food component effects, keep/drop XP on death, and advancement reload add branches. These states must not collapse into one “player level.”
- **Verification owner (`ITM-ADVANCEMENT-001`; `EXP-ITM-006`):** Hunger and experience still require dedicated leaf rules; advancement trigger order remains in this leaf until split.
