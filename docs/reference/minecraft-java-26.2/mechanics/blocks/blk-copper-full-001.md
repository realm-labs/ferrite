# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-COPPER-FULL-001` — Full copper blocks own weathering, waxing and copper-golem construction

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-005`, `RED-001`,
`PLY-005`, `PLY-006`, `ITM-003`, `ITM-004`, `ITM-006`, `ENT-001`, `MOB-004`, `MOB-005`,
`ENV-001`, `ENV-002`, `ENV-003`, `WGEN-002`, `WGEN-003`, `WGEN-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, weathering collections, random-tick and tool code, tags,
recipes, advancements, loot, the copper-golem construction path, an exhaustive structure-template
scan and client assets close the 24 property-free full copper blocks.

**Applies when:**

Any unwaxed or waxed age of `copper_block`, `cut_copper` or `chiseled_copper` is placed, randomly
ticked, scraped, waxed, mined, exploded, crafted, used below a note block, selected by a copper
golem or sulfur cube, generated from a structure template, persisted or rendered.

**Authoritative state:**

All 24 registrations have no state property or block entity. The 12 unwaxed registrations are
`WeatheringCopperFullBlock` instances whose report type is
`minecraft:weathering_copper_full`; the 12 waxed registrations are ordinary `Block` instances
whose report type is `minecraft:block`.

| Collection and age | Block ID | Item ID | State | Map color | Instrument |
|---|---:|---:|---:|---|---|
| copper block, unaffected | 1034 | 118 | 27782 | `COLOR_ORANGE` | `TRUMPET` |
| copper block, exposed | 1035 | 119 | 27783 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| copper block, weathered | 1036 | 120 | 27784 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| copper block, oxidized | 1037 | 121 | 27785 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |
| copper block, waxed unaffected | 1038 | 122 | 27786 | `COLOR_ORANGE` | `TRUMPET` |
| copper block, waxed exposed | 1039 | 123 | 27787 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| copper block, waxed weathered | 1040 | 124 | 27788 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| copper block, waxed oxidized | 1041 | 125 | 27789 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |
| cut copper, unaffected | 1044 | 137 | 27792 | `COLOR_ORANGE` | `TRUMPET` |
| cut copper, exposed | 1045 | 138 | 27793 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| cut copper, weathered | 1046 | 139 | 27794 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| cut copper, oxidized | 1047 | 140 | 27795 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |
| cut copper, waxed unaffected | 1048 | 141 | 27796 | `COLOR_ORANGE` | `TRUMPET` |
| cut copper, waxed exposed | 1049 | 142 | 27797 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| cut copper, waxed weathered | 1050 | 143 | 27798 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| cut copper, waxed oxidized | 1051 | 144 | 27799 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |
| chiseled copper, unaffected | 1052 | 129 | 27800 | `COLOR_ORANGE` | `TRUMPET` |
| chiseled copper, exposed | 1053 | 130 | 27801 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| chiseled copper, weathered | 1054 | 131 | 27802 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| chiseled copper, oxidized | 1055 | 132 | 27803 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |
| chiseled copper, waxed unaffected | 1056 | 133 | 27804 | `COLOR_ORANGE` | `TRUMPET` |
| chiseled copper, waxed exposed | 1057 | 134 | 27805 | `TERRACOTTA_LIGHT_GRAY` | `TRUMPET_EXPOSED` |
| chiseled copper, waxed weathered | 1058 | 135 | 27806 | `WARPED_STEM` | `TRUMPET_WEATHERED` |
| chiseled copper, waxed oxidized | 1059 | 136 | 27807 | `WARPED_NYLIUM` | `TRUMPET_OXIDIZED` |

Each age's waxed identity copies the corresponding unwaxed physical properties. Cut and chiseled
registrations additionally use full-copy properties from the same-age copper block. All 24
therefore require a correct tool for drops, have destroy speed `3.0`, explosion resistance `6.0`
and `COPPER` sound, and retain the ordinary full-solid defaults: unit selection, collision and
occlusion shapes; full sturdy faces and solid redstone conduction; emission zero; light dampening
15; friction 0.6; speed/jump factors 1; normal piston reaction; and no comparator output or block
entity. `COPPER` sound uses volume/pitch `1/1` and break/step/place/hit/fall sound IDs
`400/401/402/403/404`.

