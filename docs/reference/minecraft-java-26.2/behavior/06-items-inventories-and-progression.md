# 06 — Items, Containers, Processing, and Progression

Concrete items, recipes, loot tables, enchantments, advancements, and component defaults come from
`OFF-DATA-001` / `OFF-REPORT-001`. This page specifies their shared state machines.

## `ITM-001` ItemStack is a value of item type, count, and data components

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `OFF-REPORT-001`; `net.minecraft.world.item.ItemStack`;
`net.minecraft.core.component.DataComponentMap`; `net.minecraft.core.component.DataComponents`;
`COM-WIKI-ITM-001`

### Applies when

An item appears in a slot, world entity, machine input/output, or interaction context.

### Behavior and timing

A nonempty stack contains at least a registered item, positive count, and valid data-component
patch. Components carry composable semantics such as damage, custom name, food, tool, and equippable
data. Stacking compares more than item ID: components relevant to stacking must agree and the
maximum count must be respected.

### Boundaries and quirks

A zero-count stack normalizes to empty semantics. Mutable `ItemStack` operations must dirty and
synchronize their owning container/entity. Do not keep a second NBT-like field copy that can
diverge.

### Verification

**Owners:** `ITM-USE-001`, `ITM-BOOKSHELF-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-LECTERN-001`,
`BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`, `BLK-SIGN-001`, `BLK-SKULL-001`,
`ITM-HONEYCOMB-001`; `EXP-ITM-*`, `EXP-BLK-008`, `EXP-BLK-011`, `EXP-BLK-012`,
`EXP-BLK-013`, `EXP-BLK-014`, `EXP-BLK-025`, `EXP-BLK-026`

The concrete leaves fix banner/shelf/pot component projections plus the prior bookshelf, lectern and
statue transfers through placement, pick, loot and rendering. Generate the remaining
default-component and max-stack conformance tables from per-item reports, and lock component-patch
equality/serialization boundaries.
`BLK-SKULL-001` fixes the seven head-slot item defaults, player-head profile-dependent name and
three implicit block-entity components, including the six non-player loot tables that copy only name.
The sign leaf fixes all 24 sign items at maximum stack 16 plus dye-component dispatch; the honeycomb
leaf fixes its common stack-64 item and proves that use-time copper mapping is code-built, not an
item component or recipe lookup.

## `ITM-002` The server replays menu actions and corrects clients with slot snapshots

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `OFF-CLIENT-001`;
`net.minecraft.server.network.ServerGamePacketListenerImpl#handleContainerClick(net.minecraft.network.protocol.game.ServerboundContainerClickPacket)`;
`net.minecraft.world.inventory.AbstractContainerMenu#clicked(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.inventory.AbstractContainerMenu#doClick(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`;
`net.minecraft.world.inventory.AbstractContainerMenu#moveItemStackTo(net.minecraft.world.item.ItemStack,int,int,boolean)`;
`net.minecraft.world.inventory.AbstractContainerMenu#broadcastChanges()`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#handleContainerInput(int,int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`

### Applies when

A player clicks, shift-clicks, drags, swaps, throws, or picks up a slot in an open menu.

### Behavior and timing

The client immediately replays the generic click and sends hashes of predicted changed slots/cursor
with the pre-click 15-bit state ID. After identity, player-state, validity and slot admission, the
server suppresses remote comparisons, replays the click on current authoritative state, installs the
submitted hashes only as remote comparison baselines, then resumes synchronization. A current state
ID produces per-slot/cursor/data deltas; a stale ID still commits the click and then produces a full
authoritative snapshot. Slot deltas increment the state ID independently. Quick-move uses the menu's
locked half-open ranges and may repeat for a replenishing result slot.

### Boundaries and quirks

The cursor is menu state, not an inventory slot. Quick-craft is a three-header state machine
spanning packets. Merge and empty-placement are separate passes: occupied same-component stacks can
merge without `mayPlace`, while empty destinations require it. Closing resolves cursor and transient
inputs before transferring matching remote snapshots back to the inventory menu; the server does not
validate the close packet's container ID.

