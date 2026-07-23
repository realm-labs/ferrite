# Data Reload Root Inventory

**Surface:** `SURFACE-DATA-RELOAD-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns bootstrap and data-pack reload from pack selection through publication to live
worlds and unmodified clients. Generated registry and bundled-data reports lock the input identities
and values; they do not by themselves specify listener dependencies, validation, failure behavior or
the point at which a replacement snapshot becomes observable.

| Reload family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Pack discovery, selection and feature flags | `net.minecraft.server.packs.repository.PackRepository#reload`, `net.minecraft.server.packs.repository.PackRepository#setSelected`, `net.minecraft.server.MinecraftServer#configurePackRepository`, `net.minecraft.world.level.WorldDataConfiguration#enabledFeatures` | This surface owns selection and admission; content and world-generation leaves own the behavior parameterized by the accepted packs and flags. | Enumerate required, missing, incompatible, explicitly disabled and auto-enabled pack branches; preserve selection order, feature closure, warning/error outcomes and persisted world configuration. |
| Bootstrap and worldgen registries | `net.minecraft.resources.RegistryDataLoader#load` | `WGEN-002`, `WGEN-003` and the locked catalog own registry-selected behavior and concrete values; `BLK-JIGSAW-001` fixes the operator block's required generation-time template-pool lookup, `BLK-STRUCTURE-001` fixes named structure-template manager lookup after the registry snapshot is live, and `BLK-TEST-INSTANCE-001` fixes configured test-instance holder lookup plus its template/rotation/padding/required fields. | Partition both overloads by bootstrap/network source; audit codec errors, holder/reference resolution, lifecycle metadata, registry layering, duplicate keys and all-or-nothing visibility. |
| Reloadable registries and loot | `net.minecraft.server.ReloadableServerRegistries#reload`, `net.minecraft.server.ReloadableServerRegistries$LoadResult#lookupWithUpdatedTags`, `net.minecraft.server.ReloadableServerRegistries$Holder#getLootTable`, `net.minecraft.world.level.storage.loot.LootDataType#runValidation` | `ITM-006`, `ITM-007`, `ITM-CHEST-001`, `ITM-HOPPER-001`, `ITM-DISPENSER-001` and loot/content leaves own caller context and evaluation after a validated registry snapshot is published. | Enumerate reloadable registry keys and validation contexts; audit missing/default loot tables, recursive references, validation diagnostics, tag-updated lookup identity and rejected-load retention. |
| Listener construction and dependency barrier | `net.minecraft.server.ReloadableServerResources#loadResources`, `net.minecraft.server.ReloadableServerResources#listeners`, `net.minecraft.server.packs.resources.SimpleReloadInstance#create`, `net.minecraft.server.packs.resources.ReloadInstance#done` | Individual rows below own listener semantics; this family owns prepare/apply dependency edges, executors, barriers and aggregate completion. | Recover the exact ordered listener list and dependency DAG, including parallel preparation, apply serialization, profiler-visible ordering, cancellation and the first exceptional completion. |
| Tags and component rebinding | `net.minecraft.tags.TagLoader#load`, `net.minecraft.tags.TagLoader#build`, `net.minecraft.tags.TagLoader#loadTagsForExistingRegistries`, `net.minecraft.server.ReloadableServerResources#updateComponentsAndStaticRegistryTags` | `BLK-SCULK-SENSOR-001` owns vibration, ignore-sneaking, damping, occlusion and resonator membership effects; `BLK-BEACON-001` owns live base/payment membership consumption; `BLK-SIGN-001` owns standing/wall/ceiling/wall-hanging aggregation and the live hanging-sign support/orientation branch; `BLK-SKULL-001` owns live `wither_summon_base_blocks` membership during full/base-pattern checks; `BLK-STRUCTURE-VOID-001` owns the locked structure-void membership in `replaceable` alongside its code-level replaceable property; `ITM-HOPPER-001` owns the `does_not_block_hoppers` loose-item gate; `ITM-DISPENSER-001` owns the reloadable sulfur-cube swallowable item tag and all 12 archetype delegates; other tag-backed leaves own their memberships, and component-bearing item rules own derived component behavior. | Inventory every remaining tag directory and pending-tag bind; audit optional/required entries, cycles, missing references, replacement versus merge, component reinitialization and holder identity seen by existing objects. |
| Recipes, functions and advancements | `net.minecraft.world.item.crafting.RecipeManager#prepare`, `net.minecraft.world.item.crafting.RecipeManager#apply`, `net.minecraft.world.item.crafting.RecipeManager#finalizeRecipeLoading`, `net.minecraft.server.ServerFunctionLibrary#reload`, `net.minecraft.server.ServerAdvancementManager#apply` | `ITM-004` owns recipe matching and crafting results; command/function and progression leaves own execution and advancement state. | Enumerate resource decode, duplicate/replacement and feature-filter branches; audit function compilation context, advancement parent/display resolution, recipe cache/index rebuild and active-player progression reconciliation. |
| Atomic server publication and live refresh | `net.minecraft.server.MinecraftServer#reloadResources`, `net.minecraft.server.players.PlayerList#reloadResources`, `net.minecraft.commands.Commands#sendCommands` | Simulation and world owners consume the published snapshot; PlayerLifecycle and client projection own observable refresh results. | Locate every field swap and post-publication callback; prove whether worlds observe one coherent snapshot, what remains installed on each failure point, and the order of recipes, advancements, functions, commands, tags and player refresh. |
| Active-session reconfiguration and convergence | `net.minecraft.server.network.ServerGamePacketListenerImpl#switchToConfig`, `net.minecraft.server.players.PlayerList#reloadResources` | Configuration, reconfiguration and live-tag protocol families own wire state and packet layouts. | Audit admission during reload, play-to-configuration transition gates, registry/tag snapshot selection, acknowledgement ordering, disconnect/failure branches and convergence for players joining or changing dimension concurrently. |

