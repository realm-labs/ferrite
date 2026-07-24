# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-REDSTONE-BLOCK-001` — Redstone block is a nonconducting constant source embedded in every ancient-city center

**Parent:** `BLK-001`, `BLK-002`, `BLK-003`, `BLK-004`, `BLK-005`, `RED-001`, `RED-002`,
`RED-003`, `PLY-005`, `PLY-006`, `ITM-004`, `ITM-006`, `ENV-003`, `WGEN-003`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, `PoweredBlock` and signal-query source, reports,
complete direct data and class-reference searches, all 1,212 decoded structure templates, ancient-
city pool/processor data and client assets exhaust the redstone-block identity's behavior and
projection.

**Applies when:**

`minecraft:redstone_block` is placed, written, queried as a signal or control input, moved, mined,
exploded, compacted or decompressed, emitted by an ancient-city center template, serialized or
rendered.

**Authoritative state:**

Redstone block is a property-free `PoweredBlock` with no block entity and sole state `11311`.
Registration selects map color `FIRE`, requires a correct tool for drops, sets destroy
speed/explosion resistance to `5.0/6.0`, selects `METAL` sound and explicitly makes the state never
a redstone conductor. It retains the default `HARP` note instrument, unit selection/collision/
occlusion shapes, emission zero, light dampening 15, friction `0.6`, speed/jump factors `1`, normal
piston reaction, full sturdy faces and no random ticking.

The direct block tag is exactly `mineable/pickaxe`. Because the identity belongs to none of the
locked incorrect-tier tags, the wooden-pickaxe tool rules already mark it correct for drops;
non-pickaxe tools do not. The matching ordinary block item is common, stacks to 64 and has no
nondefault component beyond the standard block-item identity/model/name set. Its locked item
registry raw ID is `747`; raw IDs remain version-adapter state rather than persistent identity.

**Transition and ordering:**

#### Constant signal and authoritative updates

`PoweredBlock#isSignalSource` always returns true and `ownSignal` always returns `15`. The inherited
ordinary `getSignal` delegates to that own signal without inspecting direction, so every face
returns `15`. The inherited `getDirectSignal` remains `0`. The registration's explicit
`isRedstoneConductor = never` means `SignalGetter#getSignal` returns the ordinary `15` directly and
never combines direct signal arriving into the redstone-block position through conductor relay.

The non-diode control-input route tests exact redstone-block identity before dust or another signal
source and returns `15`. The diode-only route admits only a diode's direct signal, so the same
redstone block returns `0` there. Comparator side-input selection uses the former route. A best-
neighbor query at an adjacent receiver position observes `15` from the redstone-block neighbor;
ordinary neighboring receivers observe the same direction-neutral `15` through their existing
admission logic.

The subtype overrides no placement, removal, neighbor, shape or tick callback. Therefore ordinary
placement/component/template writes commit state `11311` through the generic block-update flags,
then nested neighbor work reads the new signal; removal commits the replacement before its generic
notifications expose zero or a different source. Dust recomputation, diode scheduling, comparator
selection, piston quasi-connectivity, lamp/note transitions, update-budget behavior and client
correction remain with their named owners. Full-cube geometry does not promote this identity to a
conductor, and constant weak signal does not make its direct-signal query nonzero.

#### Placement, breaking, recipes and loot

Rotation and mirror are identity operations because the state has no property. Generic player
breaking admits the self-drop for any locked pickaxe and rejects it for an incorrect tool. The
block loot table performs one roll, returns one `redstone_block` behind `survives_explosion`, and
uses random sequence `minecraft:blocks/redstone_block`. Incorrect-tool removal reaches no correct-
tool loot path; an admitted explosion can suppress the single entry.

The reloadable processing graph names the block in exactly two recipes:

- shaped redstone-category recipe `redstone_block` consumes a full 3-by-3 grid of nine redstone
  items and returns one block;
- shapeless redstone-category recipe `redstone` consumes one block and returns nine redstone items.

Neither recipe declares a group. Each matching advancement has one OR requirement containing its
own `recipe_unlocked` criterion and possession of the recipe input, then grants only its matching
recipe. Recipe grant and inventory discovery are alternative unlock paths. No other locked recipe,
advancement, tag, non-block loot table or optional built-in-pack JSON record directly names the
block.

#### Ancient-city center placement

An exhaustive decompression scan of all 1,212 locked structure NBT files finds redstone block in
exactly the three ancient-city center templates and nowhere else. All three templates are
`18x31x41`, use one palette and contain two live property-free cells with no block NBT at identical
local positions `(14,3,29)` and `(15,5,3)`. Palette indices are `9`, `9` and `11` in centers
`1`, `2` and `3`; those storage-local indices have no runtime meaning after decode.

The `ancient_city/city_center` pool gives the three templates equal weight one and the structure's
named `city_anchor` selects one as its start. Consequently every admitted locked center choice
offers exactly two redstone-block cells; the choice changes surrounding circuitry but not their
count or local coordinates. `ancient_city_start_degradation` matches only deepslate bricks,
deepslate tiles and soul lantern, so it passes both cells unchanged. Its following protected-block
processor can still suppress a cell when the transformed live target belongs to
`features_cannot_replace`; chunk clipping or a failed generic write can suppress it independently.
Rotation, origin, clipping, processor order, sparse-cell order and neighboring redstone apparatus
remain with the ancient-city and generic template owners.