### Verification

**Owners:** `ITM-CONTAINER-001`, `ITM-CONTAINER-CLICK-001`, `ITM-CONTAINER-MOVE-001`,
`ITM-CONTAINER-CONTROL-001`, `ITM-CONTAINER-CLOSE-001`, `ITM-DROPPER-001`, `ITM-ENDER-CHEST-001`,
`ITM-BARREL-001`, `BLK-LECTERN-001`, `BLK-BEACON-001`; `EXP-ITM-002`, `EXP-ITM-007`,
`EXP-ITM-008`, `EXP-ITM-009`, `EXP-BLK-011`, `EXP-BLK-024`

The generic algorithms and all 25 registered menu layouts are source-specified in the container leaf
page. Device leaves separately own dropper output, player-owned ender storage, barrel-owned
storage/open/removal transactions, and lectern page/take controls. Keep the experiments as
regression probes for transaction ordering and stale-state boundaries.
The beacon leaf joins its already specified layout/control/close owners to live pyramid level,
reloadable payment admission, exact power validation, payment consumption and dirty-without-update
ordering.

## `ITM-003` Use, consumption, durability, and cooldown form server-committed item transactions

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.item.ItemStack#useOn(net.minecraft.world.item.context.UseOnContext)`;
`net.minecraft.world.item.ItemStack#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`;
`net.minecraft.world.item.ItemStack#consume(int,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.item.ItemStack#hurtAndBreak(int,net.minecraft.server.level.ServerLevel,net.minecraft.server.level.ServerPlayer,java.util.function.Consumer)`;
`net.minecraft.world.item.ItemCooldowns#isOnCooldown(net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.inventory.CartographyTableMenu#setupResultSlot`;
`net.minecraft.world.inventory.LoomMenu#setupResultSlot`;
`net.minecraft.world.inventory.GrindstoneMenu#computeResult`;
`net.minecraft.world.inventory.AnvilMenu#createResult`

### Applies when

Interaction priority reaches item use, an action consumes count/durability and may set cooldown, or
a non-recipe workstation commits a component transformation.

### Behavior and timing

`InteractionResult` selects success, fallback, and swing behavior. Continuous use enters a
using-item state, advances against use duration each tick, then invokes finish/release/interruption
logic. `consume` changes count. `hurtAndBreak` changes the damage component and, at its limit,
invokes a break callback and shrinks/replaces the stack. Cooldown advances on the player tick by
item/cooldown group and blocks corresponding use. Cartography consumes map plus material before
deferred scale/lock map-data allocation; loom consumes banner plus dye while appending a selected
registry pattern and retaining its optional selector item; grindstone takes one or two stacks,
retains only curses, repairs compatible damageable pairs and grants XP from removed enchantments;
anvil preview combines prior-work, repair, enchantment and rename costs before a level-paid commit
that can damage the workstation.

### Boundaries and quirks

Creative abilities may skip consumption without skipping every side effect. Cartography
creative-clone post-processes a fresh result without consuming inputs, and its unresolved-map setup
can retain a stale preview. Loom remaps selection by holder identity, enforces six layers and does
not consume the pattern item. Grindstone merge output inherits the first input's non-enchantment
components, and a lone unenchanted damageable item produces no result. Anvil-only rename is allowed
at effective cost `39` even when accumulated prior work reaches `40`, preserves the added input and
does not increase the output's prior-work cost. Enchantments such as Unbreaking may alter actual
durability loss. Hand changes, death, or component mutation while using can interrupt or
resynchronize.

### Verification