All 24 are direct `mineable/pickaxe` and `needs_stone_tool` members. Their standard block items are
common, stack to 64 and carry no nonstandard default component.

**Transition and ordering:**

#### Random weathering

Only unaffected, exposed and weathered unwaxed states report randomly ticking. Oxidized unwaxed
states have no next collection member, and every waxed state is a nonweathering `Block`, so those
15 states never enter the callback.

An admitted callback first consumes `nextFloat()` and continues only when it is strictly below
`0.05688889`; equality and larger values are no-ops. It then traverses the locked
`BlockPos.withinManhattan(position,4,4,4)` order, skips the source and stops when that ordered
iterator first reaches Manhattan distance above four. A neighbor contributes only when its block
implements `ChangeOverTimeBlock` and its age enum has the same runtime class as the source.
Consequently all unwaxed stages from all 15 registered copper collections can contribute; waxed
blocks and unrelated change-over-time enums cannot.

Encountering any younger copper age returns empty immediately without a second random draw.
Otherwise the scan counts same-age blocks as `same` and older blocks as `older`, then computes

```text
c = (older + 1) / (older + same + 1)
threshold = c * c * modifier
modifier = 0.75 for unaffected, 1.0 for exposed or weathered
```

A second `nextFloat()` advances the block only when strictly below `threshold`; equality fails.
Success maps to the next member of the same collection, copies shared state properties and calls
`setBlockAndUpdate`, ignoring its Boolean result. These 24 states have no properties, so the target
is exactly exposed, weathered or oxidized in the same full/cut/chiseled collection. The callback
emits no direct sound, particle or game event, and there is no rollback after a rejected write.

#### Honeycomb and axe transformations

The code-built 15-collection honeycomb map contains every one of this leaf's 12 unwaxed-to-waxed
pairs. `ITM-HONEYCOMB-001` owns the full transaction: a mapped use triggers the server-player
criterion, directly shrinks the stack by one, offers the same-age waxed state with flags `11`,
then emits contextual `BLOCK_CHANGE` and level event `3003`, even if the write failed. These
property-free blocks need no shared-property conversion and never enter the double-chest companion
branch. Already waxed states pass.

For an axe, a main-hand use first passes when the player's offhand item has `BLOCKS_ATTACKS` and
secondary use is not active. Otherwise transformation priority is strip, previous weather age,
then wax removal; none of these blocks is strippable. Exposed, weathered and oxidized unwaxed
states map exactly one age backward. Before returning that target, evaluation plays
`minecraft:item.axe.scrape` (sound ID `89`, Blocks source, volume/pitch `1/1`) and emits level
event `3005`, excluding a nonnull player. Unaffected unwaxed states have no previous entry and
pass.

Every waxed state maps to its same-age unwaxed identity. Evaluation first plays
`minecraft:item.axe.wax_off` (ID `90`) and emits event `3004`. For either successful axe mapping,
a server player then receives `ITEM_USED_ON_BLOCK` against the still-original state; the method
offers the target with flags `11`, emits contextual `BLOCK_CHANGE`, damages the axe by one in the
used-hand equipment slot when a player exists, and returns `SUCCESS`. Sound/event and criterion
precede the ignored-result write; game event and durability follow it, so none rolls back on
failure. A null player skips criterion and durability but retains effects, write, game event and
success.

The `husbandry/wax_on` criterion recognizes honeycomb use against all 12 unwaxed family blocks.
The child `husbandry/wax_off` criterion recognizes any of the seven axe items against all 12
waxed family blocks. Ordinary age scraping triggers the generic item-used criterion call but does
not satisfy the waxed-block predicate.

#### Copper-golem construction

Only the eight full `copper_block` age/wax identities are direct members of both block and item
`copper`; cut and chiseled blocks are absent. The block tag's sole production runtime consumer is
`CarvedPumpkinBlock`. Its cached copper base pattern is a space over one tagged block; its full
pattern is a carved pumpkin or jack o'lantern over one tagged block. On a nonidentity pumpkin
placement, golem matching is attempted in snow, iron, then copper order.