## Current boundary conclusions

- Pack order and the enabled feature set are behavior inputs. A compatible implementation may use a
  different pack container, but it must accept, reject and prioritize the same locked inputs.
- Reloadable registries are prepared before the resource listener aggregate is constructed. The
  aggregate exposes a completion future; this does not yet prove atomicity for every later server
  field swap or player refresh, so failure behavior remains explicitly open.
- Loot tables, predicates and modifiers are reloadable registry content in 26.2. They must not be
  modeled as an independent legacy loot manager.
- Existing worlds, objects and sessions can retain holder, tag, command or recipe views. Successful
  data decoding alone is therefore insufficient without publication and convergence checks.
- `BLK-AIR-001` owns the live `air` and `replaceable` memberships shared by all three air states and
  the locked `parrots_spawnable_on` membership held only by ordinary air.
- `BLK-BEDROCK-001` owns the live dragon/wither/wind-charge protection, feature replacement,
  geode-invalid and End infiniburn memberships; direct identity and registered-property branches
  remain code-locked.
- `BLK-REINFORCED-DEEPSLATE-001` owns the live dragon/wither and feature-replacement memberships
  plus wind-charge nonmembership; its registered properties and piston identity gate remain
  code-locked.
- `BLK-TINTED-GLASS-001` owns the live `impermeable` membership and its negative boundary: the only
  locked consumer is invoked with the beehive's state, so current vanilla code never tests tinted
  glass there. Its registered light/spawn properties and golem identity gate remain code-locked.
- `BLK-GLASS-001` owns the live `impermeable` and `smelts_to_glass` memberships. The former has the
  same beehive caller-state non-interaction; the latter selects smelting inputs. Registered light/
  spawn properties, Silk Touch loot and the golem identity gate remain code-locked.

## Recovery procedure

1. Enumerate the concrete listeners returned by `ReloadableServerResources#listeners` and record
   each prepare dependency, apply barrier, executor and publication consumer.
2. For every registry and resource family, record input ordering, decode/validation branches,
   holder/tag binding and the first authoritative consumer after publication.
3. Inject failure at pack open, registry decode, tag bind, listener prepare, listener apply and
   post-publication refresh; compare the retained server/world/session snapshot at every point.
4. Reload before login, during configuration, in active play and while another player joins or
   changes dimension; verify command, registry, tag, recipe and advancement convergence over the
   locked protocol families.
5. Join each conclusion to the owning semantic leaf and executable vector before promoting this
   surface. A listener list or a successful `/reload` smoke test alone is not completion.