**Owners:** `ITM-USE-001`, `ITM-BOOKSHELF-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-LECTERN-001`,
`BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`, `ITM-CARTOGRAPHY-001`,
`BLK-BRUSHABLE-001`, `BLK-SIGN-001`, `BLK-SKULL-001`, `ITM-HONEYCOMB-001`, `ITM-LOOM-001`,
`ITM-GRINDSTONE-001`, `ITM-ANVIL-001`; `EXP-ITM-*`, `EXP-BLK-008`, `EXP-BLK-011`,
`EXP-BLK-012`, `EXP-BLK-013`, `EXP-BLK-014`, `EXP-BLK-019`, `EXP-BLK-025`, `EXP-BLK-026`

The concrete leaves fix immediate item/block transactions, including shelf's creative single-slot
duplication and powered 3N hotbar exchange. All four non-recipe workstation transforms are
source-specified. The remaining use slice owns continuous-use completion tick, release point, damage
RNG and post-break synchronization.
`BLK-BRUSHABLE-001` fixes brush-specific continuous pulses and applies one durability point only
after the tenth accepted block-entity stroke.
`BLK-SIGN-001` fixes sign placement and applicator consumption through the living-entity-aware
helper, while `ITM-HONEYCOMB-001` deliberately uses direct `shrink(1)` on mapped copper states,
ignores the state-write result and still emits success effects.
`BLK-SKULL-001` fixes player-profile filling, break-loot component whitelists and the player-head
name fallback while generic stack custom-name precedence remains here.

## `ITM-004` Crafting matches a recipe, then atomically consumes input and creates remainders

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-SERVER-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.crafting.RecipeManager#prepare(net.minecraft.server.packs.resources.ResourceManager,net.minecraft.util.profiling.ProfilerFiller)`;
`net.minecraft.world.item.crafting.RecipeManager#getRecipeFor(net.minecraft.world.item.crafting.RecipeType,net.minecraft.world.item.crafting.RecipeInput,net.minecraft.world.level.Level,net.minecraft.world.item.crafting.RecipeHolder)`;
`net.minecraft.world.item.crafting.RecipeSerializers#bootstrap(net.minecraft.core.Registry)`;
`net.minecraft.world.inventory.CraftingMenu#slotChangedCraftingGrid(net.minecraft.world.inventory.AbstractContainerMenu,net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.player.Player,net.minecraft.world.inventory.CraftingContainer,net.minecraft.world.inventory.ResultContainer,net.minecraft.world.item.crafting.RecipeHolder)`;
`net.minecraft.world.inventory.ResultSlot#onTake(net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.inventory.StonecutterMenu#setupResultSlot`;
`net.minecraft.world.inventory.SmithingMenu#createResult`

### Applies when

A crafting grid or recipe workstation input changes, a stonecutter choice is selected, or a player
takes/shift-clicks a result.

### Behavior and timing

Reload decodes recipes in key order and partitions them into seven type domains. A retained recipe
that still matches wins before the first key-ordered match where the caller supports retention. The
21 serializers apply their audited shape/allocation/component rules. Manual crafting assembles a
gated non-consuming preview, then credits it, freshly resolves remainders and consumes the cropped
grid. Stonecutting exposes a feature-filtered key-ordered choice list and consumes one input per
selected result stack. Smithing always previews the first matching key-ordered recipe and, after
credit, consumes each occupied template/base/addition role.

### Boundaries and quirks

Manual credited preview and take-time remainder recipes can differ after mutation. Stonecutter
selection persists across count/component changes to the same input item. Smithing does not retain a
previous match, and its error flag requires all three roles occupied with an empty result. Shaped
input is cropped before mirrored/ordinary matching; shapeless allocation ignores components unless a
special serializer adds a component gate. Client recipe displays and recipe-book categories cannot
authorize a manual craft.

### Verification