When the copper pattern matches, an entity factory miss leaves the pattern unchanged. A successful
factory result starts the shared transaction:

1. both cached pattern cells are offered as air with flags `2` in width-major then height-major
   order; every Boolean is ignored and each captured old state emits level event `2001`;
2. the golem is snapped to cached pattern cell `(0,0,0)` plus `(0.5,0.05,0.5)`, yaw/pitch zero,
   and `addFreshEntity` is called with its result ignored;
3. every server player selected inside the golem AABB inflated by `5` receives
   `summoned_entity`, then both pattern positions receive `updateNeighborsAt(...,AIR)`;
4. the captured source copper identity maps to the corresponding same-age copper chest, preserving
   wax when unpaired. Pumpkin facing selects chest facing and adjacent-chest state selects its type;
   when a copper chest pair forms, the helper reconciles to the less oxidized member and removes
   wax if the pair's wax states differ. The chest is offered at the source position with flags `2`
   and its result is ignored;
5. the cached source supplies the golem weather age directly when unwaxed or through the reverse
   wax map when waxed. `CopperGolem#spawn` writes that synced age and plays
   `minecraft:entity.copper_golem.spawn` (sound ID `435`).

The golem is not marked waxed by this path, even when the source block was waxed; wax preservation
belongs to the replacement chest only. Failed air, entity or chest commits do not roll back later
steps. Copper-golem AI, entity metadata, weathering, persistence and chest runtime remain with
their entity and chest owners.

#### Loot, recipes and acquisition

Each of the 24 block loot tables has one one-roll pool, offers the exact matching item through
`survives_explosion`, and uses random sequence `minecraft:blocks/<identity>`. The generic break
transaction admits that table only after the stone-tier correct-tool check; Silk Touch, Fortune
and state components add no branch.

Exactly 53 locked recipes produce a family item:

- one 3-by-3 copper-ingot recipe produces one unaffected copper block;
- each of the eight age/wax full blocks produces four matching cut blocks through both a 2-by-2
  shaped recipe and stonecutting;
- each of the eight stages produces matching chiseled copper through two vertical matching cut
  slabs, stonecutting the full block for four, or stonecutting the cut block for one;
- each of the 12 waxed identities has a shapeless same-identity unwaxed block plus honeycomb
  recipe producing one, grouped as `waxed_copper_block`.

Their standard recipe advancements mirror recipe unlock or ingredient possession. A further 75
recipes consume these identities into outside-family outputs: matching grates, bulbs, cut slabs
and stairs; nine-ingot decompression of unaffected copper block or waxed copper block; and
`bolt_armor_trim_smithing_template` duplication, where either of those two blocks is the center
material among seven diamonds and the retained template. No villager trade or nonblock loot table
directly emits one of the 24 items.

#### Tags, sulfur cubes and structures

All 24 items are direct `sulfur_cube_archetype/slow_flat` members. That archetype installs
horizontal/vertical powers `0.4125/0.105`, hit/push sounds, push cooldown `0.9`, impulse threshold
`0.03`, resistance additions `0.5/0.5`, bounciness `0.4000000059604645`, multiplied-total friction
`-0.5999999940395355` and air drag `-0.8999999985098839`. Matching, equipment and contact
handling remain with `ENT-KNOCKBACK-001`. The eight-member `copper` item tag has no production
runtime class consumer; it remains visible through tag synchronization, commands, recipes and
reload-selected predicates. All 24 have fire odds zero, do not ignite from lava and have fuel
time zero.

An exhaustive decode of all 1,212 bundled structure templates finds 23,354 raw family cells in
149 distinct trial-chamber templates:

| Identity | Template pairs | Raw cells |
|---|---:|---:|
| copper block | 1 | 20 |
| waxed copper block | 140 | 11,192 |
| waxed oxidized copper | 111 | 9,552 |
| oxidized cut copper | 4 | 37 |
| waxed cut copper | 2 | 4 |
| waxed oxidized cut copper | 84 | 2,351 |
| waxed chiseled copper | 59 | 195 |
| waxed oxidized chiseled copper | 3 | 3 |

