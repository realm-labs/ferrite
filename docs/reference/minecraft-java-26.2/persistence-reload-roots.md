# Persistence and Reload Root Inventory

**Surface:** `SURFACE-PERSISTENCE-RELOAD-001`
**Status:** `Mapped`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns observable continuity across chunk unload/reload, player disconnect/rejoin and
full server restart without requiring Minecraft's original file formats. Ferrite may encode state
differently, but every behavior-owned value must either survive, be deterministically reconstructed,
or be explicitly transient with a specified first post-boundary result.

| Continuity family | Locked source roots | Exact ownership and failure conclusion |
|---|---|---|
| Server bootstrap, save and shutdown | `net.minecraft.server.MinecraftServer#loadLevel`, `net.minecraft.server.MinecraftServer#saveAllChunks`, `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer` | The mapped WorldLifecycle transaction fixes player-before-level save, work draining, final flush and close order; `SIM-001`/`SIM-006` fix autosave and pause admission. Force/flush/no-save branches are inputs to those roots, not alternate state ownership. A clean close joins durable work; a process crash exposes only writes completed before loss and performs no catch-up transaction on restart. |
| Level and chunk lifecycle | `net.minecraft.server.level.ServerLevel#save`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad` | The mapped WorldLifecycle inventory fixes dirty/admission caps, current save-dependency identity, unload cancellation, save-before-live teardown, POI/light closure and first ready/send publication. Missing or missing-level chunk data creates an empty ProtoChunk; reported non-Error read/parse failure logs and also creates empty, while an `Error` becomes a crash report. `WGEN-PIPELINE-001` owns persisted status, section/palette, heightmap, structure, light and postprocessing reconstruction. |
| Scheduled block and fluid work | `net.minecraft.world.ticks.LevelChunkTicks#pack`, `net.minecraft.world.ticks.LevelChunkTicks#unpack`, `net.minecraft.world.ticks.SavedTick#unpack` | `SIM-SCHEDULE-001` is exhaustive: save narrows remaining absolute delay to signed 32-bit, load adds it to current signed-64 game time, preserves type/position/priority and assigns per-chunk sub-orders `-N..-1` in list order. Loaded inactive time retains absolute triggers; fully unloaded time does not reduce saved positive delay. Deduplication and first callback use the ordinary queue rules. Only the explicitly owned equal-priority/equal-reconstructed-sub-order cross-chunk tie remains `SourceInconclusive` under `EXP-SIM-002`. |
| Block states and block entities | `net.minecraft.world.level.block.entity.BlockEntity#setChanged`, `net.minecraft.world.level.block.entity.BlockEntity#saveWithFullMetadata`, `net.minecraft.world.level.block.entity.BlockEntity#loadStatic`, `net.minecraft.world.level.block.entity.BlockEntity#loadWithComponents` | Palette state and all 49 exact block-entity types are owned by `BLK-001`, `BLK-003`, `BLK-007` and their subtype leaves. `setChanged` dirties the containing level and updates comparator neighbors only for non-Air state. Full metadata writes subtype fields, components, type and coordinates. Load resolves the type, constructs it at the caller's position/state, loads subtype fields then components, and logs/skips invalid type, construction failure or any load exception. Subtype leaves define missing/malformed defaults, component merging, ticker installation, derived caches and first comparator/update/client result. |
| Persistent entities | `net.minecraft.world.level.entity.PersistentEntitySectionManager#processChunkUnload`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#processPendingLoads`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#saveAll` | `ENT-001` and every subtype leaf own the serialized entity/passenger graph and reconstructed AI caches. UUID admission occurs before section insertion; a duplicate logs and rejects the complete entity. Accepted loaded entities install section/callback, then start tracking and ticking in that order when visibility admits them, without the fresh-created callback. Load completions enter an inbox in completion order and retain each chunk payload's entity order. Hidden unload waits until the chunk is not FRESH/PENDING, stores saveable roots, then marks each root/passenger `UNLOADED_TO_CHUNK` and clears callbacks. A load failure logs without enqueuing a result, leaving PENDING and therefore blocking that chunk's unload/save completion. `saveAll` repeatedly flushes/processes loads/stores until every chunk succeeds, then performs a final flush. |
| Players and reconnect | `net.minecraft.server.players.PlayerList#loadPlayerData`, `net.minecraft.server.players.PlayerList#save`, `net.minecraft.server.level.ServerPlayer#readAdditionalSaveData`, `net.minecraft.server.level.ServerPlayer#addAdditionalSaveData` | The mapped [player lifecycle inventory](player-lifecycle-roots.md) is the exhaustive persisted/reconstructed field ledger. Player data saves before stats then advancements; reconnect rebuilds transport, timers and sent mirrors, rebinds UUID-keyed auxiliary state and projects through the locked join sequence. Missing direct fields use the listed defaults, recipe keys are revalidated against the active manager, and sleeping never resumes. |
| Saved world data and auxiliary progression | `net.minecraft.world.level.storage.SavedDataStorage#computeIfAbsent`, `net.minecraft.world.level.storage.SavedDataStorage#scheduleSave`, `net.minecraft.world.level.storage.SavedDataStorage#saveAndJoin`, `net.minecraft.server.ServerScoreboard#load`, `net.minecraft.server.ServerScoreboard#setDirty`, `net.minecraft.stats.ServerStatsCounter#save`, `net.minecraft.server.PlayerAdvancements#load`, `net.minecraft.server.PlayerAdvancements#save` | The cache memoizes both present and absent reads; `computeIfAbsent` constructs and dirties only after a null result. Read detects compressed/plain input, defaults missing data version to 1343, data-fixes to the locked version and codec-parses `data`; any exception or parse failure logs and returns null. Save snapshots every dirty cached value in cache iteration order, clears dirty before dispatch, chains after prior writes and partitions only for IO parallelism. IO failure logs inside the task and does not restore dirty, so no automatic retry occurs. `saveAndJoin` joins the chain; close rejects repeats. Files/types are independent, with no cross-file atomic commit. Scoreboard, map/border, stats and advancement leaves own their field/default/listener semantics. |
| Reconstructed and transient state | `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.players.PlayerList#placeNewPlayer`, `net.minecraft.server.players.PlayerList#sendLevelInfo` | Every completion leaf classifies its behavior state as serialized, code/data-derived, session-local or transient. Serialization includes committed world/entity/player/block-entity/item/menu/progression state and queued work explicitly named by those owners. Code/data-derived registries, tags, recipes, loot, models and behavior tables bind from the active reload snapshot. RNG draws, partial attempts, iterators, AI path/cache state, interpolation, transport IDs, acknowledgements, latency and sent mirrors reset unless an owner explicitly records a cursor/value. First observation is authoritative chunk/player publication followed by the owner's next admitted callback; no interrupted callback resumes or catches up. |