**Owners:** `ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-STONECUTTER-001`,
`ITM-SMITHING-001`, `BLK-BANNER-001`, `BLK-DECORATED-POT-001`, `BLK-SLIME-001`,
`BLK-HONEY-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`, `BLK-CONCRETE-001`,
`BLK-TERRACOTTA-001`, `BLK-GLAZED-TERRACOTTA-001`, `BLK-QUARTZ-001`,
`BLK-SANDSTONE-001`, `BLK-STONE-VARIANT-001`, `BLK-STONE-BRICK-001`,
`BLK-BEACON-STORAGE-001`, `BLK-RAW-STORAGE-001`; `EXP-ITM-003`,
`EXP-BLK-012`, `EXP-BLK-014`, `EXP-BLK-035`, `EXP-BLK-036`, `EXP-BLK-037`, `EXP-BLK-038`,
`EXP-BLK-041`, `EXP-BLK-042`, `EXP-BLK-043`, `EXP-BLK-044`, `EXP-BLK-045`, `EXP-BLK-046`,
`EXP-BLK-047`, `EXP-BLK-048`, `EXP-BLK-049`

All 21 serializer IDs and the manual, stonecutter and smithing commits are source-specified. The
content leaves own stored/tooltip/rendered banner patterns and decorated-pot faces. Keep the
experiments for callback mutation, shift-repeat and result-destination regression.
The slime leaf fixes the shaped nine-ball-to-one-block and shapeless one-block-to-nine-ball records;
matching, grid consumption, remainder handling and reload publication remain with the recipe owners.
The honey leaf fixes four honey bottles to one block with four glass-bottle remainders and the
reverse one-block-plus-four-bottles to four honey bottles; allocation/publication remain generic.
The soul-sand leaf fixes the soul-fire ingredient tag used by four-soul-torch and soul-campfire
recipes plus the one-soul-sand/eight-ghast-tear dried-ghast recipe; matching, allocation and reload
publication remain with the recipe owners.
The concrete leaf fixes that finished blocks have no direct recipe: each color's reloadable recipe
returns eight paired powder blocks from four sand, four gravel and one dye, while the code-built
water transition supplies finished concrete. Matching and allocation remain generic.
The terracotta leaf fixes clay-to-plain and dyed-to-glazed smelting at the serializer's 200-tick
default, sixteen centered eight-block dye recipes, and host/wayfinder template duplication. It also
fixes sixteen level-four mason records and their two-of-33 no-duplicate trade set; recipe/furnace
commit, offer selection/pricing and villager lifecycle remain generic.
The glazed-terracotta leaf fixes the output-side identity/facing defaults for all sixteen
dyed-to-glazed smelts, their exact unlock records and the other sixteen level-four mason records.
Each recipe returns one matching block at 0.1 XP after the 200-tick default; each glazed offer
exchanges one emerald for one block with 12 uses, 15 XP and 0.05 reputation discount.
The quartz leaf fixes the five full-block values across their crafting, stonecutting and smelting
graph, including the shape-family stair/slab joins and exact unlock criteria. Smooth quartz smelts
from quartz block in 200 ticks for 0.1 XP. The level-five mason set draws two distinct offers from
exactly quartz block and pillar, so both one-emerald outputs appear with 12 uses, 30 XP and 0.05
reputation discount; matching, allocation, menus, pricing and restock remain generic.
The sandstone leaf fixes the symmetric yellow/red full-block craft, stonecut and smooth-smelting
graph, exact shape-family stair/slab/wall joins and recipe unlocks. It also fixes dune-template
duplication from seven diamonds, one yellow sandstone and one existing template to two templates.
Matching, allocation, menus, furnace progress, smithing-template use and publication remain
generic.
The stone-brick leaf fixes stone-to-bricks crafting/cutting, vine/moss-block mossing,
stone-bricks cracking, slab/chiseled conversion, the shape-family joins and eight-chiseled-block
lodestone input. It also fixes the guaranteed level-two mason chiseled-block sale; matching,
allocation, furnace/stonecutter menus, offer pricing and publication remain generic.
The beacon-storage leaf fixes five nine-to-one compression and one-to-nine decompression pairs,
the anvil's three-block input, all eleven alternative recipe-unlock paths and the optional
trade-rebalance armorer candidates that exchange one iron or diamond block. Matching, allocation,
offer selection/pricing and publication remain generic.
The raw-storage leaf fixes three nine-to-one compression and one-to-nine decompression pairs with
no recipe group, plus all six recipe/direct-inventory alternative unlocks. No other recipe, trade
or optional-pack record consumes these block identities; matching, allocation and publication
remain generic.
The stone-variant leaf fixes diorite from cobblestone/quartz, granite from diorite/quartz,
andesite from diorite/cobblestone, all three 2-by-2 polish recipes and their stonecutting
alternatives, exact shape-family stair/slab/wall joins and recipe unlocks. Its level-three mason
joins are three 16-raw-block purchases and three four-polished-block sales within the exact
two-of-seven set; matching, allocation, menus, offer selection/pricing and publication remain
generic.

