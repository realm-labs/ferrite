# Content Dispatch Root Inventory

**Surface:** `SURFACE-CONTENT-DISPATCH-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`, `OFF-REPORT-001`, `OFF-DATA-001`

This inventory owns the point where a locked registry identity, implementation class, codec value,
tag, component or bundled-data record selects executable behavior. The catalog owns the exact
9,078-ID classification; the completion ledger separately owns all 95 registry scopes. Structural
one-owner coverage does not prove that a remaining subtype has no special control flow.

| Dispatch family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Blocks and block entities | `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#getBlock`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#tick`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#randomTick`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#entityInside`, `net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#updateShape`, `net.minecraft.world.level.block.entity.BlockEntityType#create` | `BLK-001`, `BLK-002`, `BLK-007`, `BLK-BRUSHABLE-001`, `BLK-SCULK-SENSOR-001`, `BLK-JIGSAW-001` and concrete block leaves own the virtual hooks, lifecycle and state schema. | Resolve 337 block and 12 block-entity fallback IDs to audited implementation-class families or dedicated leaves; include inherited overrides, block items, ticker factories, persistence and projection. |
| Items and data components | `net.minecraft.world.item.ItemStack#use`, `net.minecraft.world.item.ItemStack#inventoryTick`, `net.minecraft.world.item.ItemStack#onUseTick`, `net.minecraft.world.item.ItemStack#applyComponentsAndValidate`, `net.minecraft.world.item.Item#use`, `net.minecraft.core.component.DataComponentMap#makeCodec` | `ITM-001`, `ITM-003`, item-use/container/progression leaves and component-selected hooks own authoritative stack behavior. | Resolve 248 item fallback IDs; partition implementation subclasses, default components, `use_remainder`/consumable/equippable/tool/projectile hooks, inventory ticks and component-driven feature gates. |
| Entities, mobs and effects | `net.minecraft.world.entity.EntityType#create`, `net.minecraft.world.effect.MobEffectInstance#tickServer`, `net.minecraft.world.effect.MobEffect#applyEffectTick`, `net.minecraft.world.effect.MobEffect#applyInstantaneousEffect` | `ENT-001`, `ENT-004`, `MOB-004` and entity/effect leaves own construction, lifecycle, AI and effect application. | Resolve 37 entity-type fallback IDs; audit factory class, spawn finalization, tracking data, persistence, passengers, goals/brains, interaction/damage hooks and client metadata for every remaining subtype. |
| Menus and recipes | `net.minecraft.world.inventory.MenuType#create`, `net.minecraft.world.item.crafting.RecipeSerializer#codec`, `net.minecraft.world.item.crafting.RecipeSerializer#streamCodec`, `net.minecraft.world.item.crafting.RecipeManager#getRecipeFor`, `net.minecraft.world.item.crafting.RecipeManager#byKey` | `ITM-002`, `ITM-004` and container/recipe leaves own menu layout, controls, matching, assembly and convergence. | Keep the explicit 25-menu and 21-serializer partitions synchronized with new leaves; audit feature filtering, display lookup, recipe-book joins and data reload without treating recipe JSON values as new algorithms. |
| Loot, advancement and progression records | `net.minecraft.server.ReloadableServerRegistries$Holder#getLootTable`, `net.minecraft.world.level.storage.loot.LootTable#getRandomItemsRaw`, `net.minecraft.world.level.storage.loot.LootTable#getRandomItems`, `net.minecraft.server.ServerAdvancementManager#apply` | `ITM-006`, `ITM-007` and loot/progression leaves own context construction, conditions/functions, RNG, reward and criterion effects. | Preserve exact serializer/type dispatch for loot entries, conditions, functions, number/NBT/score providers and advancement triggers; join data reload, malformed references, recursion and per-player persistence. |
| Tags, holders, enchantments and data-selected predicates | `net.minecraft.core.Holder#is`, `net.minecraft.world.item.enchantment.Enchantment#getEffects`, `net.minecraft.world.item.enchantment.Enchantment#modifyUnfilteredValue`, `net.minecraft.world.item.enchantment.Enchantment#tick`, `net.minecraft.tags.TagLoader#build` | Block, item, entity, enchantment and environment leaves own each consumer; DataReload owns snapshot rebinding. | Inventory every behavior-affecting tag/component/type registry consumer, optional versus required references, ordered conditional effects and holder identity across reload; a tag list alone is not an algorithm owner. |
| Game rules and global selectors | `net.minecraft.world.level.gamerules.GameRules#codec`, `net.minecraft.world.level.gamerules.GameRules#get`, `net.minecraft.world.level.gamerules.GameRules#set`, `net.minecraft.world.level.gamerules.GameRule#valueCodec` | Simulation, environment, command, player, entity and world leaves own each rule's read/write consequences. | Keep all 59 IDs synchronized with the closed [game-rule consumer inventory](game-rule-consumers.md), including defaults/validation, indirect callers, callbacks, persistence and client projection; shared storage/codec behavior is insufficient. |
| World generation and structures | `net.minecraft.world.level.levelgen.feature.Feature#configuredCodec`, `net.minecraft.world.level.levelgen.feature.Feature#place`, `net.minecraft.world.level.levelgen.structure.Structure#findGenerationPoint`, `net.minecraft.world.level.levelgen.structure.Structure#afterPlace`, `net.minecraft.world.level.levelgen.DensityFunction#compute`, `net.minecraft.world.level.levelgen.DensityFunction#codec`, `net.minecraft.world.level.levelgen.SurfaceRules$Condition#test` | `WGEN-003`, `WGEN-PIPELINE-001`, structure, jigsaw, feature, dimension and border leaves own executable generation. | Resolve 184 worldgen fallback records by registry key and codec-selected implementation; distinguish genuine parameter trees from structure/biome/source/placement control flow and retain equivalence boundaries. |
| Catalog classification and recovery | `docs/reference/minecraft-java-26.2/catalog/catalog.toml` | Each exact/pattern family names its current rule owners; `mc-ref query` exposes the joined data and classification. | Replace all five `Unreviewed` remaining selectors with exact or proven pattern families or justified `DataOnly`; verify zero stale, zero-match, overlapping or silently broadened selectors before completion. |

## Boundary conclusions

- Registry lookup selects an identity or implementation object; later virtual, codec or data-driven
  dispatch selects the behavior. Both boundaries must be represented when they can diverge.
- `DataOnly` means a record supplies values to an already audited algorithm. It cannot be inferred
  from common JSON shape, common base class, absent catalog overlap or lack of a remembered quirk.
- Tags, components and holder references can change the branch taken by generic code without
  creating an ID-specific subclass. Consumer search is therefore part of content dispatch.
- `InProgress` remains required while any of the 818 catalog IDs is `Unreviewed`, even though every
  locked ID has exactly one structural catalog owner.

## Recovery procedure

1. For each fallback ID, resolve its registered implementation/factory and every effective virtual
   hook, codec/type discriminator, default component, tag and bundled-data reference.
2. Compare the trace with an existing audited family. Add an exact/pattern member only when all
   independent behavior is inherited; otherwise create or extend a source-specified leaf.
3. Prove `DataOnly` by tracing every decoded field into an already specified algorithm and showing
   no ID-specific dispatch, callback or consumer branch.
4. Run `mc-ref query`, symbol verification, catalog coverage and readiness after every family; keep
   all raw reports and class inspection under ignored `target/mc-reference/26.2/` paths.
5. Promote this surface only when the catalog has zero `Unreviewed` IDs and all cross-system joins
   from content selection to reload, persistence and projection have terminal ownership.