## Current boundary conclusions

- A compatible Ferrite save need not use NBT, Region, Anvil or Mojang data-fix layouts. The required
  contract is the post-boundary authoritative state and its next observable transition.
- Chunk persistence is asynchronous in the locked implementation, while server/level close paths
  join outstanding work. Ferrite may schedule writes differently only if save admission, failure,
  unload visibility and restart results remain equivalent.
- Saved scheduled ticks reconstruct relative to load time. Fully unloaded wall/game time does not
  silently become accumulated callback work; the exact queue rule and unresolved equal-head case
  remain owned by `SIM-SCHEDULE-001`.
- Runtime-only transport IDs, acknowledgement windows, interpolation caches and client mirrors are
  not persistent identities. Their reset must still lead to the same authoritative reprojection.
- Exact cave-air and void-air palette entries are durable chunk state even though an all-air
  section's live read shortcut returns ordinary air; AIR item identity never survives as a
  positive inventory stack.
- `BLK-BUDDING-AMETHYST-001` fixes ordinary palette continuity for budding state 23403 and all 48
  facing/waterlogged stage states. Random-tick RNG and rejected/failed growth attempts are not
  durable obligations; after reload only active-chunk admission can produce the next growth write.
- `BLK-CALCITE-SMOOTH-BASALT-001` fixes ordinary palette continuity for calcite state 27160 and
  smooth-basalt state 32069 after player, geode, surface and ancient-city writes. Recipe, tag,
  feature, pool and processor snapshots can change on reload, but do not rewrite saved states by
  themselves.