## `ITM-005` Ticked processors validate their own timers, inputs, fuel and destinations

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-SERVER-001`;
`net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity#serverTick(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.AbstractFurnaceBlockEntity)`;
`net.minecraft.world.level.block.entity.CampfireBlockEntity#cookTick`;
`net.minecraft.world.level.block.entity.BrewingStandBlockEntity#serverTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BrewingStandBlockEntity)`;
`net.minecraft.world.level.block.CrafterBlock#dispenseFrom`

### Applies when

A furnace-family, campfire or brewing block entity ticks, or a powered crafter's scheduled tick
runs.

### Behavior and timing

Furnace-family ticks decrement lit time before recipe/output validation, may ignite and consume
fuel, advance one cook on that same tick, and commit input/output plus recipe-use accounting at the
deadline. Campfire slots advance independently while lit and cool by two while unlit. Brewing
refills its 20-use fuel counter before brewability, then runs a cancellable 400-tick same-ingredient
transaction over three bottles. A crafter rising edge schedules one attempt after four ticks;
success delivers result and remainders before consuming one from each participating input.

### Boundaries and quirks

Furnace invalid work resets progress while still spending existing burn time; player result take,
not hopper extraction, performs accumulated recipe/XP award. Campfire completion re-resolves its
recipe and falls back to the stored input if removed. Brewing may consume fuel with no brewable
input. A short crafter pulse is retained by the scheduled tick, and fully inserted output bypasses
dispense events and the nearby-player criterion. Unloaded block entities do not perform wall-time
catch-up.

### Verification

**Owners:** `ITM-FURNACE-001`, `ITM-CAMPFIRE-001`, `ITM-BREW-001`, `ITM-CRAFTER-001`,
`BLK-SLIME-001`; `EXP-ITM-003`, `EXP-BLK-035`

The four machine transactions, constants, slot policies and boundary branches are source-specified;
keep the experiment for callback ordering and external-container regression.
The slime leaf fixes its code-built start-mix edges: water to mundane and awkward to oozing when
the feature-filtered potion/item inputs are enabled; this parent retains brew admission and commit.

## `ITM-006` Enchanting and loot use registry data plus contextual random selection

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-DATA-001`; `OFF-SERVER-001`;
`net.minecraft.world.item.enchantment.EnchantmentHelper#selectEnchantment(net.minecraft.util.RandomSource,net.minecraft.world.item.ItemStack,int,java.util.stream.Stream)`;
`net.minecraft.world.item.enchantment.EnchantmentHelper#modifyDamage(net.minecraft.server.level.ServerLevel,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.Entity,net.minecraft.world.damagesource.DamageSource,float)`;
`net.minecraft.world.level.storage.loot.LootTable#getRandomItems(net.minecraft.world.level.storage.loot.LootParams,net.minecraft.util.RandomSource)`;
`net.minecraft.world.level.storage.loot.LootTable#fill(net.minecraft.world.Container,net.minecraft.world.level.storage.loot.LootParams,long)`;
`COM-WIKI-ITM-001`

### Applies when

An enchanting table, loot source, entity drop, or related system computes effects/results from data.

### Behavior and timing

Enchantment selection takes item, level, enchantability, compatibility, registry candidates, and an
explicit `RandomSource`; resulting equipment components and enchantment-effect data join gameplay
hooks. A loot table builds context from a `LootParams` set such as luck, origin, tool, or killer,
evaluates pools/entries/conditions/functions into stacks, then applies container fill rules when
needed.

