# Persistence and Reload Root Inventory

**Surface:** `SURFACE-PERSISTENCE-RELOAD-001`
**Status:** `InProgress`
**Primary evidence:** `OFF-SERVER-001`

This inventory owns observable continuity across chunk unload/reload, player disconnect/rejoin and
full server restart without requiring Minecraft's original file formats. Ferrite may encode state
differently, but every behavior-owned value must either survive, be deterministically reconstructed,
or be explicitly transient with a specified first post-boundary result.

| Continuity family | Locked source roots | Existing semantic owners | Remaining audit |
|---|---|---|---|
| Server bootstrap, save and shutdown | `net.minecraft.server.MinecraftServer#loadLevel`, `net.minecraft.server.MinecraftServer#saveAllChunks`, `net.minecraft.server.MinecraftServer#saveEverything`, `net.minecraft.server.MinecraftServer#stopServer` | `SIM-001`/`SIM-006` own tick admission, autosave and pause timing; this surface owns continuity across the resulting durable boundary. | Audit force/flush/skip-save arguments, per-level and player ordering, pending asynchronous writes, partial failures, clean close versus crash and first restarted tick. |
| Level and chunk lifecycle | `net.minecraft.server.level.ServerLevel#save`, `net.minecraft.server.level.ServerLevel#unload`, `net.minecraft.server.level.ServerChunkCache#save`, `net.minecraft.server.level.ServerChunkCache#close`, `net.minecraft.server.level.ChunkMap#saveAllChunks`, `net.minecraft.server.level.ChunkMap#processUnloads`, `net.minecraft.server.level.ChunkMap#scheduleUnload`, `net.minecraft.server.level.ChunkMap#scheduleChunkLoad` | `WGEN-001`, `SIM-005`, block and entity owners define live state; `BLK-AIR-001` fixes exact section-palette continuity versus ordinary-air all-air read projection; persistence owns when that state becomes inactive, serialized, reconstructed and visible again. | Enumerate dirty/admission gates, active-write limits, holder futures, unload cancellation, read/write failures, data-fix/default branches, light/height/structure state and first-ready publication. |
| Scheduled block and fluid work | `net.minecraft.world.ticks.LevelChunkTicks#pack`, `net.minecraft.world.ticks.LevelChunkTicks#unpack`, `net.minecraft.world.ticks.SavedTick#unpack` | `SIM-003` and `SIM-SCHEDULE-001` own delay reconstruction, priority, sub-order and the explicit equal-head source-inconclusive case. | Cross chunk unload, full restart, already-overdue and integer-overflow cases with world time, duplicate filtering and first callback order; preserve the existing experiment-owned global tie. |
| Block states and block entities | `net.minecraft.world.level.block.entity.BlockEntity#setChanged`, `net.minecraft.world.level.block.entity.BlockEntity#saveWithFullMetadata`, `net.minecraft.world.level.block.entity.BlockEntity#loadStatic`, `net.minecraft.world.level.block.entity.BlockEntity#loadWithComponents` | `BLK-001`, `BLK-003` and `BLK-007` own state, mutation dirtiness and block-entity lifecycle; `BLK-SCULK-SENSOR-001` fixes frequency plus selector/current vibration/delay continuity, `BLK-JIGSAW-001` fixes all seven connector fields and orientation-dependent defaults, `BLK-STRUCTURE-001` fixes its complete record, divergent fresh/load defaults, load clamps, power latch and full update tag while separating entity saves from memory/disk template-manager continuity, `BLK-STRUCTURE-VOID-001` fixes ordinary world-state continuity versus its deliberately absent captured-template coordinate, `BLK-TEST-BLOCK-001` fixes mode/message/powered continuity plus the transient trigger and divergent state defaults, `BLK-TEST-INSTANCE-001` fixes its complete data/marker codecs, malformed-record retention, dirty/update path and runner-owned entity replacement, `BLK-CONDUIT-001` fixes optional target-UUID continuity plus all reset/derived fields and non-dirty target projection, `BLK-BEACON-001` fixes power/name/lock continuity plus ignored saved Levels and rebuilt beam/base state, `BLK-SIGN-001` fixes both four-line raw/filtered sides, color/glow/wax continuity, component resolution and transient editor authorization, `BLK-SKULL-001` fixes nullable profile/sound/name continuity plus nonserialized client animation retention/reset, `ITM-CHEST-001` fixes per-half items/loot/name/lock, pairing migration and transient opener/lid reconstruction, `ITM-HOPPER-001` fixes five-slot loot/name/lock and exact signed cooldown continuity plus reconstructed facing/tick time, `ITM-DISPENSER-001` fixes nine-slot loot/name/lock continuity while separating persisted facing/triggered state from the live scheduled callback, and other subtype leaves own their observable fields. | Build the remaining exhaustive block-entity field matrix, unknown/wrong type and malformed/default branches, component interaction, ticker reinstallation, cached/transient fields and first update/comparator/client projection. |
| Persistent entities | `net.minecraft.world.level.entity.PersistentEntitySectionManager#processChunkUnload`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#processPendingLoads`, `net.minecraft.world.level.entity.PersistentEntitySectionManager#saveAll` | `ENT-001` and subtype leaves own UUID, section, passenger and lifecycle state; removal reasons distinguish unload from death/discard. | Audit async load inbox order, duplicate UUID rejection, passenger trees, cross-chunk references, brain/goal/transient caches, pending teleports, removal callbacks and first ticking/tracking insertion. |
| Players and reconnect | `net.minecraft.server.players.PlayerList#loadPlayerData`, `net.minecraft.server.players.PlayerList#save`, `net.minecraft.server.level.ServerPlayer#readAdditionalSaveData`, `net.minecraft.server.level.ServerPlayer#addAdditionalSaveData` | `SURFACE-PLAYER-LIFECYCLE-001` and [its root inventory](player-lifecycle-roots.md) own join/remove/replacement phases; item and progression leaves own field meaning. | Complete the persisted-field ledger and clean-loss/crash/restart matrix already named by PlayerLifecycle, including dimension, death-before-respawn and missing stats/advancement files. |
| Saved world data and auxiliary progression | `net.minecraft.world.level.storage.SavedDataStorage#computeIfAbsent`, `net.minecraft.world.level.storage.SavedDataStorage#scheduleSave`, `net.minecraft.world.level.storage.SavedDataStorage#saveAndJoin`, `net.minecraft.server.ServerScoreboard#load`, `net.minecraft.server.ServerScoreboard#setDirty`, `net.minecraft.stats.ServerStatsCounter#save`, `net.minecraft.server.PlayerAdvancements#load`, `net.minecraft.server.PlayerAdvancements#save` | World-border, map, scoreboard/team, statistics and advancement owners define values and mutations; this family owns dirty collection, write completion and reconstruction. | Inventory every saved-data type and auxiliary file, dirty-clear timing, absent/corrupt/default/migration behavior, async write failure, cross-file atomicity and first listener/client convergence. |
| Reconstructed and transient state | `net.minecraft.server.level.ServerChunkCache#onChunkReadyToSend`, `net.minecraft.server.players.PlayerList#placeNewPlayer`, `net.minecraft.server.players.PlayerList#sendLevelInfo` | Client projection and protocol families own chunk/player convergence; simulation leaves identify RNG streams, caches, interpolators and transport/session data that may restart. | Classify every nonserialized behavior field as derived, reset-with-defined-first-result or incorrectly missing; compare uninterrupted controls at unload/reload, reconnect and restart boundaries. |

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

## Recovery procedure

1. Enumerate every behavior-owned field under the eight families and label it persisted,
   reconstructed, reset-with-defined-first-result or source-inconclusive.
2. Record its dirty/admission trigger, write ordering, load default/migration branch, reference
   resolution and first post-boundary consumer/projection.
3. Replay each field through chunk unload/reload, player disconnect/rejoin and full restart against
   an uninterrupted control; inject missing, malformed and failed-write inputs where the locked
   source exposes a branch.
4. Join every persistence result to WorldLifecycle, PlayerLifecycle, DataReload and client
   projection before promoting this surface; a list of save methods alone is not completion.
