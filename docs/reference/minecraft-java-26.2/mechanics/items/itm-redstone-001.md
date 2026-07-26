# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-REDSTONE-001` — Redstone Dust joins wire placement, ore and loot acquisition to devices, brewing, trade and armor trims

**Parent:** `SIM-004`, `SIM-005`, `SIM-RANDOM-001`, `BLK-001`, `BLK-002`,
`BLK-003`, `BLK-005`, `BLK-PLACE-001`, `BLK-BREAK-001`,
`BLK-BREAK-HOOK-001`, `PLY-002`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `PLY-BREAK-001`, `RED-001`, `RED-002`,
`RED-UPDATE-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`,
`ITM-RECIPE-001`, `ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-BREW-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-SMITHING-001`, `ITM-SMITHING-TEMPLATE-001`,
`ITM-ANVIL-001`, `BLK-REDSTONE-BLOCK-001`, `ENT-001`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `ENV-001`, `ENV-002`, `ENV-003`,
`WGEN-PIPELINE-001`, `WGEN-STRUCTURE-MINESHAFT-001`,
`WGEN-STRUCTURE-STRONGHOLD-001`, `WGEN-STRUCTURE-WOODLAND-MANSION-001`,
`WGEN-JIGSAW-VILLAGES-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/block registration, wire and Ore states, source hooks, loot,
recipes, advancements, brewing, trade, trim and worldgen records, all `1,212` decoded templates and
exact client resources determine every Redstone-Dust-specific branch. Generic placement, redstone
propagation, breaking, processing, crafting, brewing, loot, merchant, Smithing, worldgen,
persistence, packet and rendering algorithms retain their cited owners.

**Applies when:**

`minecraft:redstone` is placed as wire, recovered from wire, mined or cooked from either Ore,
looted, gifted or traded, compacted, consumed in a device recipe or potion extension, used as armor
trim, moved, renamed, persisted, synchronized or rendered before and after data, experiment or
resource reload.

**Authoritative state:**

Redstone Dust is raw item ID `745`, a common nondamageable custom-named block item bound to
`minecraft:redstone_wire`, with maximum stack `64`. Its default components include
`provides_trim_material=minecraft:redstone`; its sole direct item tag is `trim_materials`. It has no
food, consumable, remainder, fuel, compost, equipment, durability, projectile, cooldown or
inventory-tick branch.

Redstone Wire is block ID `202` with `1,296` states `4011..5306` over power `0..15` and four
`none/side/up` connections. The default state is `5171`, all connections `none`, power zero. It is
no-collision, instant-break and piston `DESTROY`.

The source Ores are `RedStoneOreBlock` instances:

| block | block/item IDs | unlit/lit state | map/sound | strength |
|---|---:|---:|---|---:|
| Redstone Ore | `271/99` | `6882/6881` | `STONE` / Stone | `3/3` |
| Deepslate Redstone Ore | `272/100` | `6884/6883` | `DEEPSLATE` / Deepslate | `4.5/3` |

Both use `BASEDRUM`, require an iron-tier-or-better pickaxe, emit light `9` only while lit and are
direct `redstone_ores` members plus the appropriate pickaxe/tool-tier tags.

**Transition and ordering:**

### Wire placement, update and recovery

The custom block item routes use-on through ordinary block placement. Wire survives when the block
below has a sturdy top face or is a Hopper. Placement starts from power-zero default state,
recomputes connection shape and power, writes the admitted state, consumes one item outside
infinite-material mode and runs the vertical/horizontal neighbor transaction.

An isolated ordinary placement normalizes to the four-side cross. Redstone connection routing,
same/up/down neighbor sampling, power recomputation, default-versus-experimental evaluator,
cross/dot player toggle, recursive notifications and transient ordering remain exactly
`RED-UPDATE-001`. Failed support, replaceability, collision, world-border, feature or component
placement consumes nothing.

Wire has one loot roll and returns one default Redstone behind `survives_explosion`, random
sequence `minecraft:blocks/redstone_wire`; it has no correct-tool gate. Support loss drops through
the same generic block-resource path. Thus successful ordinary placement followed by safe breaking
is count-preserving, while explosion decay can suppress recovery.

### Ore interaction, break, cooking and generation

Attack, a non-careful step, admitted item use and projectile contact invoke the Ore interaction
hook. It spawns exposed-face Redstone particles and changes unlit to lit with flags `3`; repeated
interaction leaves the lit state installed. Lit Ore random-ticks back to the matching unlit state
and client animation can emit the same particle family. These state changes do not emit loose Dust.

Each Ore loot table first selects one matching Ore under Silk Touch level at least one. Otherwise
it draws base count `4..5`, adds uniform Fortune bonus `0..L`, then applies explosion decay. A
correct non-Silk player break independently draws XP `1..5`; Silk suppresses that XP. Wrong tools
yield neither ordinary loot nor break XP. Lit and unlit states share the same loot identity and
sequence `minecraft:blocks/{redstone_ore,deepslate_redstone_ore}`.

Four cooking records emit one default Redstone with recipe XP `0.7`: Furnace accepts each Ore at
default time `200`, Blast Furnace at default time `100`; Smoker and Campfire reject both. Every
record has an exact-Ore advancement and input patches are not copied.

One configured size-`8`, air-discard-`0` feature targets ordered Stone state `6882` then Deepslate
state `6884`. Two placed wrappers run in all `55` Overworld biomes:

| placed ID | count | height |
|---|---:|---|
| `ore_redstone` | `4` | uniform above-bottom `0` through absolute `15` |
| `ore_redstone_lower` | `8` | trapezoid above-bottom `-32..32` |

Each then applies in-square and biome; geometry, replacement and chunk writes remain
`WGEN-PIPELINE-001`.

### Direct loose-item acquisition

Witch pool `1` has one guaranteed Redstone row, base `4..8` plus living-attacker Looting
`round(LU)`. It is independent of the Witch's weighted `1..3`-roll pool.

Six chest pools emit Redstone:

| table / pool | rolls | weight / pool total | count |
|---|---:|---:|---:|
| chests/abandoned_mineshaft `1` | `2..4` | `5/98` | `4..9` |
| chests/simple_dungeon `1` | `1..4` | `15/125` | `1..4` |
| chests/stronghold_corridor `0` | `2..3` | `5/101` | `4..9` |
| chests/stronghold_crossing `0` | `1..4` | `5/62` | `4..9` |
| chests/woodland_mansion `1` | `1..4` | `15/175` | `1..4` |
| chests/village/village_temple `0` | `3..8` | `2/19` | `1..4` |

Trade Rebalance replaces Abandoned Mineshaft but preserves the Redstone pool, row, denominator,
rolls and count; its added enchanted-book pool is independent. Hero-of-the-Village Cleric gift has
one roll choosing Redstone at `1/2`, count one. No other entity, fishing, barter, archaeology or
bundled loot table directly emits loose Redstone.

An exhaustive decoded scan finds zero loose Redstone stack entries across all `1,212` templates.
Template Redstone-Wire cells are block-state payload owned by their named structure leaves, not
pre-created Dust stacks; natural Ores remain feature-driven.

### Forty-six recipe joins and progression

Redstone participates in `46` recipes. The `28` direct records are:

- four Ore cooking records and shapeless Redstone-Block decompression emit Redstone;
- Redstone Block compression consumes `9`;
- Clock, Compass, Detector Rail, Dispenser, Dropper, Note Block, Piston, Powered Rail, Redstone
  Torch and Repeater consume one each;
- Crafter and Observer consume two each;
- Redstone Lamp and Target consume four each; and
- each of eight unwaxed/waxed Copper-Bulb oxidation variants consumes one.

All `18` generic armor-trim Smithing records additionally admit one Redstone through the live
`trim_materials` tag. All `46` records have advancements. Direct Redstone possession satisfies the
inventory alternative for Clock, Compass, Dropper, Note Block, Piston, Redstone Block, Redstone
Torch and Target (`8` direct unlocks); the remaining listeners use another exact input or template.

### Fourteen brewing edges, merchant and trim

Exact Redstone is the ingredient for `14` potion-holder edges: Water to Mundane, plus Night Vision,
Invisibility, Fire Resistance, Leaping, Slowness, Turtle Master, Swiftness, Water Breathing,
Poison, Regeneration, Strength, Weakness and Slow Falling to their Long variants. Each edge applies
to Potion, Splash Potion and Lingering Potion containers. One ingredient can transform up to three
admitted bottles after the generic `400`-tick transaction; it is not brewing fuel.

Cleric level one selects both candidates, so one Emerald to two Redstone is guaranteed. Maximum
uses are `12`, omitted Villager XP decodes to `1`, reputation discount is `0.05`; Trade Rebalance
does not replace the record or set.

The default trim provider resolves material `minecraft:redstone`, description color `#971607` and
asset `redstone`. Tag removal rejects it from all `18` trim recipes; provider removal or replacement
changes resolution independently after admission. Successful Smithing consumes one and writes the
holder into copied armor.