### Boundaries and quirks

A missing required context parameter should fail rather than silently become zero. The same table
can legitimately differ by caller context. Explicit seed overrides and function order are
observable.

### Verification

**Owners:** `ITM-LOOT-001`, `ITM-ENCHANT-001`, `ITM-DROPPER-001`, `ITM-BARREL-001`,
`BLK-DECORATED-POT-001`, `BLK-BRUSHABLE-001`, `BLK-SOUL-SAND-001`, `BLK-MAGMA-001`,
`BLK-CONCRETE-001`, `BLK-TERRACOTTA-001`, `BLK-GLAZED-TERRACOTTA-001`,
`BLK-QUARTZ-001`, `BLK-SANDSTONE-001`, `BLK-STONE-VARIANT-001`,
`BLK-STONE-BRICK-001`, `BLK-BEACON-STORAGE-001`, `BLK-RAW-STORAGE-001`,
`BLK-LAVA-CAULDRON-001`;
`EXP-ITM-004`, `EXP-ITM-005`, `EXP-ITM-007`, `EXP-ITM-009`, `EXP-BLK-014`, `EXP-BLK-019`,
`EXP-BLK-037`, `EXP-BLK-038`, `EXP-BLK-039`, `EXP-BLK-041`, `EXP-BLK-042`,
`EXP-BLK-043`, `EXP-BLK-044`, `EXP-BLK-045`, `EXP-BLK-046`, `EXP-BLK-047`, `EXP-BLK-048`,
`EXP-BLK-049`

Device leaves fix dropper/barrel chest-context construction, stored-seed handoff and post-fill
dispatch; `ITM-LOOT-001` still owns the generic table evaluator and emitted stack sequence. Add
data-driven tests for every remaining context set/table type, explicit seed, and enchantment
compatibility conflict without copying table contents here.
`BLK-MAGMA-001` fixes the four-magma-cream shaped recipe, its unlock record, self loot, Frost
Walker hot-floor immunity selector and the hot sulfur-cube archetype constants; generic matching,
allocation, enchantment iteration and composed entity effects stay with their owners.
`BLK-LAVA-CAULDRON-001` fixes that state 9464 has no item mapping/clone stack but its block loot
returns one ordinary cauldron behind `survives_explosion`; generic loot evaluation and the ordinary
cauldron item's crafting/placement remain with their owners.
`BLK-CONCRETE-001` fixes sixteen correct-tool self-loot tables behind `survives_explosion` and the
item `concrete` tag's inclusion in the slow-bouncy sulfur archetype. Generic harvest admission,
loot evaluation, equipment matching and knockback stay with their existing owners.
`BLK-TERRACOTTA-001` fixes the corresponding seventeen self-loot tables and item `terracotta`
inclusion in slow-bouncy, while preserving exact identity across recipes and mason outputs. Generic
harvest, loot, archetype composition, inventory insertion and trade commit remain with their owners.
`BLK-GLAZED-TERRACOTTA-001` fixes sixteen correct-tool self-loot tables and item
`glazed_terracotta` inclusion in the same slow-bouncy archetype, while retaining color across
smelting and mason outputs. Generic harvest, loot, archetype composition and insertion remain owned
here.
`BLK-QUARTZ-001` fixes five correct-tool self-loot tables and direct inclusion of all five items
in the same slow-bouncy archetype, while preserving exact identities across processing and mason
outputs. Generic harvest, loot, archetype composition and insertion remain with their owners.
`BLK-SANDSTONE-001` fixes eight correct-tool self-loot tables, direct inclusion of all eight items
in the same slow-bouncy archetype, and carver/sculk replacement membership only for the two base
blocks. Exact full-block identities survive processing; generic harvest, loot, replacement,
archetype composition and insertion remain with their owners.
`BLK-STONE-VARIANT-001` fixes six correct-tool self-loot tables and direct inclusion of all six
items in slow-bouncy. Only the three raw identities enter base-stone and stone-ore replacement
tags; exact full-block identities survive processing and mason outputs, while generic harvest,
loot, replacement, archetype composition and insertion remain with their owners.
`BLK-STONE-BRICK-001` fixes four correct-tool self-loot tables, four Silk-Touch-only host outputs
from matching infested loot, the weight-2 stone-bricks village-mason chest candidate and direct
inclusion of all four items in slow-bouncy. Generic loot evaluation, infestation callbacks,
archetype composition and inventory insertion remain with their owners.
`BLK-BEACON-STORAGE-001` fixes five correct-tool self-loot tables, eight exact non-block
acquisition-table records, gold's loved-item membership, the 3/2 slow-flat/slow-bouncy item split
and netherite block's fire-resistant item component. Generic loot evaluation, piglin/archetype
state machines, damage reduction and inventory insertion remain with their owners.
`BLK-RAW-STORAGE-001` fixes three correct-tool self-loot tables, all three items' slow-flat
membership and raw gold's loved nonbarter role. No non-block loot table emits a family identity;
generic loot evaluation, piglin/archetype state machines and inventory insertion remain with their
owners.
The brushable leaf fixes the archaeology context, stored seed, zero/one/many-result selection and
first-item-only materialization before its first accepted count increment.
The soul-sand leaf fixes self loot, the weight-40/count-2..8 piglin barter entry and the
count-2..7 hoglin-stable chest entry, while `ITM-LOOT-001` retains pool selection, provider rounding,
luck and container insertion. It also fixes Soul Speed's tag-selected level-scaled attributes,
chance-first durability/sound effects and five-tick particle gate; generic enchantment iteration
and attribute lifetime remain with `ITM-ENCHANT-001`.