- `BLK-DEEPSLATE-001` fixes palette continuity for axis states 30416..30418 after player, surface,
  flat, retrogen, feature and ancient-city writes. Recipe/tag/worldgen reload changes future
  selection only; structure rot decisions, failed writes, recipe progress and surface positional
  RNG are not durable state obligations beyond already committed blocks and inventories.
- `BLK-DEEPSLATE-MASONRY-001` fixes palette continuity for property-free states 30419, 30830,
  31241, 31652 and 32063..32065 after player, recipe, sculk and structure writes. Reload changes
  future recipe/tag/processor selection; prior rot/crack draws and rejected writes are not durable
  obligations beyond the blocks and inventories already committed.
- `BLK-DRIPSTONE-BLOCK-001` fixes palette continuity for property-free state 30208 after player,
  recipe and all three normal cave-feature writes; the large-feature debug branch writes no such
  state. Reload changes future recipe/trade/tag/feature
  selection; pointed growth draws, failed scans and rejected writes are not durable obligations.
- `BLK-BRICKS-001` fixes palette continuity for property-free state 2340 after player, recipe and
  admitted trail-ruins, cold-ocean-ruin or plains-village template writes. Reload changes future
  loot, recipe, tag, archetype, template and processor selection; prior integrity draws, rejected
  writes and raw template cells are not durable obligations beyond committed blocks/inventories.
- `BLK-PACKED-MUD-001` fixes palette continuity for property-free state 7758 after player, recipe,
  raw trail-template or admitted mud-bricks aging writes. Reload changes future loot, recipe, tag,
  archetype, template and processor selection; prior position draws, rejected writes, raw cells
  and eligible processor inputs are not durable obligations beyond committed blocks/inventories.
- `BLK-MUD-BRICKS-001` fixes palette continuity for property-free state 7759 after player, recipe,
  raw template, connector-final or aging-survivor writes. Reload changes future loot, recipe, tag,
  archetype, template and processor selection; prior position draws, rejected writes and raw or
  connector cells are not durable obligations beyond committed blocks/inventories.
- `BLK-PURPUR-BLOCK-001` fixes palette continuity for state 14712 after player, recipe or admitted
  End-city writes. Reload changes future loot, recipe, tag, archetype, template and advancement
  display selection without rewriting committed blocks or inventories.
- `BLK-NETHER-WART-BLOCK-001` fixes palette continuity for state 14846 after player, surface,
  huge-fungus or weeping-vines writes. Reload changes future loot, recipe, tutorial, archetype,
  carver and configured-worldgen selection; composter draws, spawn decisions and rejected
  feature/carver writes are not durable obligations beyond committed state and inventories.
- `BLK-WARPED-WART-BLOCK-001` fixes palette continuity for state 20959 after player, surface or
  huge-fungus writes. Reload changes future loot, tutorial, archetype, carver and configured
  worldgen selection; composter draws, twisting support decisions and rejected writes are not
  durable obligations beyond committed state and inventories.
- `BLK-NETHER-SPROUTS-001` fixes palette continuity for state 20961 after player or vegetation
  writes. Support, loot, replacement, enchanting and configured-worldgen membership use the active
  reload snapshot; rejected placement/feature offers, composter draws and combined-step decisions
  are not durable obligations beyond committed blocks, inventories and scheduled composter work.
- `BLK-NETHER-ROOTS-001` fixes palette continuity for root states 20960/21031 and potted states
  21829/21828 after player, Enderman or generation writes. Active reload snapshots select support,
  loot, replacement, Enderman and worldgen behavior; uncommitted composter/provider/goal draws and
  failed pot/inventory/feature operations do not become durable state. Carried-root persistence
  remains with the Enderman owner.
- `BLK-NETHER-WART-001` fixes palette continuity for age states 9447..9450 after player,
  random-tick, fortress or bastion writes. Active reload snapshots select support, loot, recipes,
  advancements, trades, chest and bastion records; uncommitted growth, loot, composter, brewing,
  trade, chest and structure-selection draws do not become durable state beyond committed blocks,
  inventories, offers, potion contents and scheduled composter work owned by their subsystems.