**Persistence and reload boundary:**

Stacks, wire/Ore states, machines, knowledge, brewing stands, containers, offers and trimmed armor
persist with their owners. Recipe, advancement, loot, tag, potion, trade, trim and worldgen reload
changes future evaluation only. The redstone experiment switch changes future wire evaluation;
installed state remains. Completed placement, mining, cooking, loot, brewing, crafting, trading,
trimming and generated chunks are not replayed. Resource reload independently changes projection.

**Wire and client projection:**

Generic stack publication uses item ID `745`; Wire states use `4011..5306`, and Ore block/item/
state IDs are `271/99/6881..6882` and `272/100/6883..6884`. No Redstone-specific packet exists.
English names are `Redstone Dust`, `Redstone Ore`, `Deepslate Redstone Ore` and
`Redstone Material`.

Loose Dust selects one untinted `item/generated` flat. Wire uses multipart dot/line/side/up models
and power-dependent tint; Ore uses unlit/lit cube variants. Ingredients orders Nether Wart,
Redstone Dust, Glowstone Dust; the Redstone Blocks tab begins Dust, Torch, Block, Repeater and
Comparator. Trim projection has one `redstone` palette and `29` compatible armor item-model
overlays plus equipped atlas projection.

**Branches and aborts:**

Default/patched stack; placement/support/recovery and default/experimental wire evaluator; two
Ores with lit/unlit and Silk/Fortune/explosion/XP paths; four cooks/two placements; Witch, six
chests/one overlay and Cleric gift; 46 recipes/listeners/eight direct unlocks; 14 brewing edges;
merchant; trim tag/provider; zero loose template stacks; persistence/reload/wire/client branches
are distinct.