These total 404 identity/template pairs; the other 16 identities have zero cells. Exact-value
scans find no configured-feature or processor-list record that directly names one of the 24.
Template presence is not unconditional generation: pool selection, processors, transforms,
clipping, placement admission and write results remain with `WGEN-PIPELINE-001`.

**Client projection:**

Terrain and block updates publish states `27782..27789`, `27792..27799` and `27800..27807`.
Each of the 12 unwaxed identities has a one-variant `cube_all` block model using its like-named
texture. Each waxed blockstate aliases the corresponding same-age unwaxed model rather than
declaring a separate waxed model. All 24 item selectors choose that same resolved block model.
There is no tint, emissive layer, rotation or special material case.

As note-block substrates, the four ages select sound IDs `1171`, `1172`, `1174` and `1173` for
unaffected, exposed, weathered and oxidized trumpet respectively; waxing does not change the
instrument. Note admission, pitch and block-event projection remain with the note-block owner.

The Building Blocks tab places the copper collections after smooth quartz and amethyst block.
For each of two passes, the code visits full block, chiseled, grate, cut, stairs, slab, bars, door,
trapdoor, bulb and chain collections; the first pass emits four unwaxed ages and the second emits
four waxed ages. This leaf's 24 items are therefore interleaved with other copper collections
rather than appearing as one contiguous run.

**Branches and aborts:**

Three collections, four ages and waxed/unwaxed identity; nine ticking versus 15 nonticking states;
first-gate miss/equality/hit; younger-neighbor early abort; same/older counts; second-gate
miss/equality/hit; accepted/rejected weather write; honeycomb/axe blocking, mapping and failure
order; seven axe identities and advancement predicates; full-tag versus cut/chiseled pattern
admission; entity factory/add/chest write outcomes, paired-chest age/wax reconciliation and source
wax; correct/wrong tool and explosion survival; every recipe, tag snapshot, template, persistence,
note, tab and block/item model are distinct.

**Constants and randomness:**

Block IDs `1034..1041`, `1044..1059`; item IDs `118..125`, `129..144`; states
`27782..27789`, `27792..27807`; strength `3.0/6.0`; emission/dampening/friction/speed/jump
`0/15/0.6/1/1`; stack `64`; weather scan radius `4`; first gate `<0.05688889`; age modifier
`0.75/1.0`; axe flags/events `11/3005/3004`; honeycomb flags/event `11/3003`; golem flags/events
`2/2001`, offset `0.5/0.05/0.5`, criterion radius `5`; recipes `53/75`; template
pairs/templates/cells `404/149/23,354`; all sound IDs and slow-flat values as listed.

**Side effects:**

Ordinary placement/removal and conditional self loot; random age write; honeycomb stack shrink and
wax effects; axe scrape/wax-off effects, criterion, block change and durability; wax
advancements; destructive pumpkin pattern, copper chest replacement, golem entity/metadata,
criterion and spawn sound; recipe and advancement outputs; sulfur-cube equipment behavior;
structure-template writes; palette persistence; note, map, sound, tab and model projection.

**Gates:**

Random-tick chunk/activity/rate admission; current exact age and wax identity; two strict floats and
the complete radius-four age population; generic reach/hand/build and item-use admission; offhand
blocking intent; honeycomb/axe identity and durability; carved-pumpkin pattern search and entity
factory; block/chest/entity write authority; correct harvest tier and explosion context; active
recipe, advancement, loot, tag, archetype and structure snapshots; valid registry, pack and client
connection context.

**Boundary cases and quirks:**