- `BLK-NETHER-STEM-001` fixes palette continuity for all 24 axis states after placement, stripping
  or huge-fungus writes. Active reload snapshots select loot, tags, recipes,
  advancements, sulfur equipment and fungus records; uncommitted strip, recipe, parrot, tree and
  feature attempts do not become durable beyond committed blocks, item durability, inventories and
  entity equipment owned by their subsystems.
- `BLK-CORAL-BLOCK-001` fixes palette continuity for live states 15142..15146 and dead states
  15137..15141 after placement, dry conversion or warm-ocean writes. A pending dry-conversion tick
  persists separately and its restored callback revalidates current identity and adjacent water;
  active reload snapshots select loot, tags, trades and worldgen without rewriting existing states.
  Uncommitted loot, pickle, trade and feature draws do not become durable beyond committed blocks,
  inventories, offers and entity equipment owned by their subsystems.
- `BLK-CORAL-PLANT-001` fixes palette continuity for states 15147..15266, including waterlogged
  values and wall facing, after placement, bonemeal or coral-feature writes. Pending live drying
  persists separately; its restored callback revalidates identity and adjacent water, preserves wall
  facing on conversion and cannot revive dead forms. Reload-selected loot and coral/bonemeal/pickaxe
  tags do not rewrite existing states, and uncommitted loot or selection draws are not durable.
- `BLK-FLOWER-POT-001` fixes palette-only continuity for 37 property-free states with no block
  entity. The static content map reconstructs from registration; interrupted insertion/extraction,
  random-tick position/Trail draws and hoglin sensor state are not stored in the pot. The next
  admitted eyeblossom tick rereads the current environment attribute, while loot, recipe, tag and
  structure snapshots reload without rewriting an existing pot identity.
- `BLK-COPPER-FULL-001` fixes palette-only continuity for 24 property-free states with no block
  entity. The fifteen unwaxed weather/age collections, twelve wax maps and pumpkin/chest maps
  reconstruct from code; interrupted draws, neighborhood censuses and partially committed
  honeycomb/axe/golem transactions are not replayed. The next admitted weather tick rereads current
  neighbors, while loot, recipe, advancement, tag, archetype and structure snapshots reload without
  rewriting an existing age/wax identity.
- `BLK-SAPLING-001` fixes palette continuity for states 29..44, including the sole `stage`
  property and no block entity. Support/flower tag snapshots, selected configured-feature holders,
  RNG position and partially executed clear/place/restore transactions are not stored in the
  sapling. Reload changes later support and growth selection; existing saplings are revalidated
  only by a later placement/update/tick/use path.
- `BLK-BAMBOO-001` fixes palette continuity for sapling state 15278 and stalk states 15279..15290,
  including age/leaves/stage and no block entity. Support/tag snapshots, growth RNG, local height
  counts and partially executed leaf/segment/feature writes are not stored. A pending support-loss
  stalk tick persists through generic scheduled-tick ownership and revalidates live support; later
  random/use/worldgen paths read the current data snapshot.
- `BLK-ANCIENT-DEBRIS-001` fixes palette continuity for property-free state 21819 with no block
  entity and item-stack continuity for its fire-resistant component. Loot, recipe, advancement,
  tag, archetype and worldgen snapshots reload independently. Scattered-ore RNG, rejected
  candidates and failed writes do not persist or resume; committed ore cells persist ordinarily.
- `BLK-STEM-CROP-001` fixes palette continuity for attached states 8334..8341 and age states
  8342..8357 with no block entity, plus ordinary seed-stack continuity. Crop-speed scans, random
  and bone-meal cursors, faced-fruit checks and the two-write fruit transaction do not persist or
  resume; only committed cells survive. Loot, recipes, advancements, trades, tags, processors,
  fungus records and templates reload independently.
- `BLK-OVERWORLD-CROP-001` fixes ordinary palette continuity for wheat states 5311..5318, carrots
  10659..10666, potatoes 10667..10674 and beetroots 14811..14814, with no block entity, plus
  ordinary continuity for their seven coupled item stacks and components. Growth, bone-meal,
  loot and villager draws never persist or catch up; each callback or AI tick rereads current
  light, farmland, crowding, inventory and crop state.