**Constants and randomness:**

Dust ID `745`, stack `64`; Wire block ID `202`, states `4011..5306`, default `5171`, power
`0..15`; Ore block/item/states `271/99/6881..6882`, `272/100/6883..6884`, strength `3/3`,
`4.5/3`, light `9`, drops `4..5 + U[0,L]`, XP `1..5`; cooking `200/100/0.7`; feature
size/discard `8/0`, placement `4/8`; direct chest rows `6`; recipes/listeners/unlocks
`46/46/8`; brewing edges `14`; trade `1 Emerald:2 Dust`, uses/XP/discount `12/1/0.05`;
trim color `#971607`, recipes/models `18/29`; templates/loose-stack matches `1212/0`.

**Side effects:**

Wire placement/drop and network updates; Ore lighting/particles/loot/XP and worldgen; machine,
Witch/chest/gift outputs; device and knowledge results; potion conversion; merchant transaction;
trimmed armor; durable state, synchronization and exact client projection.

**Gates:**

Placement support/replaceability/collision/border; wire evaluator and neighbor topology; Ore
interaction/tool/Silk/Fortune/explosion; machine/input/capacity; feature/biome/replacement; loot
selection; recipe/tag/result capacity/knowledge; brewing holder/container/fuel/timer; profession/
level/set; trim tag/provider; registry/state/stack decode and client resources.