The complete server/client constant-pool sweep finds no current production consumer beyond
registration and `SignalGetter`'s control-input special case. Other exact field hits are item/
creative registration, data or model generation, the GameTest `pulseRedstone` helper and legacy
data-fix mappings; they introduce no additional runtime rule.

**Client projection:**

Chunk and block-update paths publish exact state `11311`; inventory paths project the namespaced
item through the configured registry whose locked raw ID is `747`. The client resolves one opaque
matching `cube_all` block model and the item directly selects that same model without a condition or
transform. Signal propagation itself has no dedicated packet: receiver states, sounds, particles
and corrections use their existing block/effect families.

**Branches and aborts:**

Ordinary versus component/template placement; source present versus removed; each queried face;
own/ordinary/direct/best/control signal; control's diode-only flag false versus true; conductor
relay rejected; correct versus incorrect tool; survived versus suppressed explosion loot;
compacting versus decompression; recipe-unlocked versus inventory discovery; three equal center
choices; every center transform, chunk clip, protected target and write result; server state versus
block/item projection are distinct.

**Constants and randomness:**

State `11311`; block raw ID `475`; item raw ID `747`; signal `15`; direct signal `0`; strength
`5.0/6.0`; emission `0`; dampening `15`; friction `0.6`; speed/jump `1`; stack `64`; compression
`9:1`; decompression `1:9`; three center entries each weight `1`; template size `18x31x41`; two live
cells per center at `(14,3,29)` and `(15,5,3)`. The block and signal queries consume no RNG.
Recipe/loot, center selection, processor and generic update owners retain their documented streams.

**Side effects:**

Generic state placement/removal and nested neighbor work; receiver state/tick/event changes;
conditional matching self loot; two recipe outputs and two recipe grants; two transformed ancient-
city block writes; ordinary persistence, map shading and opaque block/item projection.

**Gates:**

Placement/write authority; live signal query and direction; control-input mode; receiver-specific
admission; update flags and budget; correct harvest tool; explosion context; active recipe,
advancement and loot snapshots; ancient-city start/pool/template admission, transform, processors,
processing box and live protected target; client registry/model context.

**Boundary cases and quirks:**

The visually solid full cube is explicitly not a redstone conductor. It emits ordinary signal 15
on every face but inherited direct signal zero; the exact-identity control-input shortcut is
therefore observable and must not be replaced by a generic direct-signal query. A wooden pickaxe is
already correct because there is no higher-tier requirement. Every ancient-city center contains
the same two local cells even though its surrounding redstone ruin differs.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.PoweredBlock#isSignalSource`;
`net.minecraft.world.level.block.PoweredBlock#ownSignal`;
`net.minecraft.world.level.SignalGetter#getSignal`;
`net.minecraft.world.level.SignalGetter#getControlInputSignal`;
`net.minecraft.world.level.block.state.BlockBehaviour$BlockStateBase#isRedstoneConductor`;
`net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate#placeInWorld`;
`reports/blocks.json#minecraft:redstone_block`;
`reports/registries.json#minecraft:{block,item}/minecraft:redstone_block`;
`reports/minecraft/components/item/redstone_block.json`;
`data/minecraft/tags/block/mineable/pickaxe.json`;
`data/minecraft/loot_table/blocks/redstone_block.json`;
`data/minecraft/recipe/{redstone_block,redstone}.json`;
`data/minecraft/advancement/recipes/redstone/{redstone_block,redstone}.json`;
`data/minecraft/worldgen/template_pool/ancient_city/city_center.json`;
`data/minecraft/worldgen/processor_list/ancient_city_start_degradation.json`;
`data/minecraft/structure/**/*.nbt`;
`data/minecraft/structure/ancient_city/city_center/city_center_{1,2,3}.nbt`;
`assets/minecraft/blockstates/redstone_block.json`;
`assets/minecraft/models/block/redstone_block.json`;
`assets/minecraft/items/redstone_block.json`.

**Test vectors:**

Run `EXP-BLK-051` across state `11311` under ordinary/component/template writes; correct/incorrect
tools and explosion survival; all six faces, own/ordinary/direct queries, adjacent-receiver best-
neighbor queries and control queries; both control modes; conductor-relay probes; source
placement/removal beside dust, diodes, comparator,
piston, lamp and note block; both recipes and OR-unlocks across reload; every center choice,
transform, clip, protected-target and write boundary; save/reload and block/item models. Assert
exact state, constants, update/correction order, outputs, six live template cells, absence
boundaries and projection.

**Limits:**

Generic placement/breaking, nested neighbor updates, dust/diode/comparator/piston/receiver runtime,
recipes/advancements/loot, ancient-city jigsaw/processor/template placement, packet encoding and
client rendering remain with `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-UPDATE-001`,
`RED-UPDATE-001`, `RED-COMPARATOR-001`, receiver leaves, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ITM-LOOT-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`WGEN-JIGSAW-PROCESSORS-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001` and `CLI-006`.