- `BLK-TORCHFLOWER-CROP-001` fixes palette continuity for crop states 14797..14798 and mature
  flower state 2323 with no block entity, plus ordinary seed/flower stack continuity. Growth,
  flower-spread, loot, compost and AI draws never persist or catch up; later callbacks reread
  current light, support, crowding, tags and destinations.
- `BLK-PITCHER-CROP-001` fixes palette continuity for all age/half crop states 14799..14808 and
  independent mature-plant states 14809..14810 with no block entity, plus ordinary pod/plant stack
  continuity. Growth, bone-meal, loot, compost, sniffer and farmer cursors never persist or catch
  up; later callbacks reread current light, support, counterpart, top cell, tags and inventory.
- `BLK-SWEET-BERRY-BUSH-001` fixes palette continuity for age states 20941..20944 with no block
  entity plus ordinary berry-stack continuity. Growth, bone-meal, harvest, movement-damage,
  bee/fox, loot, compost and feature draws never persist or catch up; later callbacks reread age,
  support, light, movement, AI, gamerule, tag and data snapshots.
- `BLK-CAVE-VINES-001` fixes palette continuity for head states 30249..30300 and body states
  30301..30302 with no block entity, plus ordinary glow-berry stacks. Placement/conversion age,
  growth/berry, harvest, bee, loot, compost and feature draws never persist or catch up; later
  callbacks reread support, below state, age/berries, AI, tag and data snapshots.
- `BLK-CHORUS-001` fixes palette continuity for plant connection states 14642..14705 and flower
  ages 14706..14711 with no block entity, plus four ordinary stacks and chorus fruit's item-keyed
  cooldown. Growth/branch, projectile, loot, teleport and feature draws never persist or catch up;
  later callbacks reread support, neighbors, age, chunks, collision, liquid, gamerules, tags and
  data snapshots.
- `ITM-STEW-001` fixes ordinary stack continuity for bowl and all four filled foods, including the
  ordered suspicious-effect component, plus brown mooshroom's nullable `stew_effects` payload.
  Consumption progress, remainder insertion/drop, effect offers, charge/milk particles and loot,
  recipe or trade draws do not persist or catch up; later actions reread stacks, hunger, entity
  payload, inventory and active data snapshots.
- `ITM-BUNDLE-001` persists each bundle's ordered stored-stack list through the ordinary component
  codec, including nested contents and patched components. The selected index, mouse-scroll state,
  held-use progress and tooltip/render derivations do not persist; decode or reconstruction starts
  selection at `-1`, and later actions recompute weight, visible entries and projection from the
  committed list.
- `ITM-BOAT-001` persists exact vehicle type, transform, passenger graph, generic entity fields and
  either 27 materialized chest slots or a mutually exclusive pending loot table/seed. Placement
  hit/sweep/admission, active menus, content-split/trade/loot RNG and render water masks are
  transient. `KILLED`/`DISCARDED` removal empties chest storage before matching itemization,
  whereas unload/dimension transfer retains it; data reload changes future recipe/tag/trade and
  advancement reads without rewriting existing stacks or entities.
- `ITM-POTTERY-SHERD-001` persists ordinary sherd stacks and ordered pot-face identities through
  generic stack and decorated-pot component codecs; advancement criteria persist independently.
  Crafting, cracking, archaeology and jigsaw attempts plus their RNG cursors are transient. Data
  reload changes later tag, recipe, loot and criterion reads without rewriting existing stacks,
  faces or completed progress; client item/pattern assets reload independently.
- `ITM-SMITHING-TEMPLATE-001` persists ordinary template stacks, transformed base patches and
  trimmed-equipment components through generic stack codecs; recipe unlocks and advancement
  criteria persist independently. Crafting, smithing-preview/take, loot, entity-drop and vault
  attempts plus their RNG cursors are transient. Data reload changes later recipe, pattern, loot
  and criterion reads without rewriting existing stacks or completed progress; client assets,
  language and slot presentation reload or reconstruct independently.
- `ITM-HARNESS-001` persists ordinary harness stacks, Happy Ghast body equipment/drop chance,
  passenger graph and `still_timeout` through generic stack/entity owners; equip, shearing,
  controller input and sound/game-event attempts are transient. Data reload changes later
  allowed-entity, temptation, recipe and unlock reads without rewriting stored body equipment,
  which may remain rendered while functionally invalid; client assets reload independently.