**Boundary cases and quirks:**

Redstone Dust is a custom-named block item whose placed block has `1,296` topology/power states,
not an ordinary inert material. Safe wire breaking returns the same loose identity; explosion can
erase it. Ore contact lights but does not drop Dust, and lit/unlit share loot. Fortune uses uniform
bonus rather than `ore_drops`. One ingredient owns Water-to-Mundane plus thirteen duration
extensions, all across three container types. Tag admission and trim-provider resolution are
independent.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items#createBlockItemWithCustomItemName(net.minecraft.world.level.block.Block)`;
`net.minecraft.world.item.BlockItem#useOn`;
`net.minecraft.world.level.block.RedStoneWireBlock`;
`net.minecraft.world.level.block.RedStoneOreBlock`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes(net.minecraft.world.item.alchemy.PotionBrewing$Builder)`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/blocks.json#minecraft:{redstone_wire,redstone_ore,deepslate_redstone_ore}`;
`reports/registries.json#minecraft:{block,item,recipe,recipe_serializer,loot_table,advancement,potion,villager_trade,trade_set,trim_material,worldgen}`;
`reports/minecraft/components/item/redstone.json`;
`data/minecraft/tags/{block/{redstone_ores,mineable/pickaxe,needs_iron_tool},item/trim_materials}.json`;
`data/minecraft/trim_material/redstone.json`;
`data/minecraft/loot_table/{blocks/{redstone_wire,redstone_ore,deepslate_redstone_ore},entities/witch,chests/{abandoned_mineshaft,simple_dungeon,stronghold_corridor,stronghold_crossing,woodland_mansion,village/village_temple},gameplay/hero_of_the_village/cleric_gift}.json`;
`data/minecraft/recipe/{clock,compass,*copper_bulb,crafter,detector_rail,dispenser,dropper,note_block,observer,piston,powered_rail,redstone*,repeater,target,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/{villager_trade/cleric/1/emerald_redstone,tags/villager_trade/cleric/level_1,trade_set/cleric/level_1}.json`;
`data/minecraft/worldgen/{configured_feature/ore_redstone,placed_feature/ore_redstone*,biome/*.json}`;
`data/minecraft/structure/**/*.nbt`; `assets/minecraft/**/*redstone*`;
`RED-UPDATE-001`; `BLK-REDSTONE-BLOCK-001`; `ITM-SMITHING-TEMPLATE-001`;
`ITM-RECIPE-SERIALIZER-001`; `WGEN-PIPELINE-001`; `EXP-ITM-092`.

**Test vectors:**

Run `EXP-ITM-092` across default/patched Dust, every placement/support/recovery and wire-evaluator
branch, both Ores under interaction/lit/tick/tool/Silk/Fortune/explosion/XP boundaries, four cooks,
two placements, Witch, all chest/gift paths, 46 recipes/listeners, 14 brewing edges, Cleric offer
and all 18 trims under independent tag/provider reload. Scan all templates, persist/reload/
synchronize owners and assert IDs/states, names, flat/wire/Ore models, tint, palette and both tab
orders.

**Limits:**

Generic placement, redstone propagation, breaking, processing, crafting, brewing, loot, merchant,
Smithing, feature, packet and renderer control flow remains with cited owners. Redstone Block,
device outputs, potion holders, templates and trimmed equipment retain their dedicated owners.
This leaf fixes the exact loose/block-item identity, source/sink joins, absences and projection.