Oxidized unwaxed and every waxed state never random-tick. A single younger neighbor suppresses
weathering without consuming the second float, while all 15 unwaxed copper collections can affect
the ratio. Random aging and both tool paths ignore their state-write result and do not roll back
effects. Axe scrape/wax-off sound and particles occur before the advancement criterion and write.
Only full copper blocks build copper golems. A waxed base produces a waxed unpaired chest but an
unwaxed golem at the copied age. The golem transaction ignores all material write/entity-add
results. Waxed assets deliberately reuse unwaxed models and textures.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.level.block.WeatheringCopperCollection`;
`net.minecraft.world.level.block.WeatheringCopperFullBlock#codec`;
`net.minecraft.world.level.block.WeatheringCopperFullBlock#randomTick`;
`net.minecraft.world.level.block.WeatheringCopperFullBlock#isRandomlyTicking`;
`net.minecraft.world.level.block.ChangeOverTimeBlock#changeOverTime`;
`net.minecraft.world.level.block.ChangeOverTimeBlock#getNextState`;
`net.minecraft.world.level.block.WeatheringCopper#getNext`;
`net.minecraft.world.level.block.WeatheringCopper#getPrevious`;
`net.minecraft.world.level.block.WeatheringCopper#getChanceModifier`;
`net.minecraft.world.item.HoneycombItem#useOn`;
`net.minecraft.world.item.AxeItem#useOn`;
`net.minecraft.world.item.AxeItem#evaluateNewBlockState`;
`net.minecraft.world.item.AxeItem#spawnSoundAndParticle`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#trySpawnGolem`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#getOrCreateCopperGolemBase`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#getOrCreateCopperGolemFull`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#spawnGolemInWorld`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#replaceCopperBlockWithChest`;
`net.minecraft.world.level.block.CarvedPumpkinBlock#getWeatherStateFromPattern`;
`net.minecraft.world.level.block.CopperChestBlock#getFromCopperBlock`;
`net.minecraft.world.entity.animal.golem.CopperGolem#spawn`;
`net.minecraft.world.level.block.state.properties.NoteBlockInstrument`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube#matchingArchetypes`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
`net.minecraft.world.level.block.FireBlock#bootStrap`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes`;
`reports/blocks.json#minecraft:{copper_block,cut_copper,chiseled_copper and age/wax prefixes}`;
`reports/registries.json#minecraft:{block,item,sound_event,game_event}`;
`reports/minecraft/components/item/{copper_block,cut_copper,chiseled_copper and age/wax prefixes}.json`;
`data/minecraft/tags/block/{copper,mineable/pickaxe,needs_stone_tool}.json`;
`data/minecraft/tags/item/{copper,sulfur_cube_archetype/slow_flat}.json`;
`data/minecraft/sulfur_cube_archetype/slow_flat.json`;
`data/minecraft/loot_table/blocks/{copper_block,cut_copper,chiseled_copper and age/wax prefixes}.json`;
`data/minecraft/{recipe,advancement}/**/*copper*.json`;
`data/minecraft/advancement/husbandry/{wax_on,wax_off}.json`;
`data/minecraft/structure/**/*.nbt`;
`data/minecraft/worldgen/**/*.json`;
`assets/minecraft/blockstates/{copper_block,cut_copper,chiseled_copper and age/wax prefixes}.json`;
`assets/minecraft/models/block/{copper_block,cut_copper,chiseled_copper and age prefixes}.json`;
`assets/minecraft/items/{copper_block,cut_copper,chiseled_copper and age/wax prefixes}.json`.

**Test vectors:**

Run `EXP-BLK-073` across all 24 states and physical/tag/model queries; every first/second float
boundary and radius-four age population drawn from all 15 collections; honeycomb, axe and
failed-write order; every copper-golem pattern/factory/write/chest-pair/source-wax outcome; all
loot, recipes, advancements, tags and 1,212 templates; save/reload, note instruments, creative
order and vanilla-client convergence. Assert exact IDs, constants, mappings, read/draw/effect/write
order, ignored results and negative cut/chiseled golem joins.

**Limits:**

Generic random-tick admission, placement/breaking, note-block runtime, recipes/advancements/loot,
sulfur-cube contact, structure placement, block/item protocol and rendering remain with
`SIM-RANDOM-001`, `BLK-PLACE-001`, `BLK-BREAK-001`, `ITM-HONEYCOMB-001`,
`ITM-CRAFT-001`, `ITM-ADVANCEMENT-001`, `ITM-LOOT-001`, `ENT-KNOCKBACK-001`,
`WGEN-PIPELINE-001`, `PROTO-PLAY-CLIENTBOUND-BLOCK-001` and `CLI-006`. Copper chests, the copper
golem entity and the other 12 copper collections retain their separate owners or catalog status.