## `ITM-007` Hunger, experience, and advancements are three independent server progression systems

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.food.FoodData#eat(int,float)`;
`net.minecraft.world.food.FoodData#tick(net.minecraft.server.level.ServerPlayer)`;
`net.minecraft.world.entity.player.Player#giveExperiencePoints(int)`;
`net.minecraft.server.PlayerAdvancements#award(net.minecraft.advancements.AdvancementHolder,java.lang.String)`;
`net.minecraft.server.PlayerAdvancements#flushDirty(net.minecraft.server.level.ServerPlayer,boolean)`;
`COM-WIKI-ITM-001`

### Applies when

A player eats or exerts, gains/loses experience, or satisfies an advancement criterion.

### Behavior and timing

`FoodData` separately stores food level, saturation, and exhaustion. Activity adds exhaustion;
thresholds convert it into saturation/food loss, after which difficulty and health decide
regeneration or starvation damage. Experience points, level, and progress convert across total-point
boundaries. Server triggers award individual advancement criteria; completing requirements finishes
the advancement, synchronizes it, and grants rewards.

### Boundaries and quirks

Peaceful difficulty, `naturalRegeneration`, food component effects, keep/drop XP on death, and
advancement reload add branches. These states must not collapse into one “player level.”

### Verification

**Owners:** `ITM-ADVANCEMENT-001`, `BLK-BELL-001`, `ITM-HONEYCOMB-001`, `BLK-HONEY-001`;
`EXP-ITM-006`, `EXP-BLK-009`, `EXP-ITM-012`, `EXP-BLK-036`

Hunger and experience still require dedicated leaf rules; advancement trigger order remains in the
generic leaf, while the bell leaf fixes the exact successful-player/direct-or-projectile `bell_ring`
stat ingress.
The honeycomb leaf fixes `ITEM_USED_ON_BLOCK` before direct stack shrink and copper replacement;
unmapped states trigger neither criterion nor mutation.
The honey-block leaf fixes the 20-game-tick `HONEY_BLOCK_SLIDE` trigger cadence and live-state
predicate; generic requirement completion, persistence, rewards and telemetry remain here.