- `ITM-MINECART-001` persists exact minecart subtype, transform, passengers, custom display/name,
  container pending loot or slots, hopper enabled, furnace push/fuel, TNT fuse/factors and command
  carrier state through generic entity owners; placement, menu, fuel, activator, destruction and
  crafting attempts are transient. Data reload changes later rail/fuel tags, recipes, advancements,
  mineshaft and loot selection without rewriting stacks or entities; feature flags remain world
  configuration and client assets reload independently.
- `ITM-STEERING-STICK-001` persists stick/fishing-rod identity, damage and component patch plus
  separately owned pig/strider saddle/passenger state. Active boost flag, elapsed clock and total
  are not saved: losing the controller merely pauses them in a process-continuous entity, while
  entity reload cancels them. Data reload changes later durability-enchantable and strider-tempt
  membership, recipes and advancements without rewriting stacks, mounts or progress; exact pig
  lure/controller mappings stay code-built and client assets reload independently.
- `ITM-SPEAR-001` persists spear identity, damage, enchantments and component patch through ordinary
  stacks and mob equipment. STAB swings, held-use flags, kinetic contact timestamps, feedback,
  Lunge execution and AI approach/charge/retreat never resume after reload. Data reload changes
  later tags, enchantment, recipes, recycling, loot and criteria without rewriting stored stacks,
  equipment or progress; resources reload dual-context models and animations independently.
- `ITM-NAUTILUS-ARMOR-001` persists armor identity/components in stacks or nautilus BODY equipment
  together with the mob's guaranteed-drop state. Equip, menu, shear and sunlight attempts do not
  resume; attributes, rendering and zombie sunlight protection derive again from the stored stack.
  Data reload changes later allowed-entity/sunlight checks, recipes, loot and unlocks without
  rewriting equipment, while resources replace item/body/menu presentation independently.
- `ITM-EGG-001` persists all three stack identities/component patches, a thrown entity's one-stack
  `Item`, owner and motion, and a chicken's variant plus `EggLayTime`; absent projectile `Item`
  reconstructs ordinary egg. Flight collision, hatch draws/loop, laying gift evaluation, sounds,
  particles and crafting never resume. Data reload changes later tag, recipe, unlock, gift-table
  and variant reads without replaying transactions, while resources replace flat/projectile
  presentation independently.

## Reproduction matrix

1. Replay each persisted, reconstructed and transient field through clean chunk unload/reload,
   player disconnect/rejoin and full restart, and compare the first authoritative publication plus
   next admitted callback with an uninterrupted control.
2. Repeat chunk and saved-data reads with missing input, missing level data, malformed codec input
   and an injected non-`Error` failure; verify the documented empty/default result, logging,
   cache memoization and absence of a catch-up callback.
3. Inject entity-chunk load failure, duplicate UUIDs and interleaved successful completions; verify
   the retained `PENDING` failure, complete duplicate rejection, completion/payload ordering and
   tracking-before-ticking insertion.
4. Save scheduled ticks at positive, overdue and signed-32 boundary delays, then reload at changed
   game times and across chunks. Verify reconstructed trigger/sub-order behavior, and run
   `EXP-SIM-002` for the sole source-inconclusive equal-head global tie.
5. Round-trip every locked block-entity type with absent optional fields, invalid type IDs,
   construction failures and malformed subtype/component input; verify skip/default behavior,
   dirty/comparator effects and the first update/client projection.
6. Reconnect players with present, absent and malformed primary/auxiliary files, including changed
   recipe data and interrupted sleep; verify save ordering, UUID rebinding, defaults, recipe
   revalidation and the locked join projection.
7. Inject saved-data encoding and IO failures across one and multiple files; verify dirty-clear
   timing, serialized batch chaining, per-file independence, no automatic retry and clean-close
   joining.
8. Crash between admitted writes and compare restart with a clean save/close. Only completed writes
   may survive the crash; no interrupted RNG draw, iterator, callback, transport state or client
   mirror may resume unless its semantic owner explicitly persists a cursor or value.
