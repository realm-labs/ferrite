# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-IRON-MATERIAL-001` — Raw Iron, Iron Ingots and Iron Nuggets join ore, chest, mob, barter and trade acquisition to repair, crafting, beacon payment and armor trim

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-CONTAINER-MOVE-001`,
`ITM-CONTAINER-CLOSE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-SMITHING-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`, `ITM-ANVIL-001`, `ENT-001`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-004`,
`BLK-BEACON-001`, `BLK-BREAK-HOOK-001`, `BLK-RAW-STORAGE-001`,
`WGEN-PIPELINE-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations and components, six direct tag roles, 71 unique recipe
records and their advancement joins, Iron-Golem interaction, Beacon payment, three profession
trade sets, 30 container/entity/barter loot rows, two ore tables, Iron trim material, tool/armor
material construction, three ordinary ore placements and the ore-vein join determine every
Raw-Iron, Iron-Ingot and Iron-Nugget-specific branch. Generic stack, menu, recipe, cooking, anvil,
smithing, merchant, loot, worldgen and rendering algorithms remain with the cited owners.

**Applies when:**

A `minecraft:raw_iron`, `minecraft:iron_ingot` or `minecraft:iron_nugget` stack is created,
matched, cooked, crafted, used as repair, trim or Beacon material, offered to an Iron Golem, moved,
traded, persisted, synchronized or rendered; or when an Iron Ore, Deepslate Iron Ore, chest, pot,
Piglin barter, Iron Golem, Husk, Zombie or Zombie Villager is evaluated as one of the family's
acquisition sources before and after recipe, tag, advancement, loot, trade, trim-material or
resource reload.

**Authoritative state:**

Raw Iron, Iron Ingot and Iron Nugget have raw item IDs `931`, `932` and `1335`. All three register
as common nondamageable plain `Item` instances with maximum stack `64`. Their defaults contain the
common empty modifiers, enchantments and lore, item-break sound, translated name, direct item-model
key, repair cost, swing animation, tooltip display and use effects.

Iron Ingot additionally has default
`minecraft:provides_trim_material=minecraft:iron`. Raw Iron and Iron Nugget do not. None has food,
consumable, remainder, durability, equipment, tool, projectile, cooldown, inventory-tick or
identity-specific air-use behavior. Arbitrary valid ordinary component patches persist through the
generic stack owners. Except for trim assembly, the exact-item and tag tests below do not require
the default component map.

The complete direct item-tag memberships are:

| Item | Direct tags |
|---|---|
| Raw Iron | none |
| Iron Ingot | `beacon_payment_items`, `iron_tool_materials`, `repairs_chain_armor`, `repairs_iron_armor`, `trim_materials` |
| Iron Nugget | `metal_nuggets` |

Each of the three Iron-specific material/repair tags contains only Iron Ingot.
`beacon_payment_items` contains Netherite Ingot, Emerald, Diamond, Gold Ingot and Iron Ingot;
`trim_materials` contains eleven identities; and `metal_nuggets` contains Copper, Iron and Gold
Nuggets. These six roles remain independent of exact recipe ingredients and of the Ingot's actual
trim-material component.

**Transition and ordering:**

The identities do not override air or block use. A prototype stack returns generic `PASS` in air
and participates in ordinary block-first interaction. Container movement, pickup, dropping,
naming and valid component patching use generic owners. Operational behavior enters only through
the exact identity, tag, component, recipe, loot, trade and generation joins below.

### Material tags and repair

`ToolMaterial.IRON` is constructed with `incorrect_for_iron_tool`, durability `250`, mining speed
`6.0`, attack-damage bonus `2.0`, enchantment value `14` and `iron_tool_materials` as its repair
set. Iron Pickaxe, Shovel, Axe, Hoe, Sword and Spear therefore store the named live repair set whose
locked sole member is Iron Ingot.

`ArmorMaterials.IRON` has durability multiplier `15`, defense
Boots/Leggings/Chestplate/Helmet/Body `2/5/6/2/5`, enchantment value `9`, Iron equip sound, zero
toughness and knockback resistance, and `repairs_iron_armor` as its repair set. The four humanoid
Iron armor pieces use that set.

`ArmorMaterials.CHAINMAIL` also has durability multiplier `15`, defense `1/4/5/2/4`, enchantment
value `12`, Chain equip sound, zero toughness and knockback resistance, and
`repairs_chain_armor`. Its four humanoid pieces therefore also admit Iron Ingot. Iron Horse Armor
and Iron Nautilus Armor have no repairable component, despite both recycling to an Iron Nugget.

Anvil admission tests the offered item holder against the target's stored repair set. Any
Iron-Ingot identity, including one with ordinary patches or a removed/replaced trim-material
component, therefore repairs the six tools and eight humanoid armor pieces. Raw Iron and Iron
Nugget do not. Damage removal, prior-work cost, level pricing, output copying and commit remain
`ITM-ANVIL-001`; this leaf fixes the fourteen target/material joins.

### Direct Iron-Golem and Beacon use

Entity interaction first compares the held identity exactly with Iron Ingot. A different item
returns `PASS`. For Iron Ingot, the Iron Golem records old health and calls `heal(25.0)`. If health
does not increase—including an already-full Golem—it returns `PASS` without consuming the Ingot,
drawing randomness or playing a sound.

If health increases, the Golem draws two floats `A,B`, computes pitch
`1.0+(A-B)*0.2`, plays `entity.iron_golem.repair` at volume `1.0`, consumes one held Ingot through
living-entity consumption and returns `SUCCESS`. Infinite-material players lose zero. The
identity test ignores ordinary patches and the trim component; the heal is capped by the Golem's
current maximum health.

Beacon payment is instead live-tag-selected. Payment slot zero admits any current
`beacon_payment_items` member and has maximum size one; default or patched Iron Ingot qualifies.
Direct placement can therefore hold one. Quick-move sends a valid payment stack to the slot only
when it is currently empty and the source stack count is exactly one. A valid effect update
removes exactly one payment, while close returns/drops an unspent payment. Effect validation,
control packets and the deliberately weaker nonempty-at-commit check remain `BLK-BEACON-001` and
the container owners. Tag reload changes future slot admission, not an already inserted stack.

### Cooking and recycling

Six Iron-Ingot cooking records form three exact input pairs:

| Input | Furnace output / time / recipe XP | Blast Furnace output / time / recipe XP |
|---|---|---|
| Iron Ore | one Iron Ingot / `200` / `0.7` | one Iron Ingot / `100` / `0.7` |
| Deepslate Iron Ore | one Iron Ingot / `200` / `0.7` | one Iron Ingot / `100` / `0.7` |
| Raw Iron | one Iron Ingot / `200` / `0.7` | one Iron Ingot / `100` / `0.7` |

They omit cooking time and use the serializers' `200`/`100` defaults, share group `iron_ingot`,
and are admitted only by the matching Furnace or Blast Furnace recipe type. Smoker and Campfire
reject all six.

Two recycling records accept the same exact sixteen identities: six Iron tools, four humanoid
Iron armor pieces, Iron Horse Armor, Iron Nautilus Armor and four Chainmail armor pieces. Furnace
processing emits one Iron Nugget after `200` ticks and records XP `0.1`; Blast Furnace processing
emits one after `100` ticks and records XP `0.1`. Remaining durability, enchantments, trim and
other input patches do not affect identity matching and are discarded.

All eight outputs are default stacks, copy no input component and leave no remainder. Fuel,
progress/reset, capacity, recipe-use accounting, extraction and fractional XP remain
`ITM-FURNACE-001`.

### Crafting graph

Six compacting/decompression records are exact:

- a full `3×3` of nine Iron Ingots emits one Iron Block;
- one Iron Block emits nine Iron Ingots;
- a full `3×3` of nine Iron Nuggets emits one Iron Ingot;
- one Iron Ingot emits nine Iron Nuggets;
- a full `3×3` of nine Raw Iron emits one Raw Iron Block;
- one Raw Iron Block emits nine Raw Iron.

The Raw-Iron pair is also fixed from the block side by `BLK-RAW-STORAGE-001`.

Six shaped tool recipes use the live `iron_tool_materials` tag at every `X` and Stick at `#`:

| Result | Pattern |
|---|---|
| Iron Axe | `XX / X# /  #` |
| Iron Hoe | `XX /  # /  #` |
| Iron Pickaxe | `XXX /  #  /  # ` |
| Iron Shovel | `X / # / #` |
| Iron Spear | `  X /  #  / #  ` |
| Iron Sword | `X / X / #` |

Four humanoid Iron armor recipes instead use exact Iron Ingots: Boots `X X / X X`, Chestplate
`X X / XXX / XXX`, Helmet `XXX / X X`, and Leggings `XXX / X X / X X`.

The remaining 29 construction records are:

| Result | Exact pattern/ingredients and output |
|---|---|
| Activator Rail | `XSX / X#X / XSX`; `X` Ingot, `S` Stick, `#` Redstone Torch; `6` |
| Anvil | `III /  i  / iii`; `I` Iron Block, `i` Ingot; one |
| Blast Furnace | `III / IXI / ###`; `I` Ingot, `X` Furnace, `#` Smooth Stone; one |
| Bucket | `# # /  # ` of Ingots; one |
| Cauldron | `# # / # # / ###` of Ingots; one |
| Compass | ` #  / #X# /  # `; `#` Ingot, `X` Redstone; one |
| Crafter | `### / #C# / RDR`; `#` Ingot, `C` Crafting Table, `R` Redstone, `D` Dropper; one |
| Crossbow | `#&# / ~$~ /  # `; `#` Stick, `&` Ingot, `~` String, `$` Tripwire Hook; one |
| Detector Rail | `X X / X#X / XRX`; `X` Ingot, `#` Stone Pressure Plate, `R` Redstone; `6` |
| Flint and Steel | shapeless Ingot plus Flint; one |
| Heavy Weighted Pressure Plate | `##` of Ingots; one |
| Hopper | `I I / ICI /  I `; `I` Ingot, `C` Chest; one |
| Iron Bars | `### / ###` of Ingots; `16` |
| Iron Chain | vertical Nugget / Ingot / Nugget; one |
| Iron Door | three rows of two Ingots; `3` |
| Iron Trapdoor | full `2×2` of Ingots; one |
| Lantern | eight Iron Nuggets surrounding Torch; one |
| Lodestone | eight Chiseled Stone Bricks surrounding Ingot; one |
| Minecart | `# # / ###` of Ingots; one |
| Name Tag | diagonal ` X / # `; `X` live `metal_nuggets`, `#` Paper; one |
| Piston | `TTT / #X# / #R#`; `T` live `planks`, `#` Cobblestone, `X` Ingot, `R` Redstone; one |
| Rail | `X X / X#X / X X`; `X` Ingot, `#` Stick; `16` |
| Saddle | ` X  / X#X`; `X` Leather, `#` Ingot; one |
| Shears | diagonal ` # / # ` of Ingots; one |
| Shield | `WoW / WWW /  W `; `W` live `wooden_tool_materials`, lowercase `o` Ingot; one |
| Smithing Table | `@@ / ## / ##`; `@` Ingot, `#` live `planks`; one |
| Soul Lantern | eight Iron Nuggets surrounding Soul Torch; one |
| Stonecutter | ` I  / ###`; `I` Ingot, `#` Stone; one |
| Tripwire Hook | vertical Ingot / Stick / live `planks`; `2` |

Extra, missing or misplaced inputs fail. Live tag reload can broaden the six tool recipes, Name
Tag, Piston, Shield, Smithing Table and Tripwire Hook without changing exact Ingot/Nugget
positions. Patterns translate and mirror under the shaped-recipe owner where applicable.
Successful assembly emits the listed default stack, copies no arbitrary input patches and leaves
no remainder.

### Iron armor trim

Eighteen smithing-trim records—Bolt, Coast, Dune, Eye, Flow, Host, Raiser, Rib, Sentry, Shaper,
Silence, Snout, Spire, Tide, Vex, Ward, Wayfinder and Wild—each require their exact template, a
live `trimmable_armor` base and a live `trim_materials` addition. Default Iron Ingot matches the
addition.

Assembly then reads `provides_trim_material` from the actual addition stack. Default Iron Ingot
supplies the `minecraft:iron` holder. A patched Ingot can supply a different valid holder; removing
the component makes assembly return empty despite tag admission. An already identical
material-and-pattern trim also returns empty. Otherwise assembly copies the base at count one and
replaces its trim; occupied roles are consumed only when the preview is taken under
`ITM-SMITHING-001`.

The locked Iron trim-material record has asset name `iron`, description color `#ECECEC`,
translation `trim_material.minecraft.iron` (`Iron Material`) and one asset override: Iron
equipment uses `iron_darker`; every other admitted equipment asset uses `iron`. Tag admission,
stack-selected holder and registry/resource projection are separate gates.

### Recipe progression

Every corresponding recipe advancement has one requirement row, so the named possession criteria
are OR alternatives with the already-known-recipe criterion.

Iron Ingot possession unlocks Bucket, Crossbow, Heavy Weighted Pressure Plate, Hopper, Iron Bars,
Iron Block, all four humanoid Iron armor pieces, Iron Chain, Iron Door, Iron Nugget, Iron Trapdoor,
Lantern, Lodestone, Minecart, Shears, Shield and Smithing Table. Any live
`iron_tool_materials` member unlocks each of the six Iron tool recipes. Iron Nugget unlocks Iron
Chain, Iron Ingot from Nuggets and Lantern; any live `metal_nuggets` member also unlocks Name Tag.
Raw Iron unlocks its Furnace and Blast-Furnace records plus Raw Iron Block.

The corresponding exact ore unlocks each ore cooking record; Iron Block unlocks Anvil and its
decompression record; Raw Iron Block unlocks Raw Iron; and any one of the sixteen recyclable gear
identities unlocks each Nugget cooking record.

The remaining construction routes do not follow their Iron input: Rail unlocks Activator and
Detector Rails; Smooth Stone unlocks Blast Furnace; Water Bucket unlocks Cauldron; Redstone unlocks
Compass and Piston; Dropper unlocks Crafter; Flint or Obsidian unlocks Flint and Steel; Minecart
unlocks Rail; Leather unlocks Saddle; Soul Torch unlocks Soul Lantern; Stone unlocks Stonecutter;
String unlocks Tripwire Hook; and each trim recipe is unlocked by its exact template. Crossbow
also has String and Tripwire-Hook alternatives, Lodestone has a Lodestone alternative, and Name
Tag has Paper and Name-Tag alternatives.

Possessing Iron Ingot therefore does not alone unlock Anvil, Blast Furnace, Cauldron, Compass,
Crafter, Flint and Steel, Piston, Rail, Saddle, Soul Lantern, Stonecutter, Tripwire Hook, the six
Ingot cooking records, either recycling record or any trim recipe. Listener installation,
knowledge persistence and craft criteria remain `ITM-ADVANCEMENT-001`.

### Ore-break acquisition

Iron Ore and Deepslate Iron Ore are property-free `DropExperienceBlock` states `131` and `132`.
Both are pickaxe-mineable, require the stone tool tier for drops and specify zero break XP.

Each one-roll loot table first tests Silk Touch level at least one and emits its own default ore
block. Otherwise it creates one Raw Iron, applies Fortune's `ore_drops` formula and then explosion
decay. At Fortune zero there is no bonus draw. At positive level `L`, it draws
`D=nextInt(L+2)` and emits `max(1,D)` units before explosion: multiplier one has probability
`2/(L+2)` and each `2..L+1` has probability `1/(L+2)`. Explosion decay then tests every unit
independently. Silk bypasses the Raw-Iron Fortune and explosion path. Wrong-tool breaks emit
neither table output nor XP.

The named sequences are `minecraft:blocks/iron_ore` and
`minecraft:blocks/deepslate_iron_ore`. Tool admission, removal and item-entity placement remain
with `BLK-BREAK-HOOK-001` and `ITM-LOOT-001`.

### Container acquisition

For every row below, the listed pool draws an inclusive integer number of rolls; each roll selects
one weighted entry from the stated total and a selected Iron entry replaces its count with the
listed inclusive integer range.

| Loot table | Rolls | Iron entry | Weight / pool total | Count |
|---|---:|---|---:|---:|
| `chests/abandoned_mineshaft` | `2..4` | Ingot | `10/98` | `1..5` |
| `chests/bastion_bridge` | `1..2` | Ingot | `1/13` | `4..9` |
| `chests/bastion_bridge` | `2..4` | Nugget | `1/5` | `2..6` |
| `chests/bastion_other` | `2` | Ingot | `2/20` | `1..6` |
| `chests/bastion_other` | `3..4` | Nugget | `1/13` | `2..8` |
| `chests/bastion_treasure` | `3..4` | Ingot | `1/9` | `3..9` |
| `chests/buried_treasure` | `5..8` | Ingot | `20/35` | `1..4` |
| `chests/desert_pyramid` | `2..4` | Ingot | `15/247` | `1..5` |
| `chests/end_city_treasure` | `2..6` | Ingot | `10/89` | `4..8` |
| `chests/jungle_temple` | `2..6` | Ingot | `10/89` | `1..5` |
| `chests/nether_bridge` | `2..4` | Ingot | `5/78` | `1..5` |
| `chests/pillager_outpost` | `2..3` | Ingot | `3/22` | `1..3` |
| `chests/ruined_portal` | `4..8` | Nugget | `40/398` | `9..18` |
| `chests/shipwreck_treasure` | `3..6` | Ingot | `90/150` | `1..5` |
| `chests/shipwreck_treasure` | `2..5` | Nugget | `50/80` | `1..10` |
| `chests/simple_dungeon` | `1..4` | Ingot | `10/125` | `1..4` |
| `chests/stronghold_corridor` | `2..3` | Ingot | `10/101` | `1..5` |
| `chests/stronghold_crossing` | `1..4` | Ingot | `10/62` | `1..5` |
| `chests/trial_chambers/reward_common` | `1` | Ingot | `3/25` | `1..4` |
| `chests/village/village_armorer` | `1..5` | Ingot | `2/8` | `1..3` |
| `chests/village/village_taiga_house` | `3..8` | Nugget | `1/54` | `1..5` |
| `chests/village/village_toolsmith` | `3..8` | Ingot | `5/53` | `1..5` |
| `chests/village/village_weaponsmith` | `3..8` | Ingot | `10/107` | `1..5` |
| `chests/woodland_mansion` | `1..4` | Ingot | `10/175` | `1..4` |
| `pots/trial_chambers/corridor` | `1` | Ingot | `100/351` | `1..2` |

Trade Rebalance replaces Abandoned-Mineshaft, Desert-Pyramid, Jungle-Temple and Pillager-Outpost
tables. Their Iron row, rolls, weight and count stay identical. The first, third and fourth retain
the same denominator; Desert Pyramid changes only its relevant pool total from `247` to `237`, so
that row becomes `15/237`. Table selection, container installation, pot breaking, named cursors
and stack placement remain with the loot and structure owners.

### Mob drops and Piglin barter

Iron Golem's second one-roll pool has no player-kill condition and emits an inclusive uniform
`3..5` Iron Ingots. It has no Looting function; its independent Poppy pool does not affect this
count. The named sequence is `minecraft:entities/iron_golem`.

Husk, Zombie and Zombie Villager each have the same player-killed rare pool. With no positive
living-attacker Looting level, its chance gate succeeds below `0.025`. At positive level `L`, the
threshold is `0.035+0.01*(L-1)`, equivalently `0.025+0.01L`. A success then selects one of Iron
Ingot, Carrot or Potato with equal weight, so the Ingot's final conditional probability is one
third of that threshold until the threshold admits every chance draw. A selected Ingot is one
default stack; the Potato-only furnace-smelt function cannot affect it. Each entity uses its own
like-named sequence.

Piglin bartering takes one roll from total weight `469`. Iron Nugget has weight `10`, probability
`10/469`; selection emits an inclusive uniform `10..36` default Nuggets. Payment admission,
barter timing, admire state, table invocation and item throwing remain with the Piglin and loot
owners.

### Villager purchase

The baseline `smith/2/iron_ingot_emerald` record wants four exact default-predicate Iron Ingots and
gives one default Emerald. It has maximum uses `12`, Villager XP `10` and reputation discount
`0.05`, with no second cost, component predicate, output modifier or double-price enchantment.

`common_smith/level_2` expands to that purchase and the Bell sale. Toolsmith and Weaponsmith level
two each draw amount two from exactly those two candidates, making the Iron purchase guaranteed.
Armorer level two draws two without replacement from those two plus Chainmail Boots and Leggings
sales, giving the purchase inclusion probability `1/2`.

With Trade Rebalance, Armorer level two replaces its tag with eight armor sales and loses the
baseline Iron purchase. Armorer level one instead replaces its two-candidate tag with Coal and a
new Iron purchase while retaining amount two. The new record wants five Iron Ingots, gives one
Emerald, has uses `12`, XP `5`, discount `0.05`, and admits only Desert, Plains, Savanna, Snow and
Taiga Villager variants. It is guaranteed for those admitted variants and cannot construct an
offer for Jungle or Swamp variants. Toolsmith and Weaponsmith remain unchanged.

Ordinary component-patched Iron-Ingot stacks satisfy these empty input predicates. Offer
construction, selection cursors, demand, reputation, menu commit, exhaustion and restock remain
merchant-owned.

### Generation join and absence boundary

All `55` locked Overworld biomes schedule all three ordinary Iron placements:

- `ore_iron_upper` uses size-`9` ordered Stone/Deepslate replacement targets, air-exposure discard
  `0`, count `90`, in-square positions and trapezoid absolute height `80..384`;
- `ore_iron_middle` uses that same configured feature, count `10`, in-square and trapezoid
  absolute height `-24..56`;
- `ore_iron_small` uses size `4`, the same targets/discard, count `10`, in-square and uniform
  height from bottom `0` through absolute `72`.

Ore geometry, target reads, failed writes, feature order and biome scheduling remain
`WGEN-PIPELINE-001`.

Noise filling independently joins the Iron ore-vein resolver at inclusive Y `-60..-8`. After its
density/noise/admission gates it can write Deepslate Iron Ore, Raw Iron Block or Tuff; the strict
raw-block decision is the third position-seeded float below `0.02` after prior admission. Breaking
an emitted ore through the non-Silk path reaches Raw Iron above, while crafting an emitted Raw
Iron Block reaches nine Raw Iron through `BLK-RAW-STORAGE-001`. Generation never writes loose
item stacks.

Exhaustive direct data and server-class scans find no archaeology, fishing, cat-gift, brewing,
composting, fuel or dispenser branch for the three loose identities. Raw Iron occurs in no
container, entity, barter or trade table; Iron Nugget occurs in no entity or villager trade.
An exhaustive decoded-string scan of all `1,212` locked structure NBT files finds no stored
`raw_iron`, `iron_ingot` or `iron_nugget` identity. Fixed Iron blocks, equipment or template
palettes do not create a loose stack without one of the specified loot, recipe or interaction
paths.

**Persistence and reload boundary:**

The three stacks persist identity, count and arbitrary valid component patches. They do not own
Golem health, Beacon payment/effects, recipe progress/XP, recipe knowledge, anvil or Smithing
preview, loot cursor, merchant offer, barter state or worldgen state; those values persist with
their owners.

Recipe reload changes future cooking, crafting and smithing matches. Tag reload changes future
tool crafting, Name-Tag crafting, repair, Beacon and trim-recipe admission; exact-item positions
and Golem interaction do not change. Advancement, loot and trade reload change future listeners,
table evaluation and offer construction. Trim-material registry reload changes future material
decode/tooltip/asset selection. Existing stacks, inserted payments, offers, completed work, deaths,
barters and generated chunks are not replayed or rewritten. Resource reload independently changes
names, models, textures and trim palettes.

**Wire and client projection:**

Ordinary item-stack codecs publish raw registry IDs `931`, `932` and `1335`, count and component
patches. No family-specific packet exists. Recipe, tag, trim, loot, trade and advancement
identities use their generic synchronization owners.

The Ingredients tab orders the surrounding sequence:
`Coal, Charcoal, Raw Copper, Raw Iron, Raw Gold, Emerald, Lapis Lazuli, Diamond, Ancient Debris,
Quartz, Amethyst Shard, Copper Nugget, Iron Nugget, Gold Nugget, Copper Ingot, Iron Ingot,
Gold Ingot`. Each family item appears exactly once and in no second ordinary tab.

The English names are `Raw Iron`, `Iron Ingot` and `Iron Nugget`. Each item definition selects its
like-named `item/generated` model with a single like-named item texture. Generic transforms, glint,
count text and arbitrary tooltip components remain client-owned. Iron trim uses its description
and `iron`/`iron_darker` palettes rather than the loose Ingot texture.

**Branches and aborts:**

Three identities and six direct tags; default versus patched/removed trim component; exact versus
live-tag inputs; Golem wrong/full/damaged health; Beacon direct/quick placement and valid/invalid
commit; eight cooking, 45 crafting and 18 trim records; fourteen repair targets versus
nonrepairable Horse/Nautilus armor; wrong/correct tool, Silk, Fortune and explosion ore paths; 25
container rows and two datapack states; Golem versus three rare-undead pools; Piglin barter; three
profession sets under both trade states; three biome features and Iron ore-vein outcomes;
persistence/reload and wire/client projection are distinct.

**Constants and randomness:**

Raw IDs `931/932/1335`; stack `64`; Iron tool `250/6.0/2.0/14`; Iron and Chain armor multiplier
`15`, enchantments `9/12`; Golem heal `25`, sound volume `1`, pitch
`1+(A-B)*0.2`; compacting `9:1`; cooking `200/100`, XP `0.7/0.1`; ore states `131/132`, Fortune
multiplier above and break XP `0`; Golem drop `3..5`; rare-undead gate `0.025+0.01L` for positive
`L` then equal `1/3` selection; barter `10/469` and `10..36`; upper/middle/small sizes
`9/9/4`, counts `90/10/10`, heights `80..384/-24..56/bottom..72`; ore-vein band `-60..-8` and raw
block gate `<0.02`; trim color `#ECECEC`.

**Side effects:**

Golem healing, sound and consumption; Beacon payment; default cooking/crafting/smithing outputs;
recipe knowledge and XP; anvil repair; ore/container/entity/barter drops; merchant offers and
transactions; generated ore/raw-block terrain; ordinary inventory persistence, synchronization,
model rendering and trim projection.

**Gates:**

Selected identity/component patch; live item tags; Golem health; Beacon slot and validated powers;
machine recipe type/capacity; crafting grid; repair holder set; Smithing roles/component/existing
trim; advancement knowledge/listeners; tool/Silk/Fortune/explosion; table/pool/pack/sequence;
player-kill/attacker/Looting; Piglin and merchant state; biome/feature/ore-vein admission; client
resources.

**State read/written:**

Reads stack identity/components/tags, Golem health/randomness, Beacon menu/effect state, machine and
grid state, repair/trim/knowledge state, loot and death context, merchant/Piglin state,
feature/biome/ore-vein state and client resources. Writes only the health, sound, consumption,
payment, processing, result, repair, trim, knowledge, loot, offer, generated-terrain, stack, wire
and projection state listed above.

**Failure behavior:**

Wrong/full-health Golem use returns `PASS` and spends nothing. Invalid Beacon admission or effect
selection does not consume payment. Wrong machines, invalid grids, nonmembers, full results and
missing/unchanged trim previews produce no commit. Rejected repair does not consume levels/items.
Wrong-tool ore breaks emit nothing; Silk bypasses Raw Iron; failed explosion units vanish. Failed
loot/chance/weight/merchant/worldgen gates emit or write nothing. Reload affects future evaluation
only, and missing client resources cannot grant server behavior.

**Boundary cases and quirks:**

Iron Ingot's exact Golem role, five tag roles and trim component can diverge independently. A
full-health Golem calls capped healing but produces no success side effect. Iron Ingot repairs
Chainmail as well as Iron armor, yet Iron Horse/Nautilus armor can be recycled and cannot be
repaired. Iron Nugget shares Name-Tag admission with two other metals but only exact Iron Nugget
enters Iron Chain, Lantern, Soul Lantern and compacting. Trade Rebalance improves the Desert
Pyramid Iron row solely by shrinking its denominator and moves only the Armorer purchase from
level two to a variant-filtered level one. All three ordinary ore placements coexist in every
Overworld biome; the negative-Y ore vein remains separate.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.ToolMaterial#applyCommonProperties`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.item.Item$Properties#repairable(net.minecraft.tags.TagKey)`;
`net.minecraft.world.entity.animal.golem.IronGolem#mobInteract`;
`net.minecraft.world.inventory.BeaconMenu$PaymentSlot`;
`net.minecraft.world.inventory.BeaconMenu#quickMoveStack`;
`net.minecraft.world.inventory.BeaconMenu#updateEffects`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe#assemble`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe#applyTrim`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromTradeSet`;
`net.minecraft.world.entity.npc.villager.AbstractVillager#addOffersFromItemListingsWithoutDuplicates`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.level.levelgen.OreVeinifier`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,villager_trade,trade_set,trim_material,worldgen}`;
`reports/blocks.json#minecraft:{iron_ore,deepslate_iron_ore}`;
`reports/minecraft/components/item/{raw_iron,iron_ingot,iron_nugget,iron_pickaxe,iron_helmet,chainmail_helmet,iron_horse_armor,iron_nautilus_armor}.json`;
`data/minecraft/tags/item/{beacon_payment_items,iron_tool_materials,repairs_chain_armor,repairs_iron_armor,metal_nuggets,trim_materials}.json`;
`data/minecraft/trim_material/iron.json`;
`data/minecraft/recipe/{activator_rail,anvil,blast_furnace,bucket,cauldron,compass,crafter,crossbow,detector_rail,flint_and_steel,heavy_weighted_pressure_plate,hopper,iron_*,lantern,lodestone,minecart,name_tag,piston,rail,raw_iron,raw_iron_block,saddle,shears,shield,smithing_table,soul_lantern,stonecutter,tripwire_hook,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/recipes/**/*.json`;
`data/minecraft/loot_table/{blocks/{iron_ore,deepslate_iron_ore,raw_iron_block},entities/{iron_golem,husk,zombie,zombie_villager},chests/**/*.json,gameplay/piglin_bartering,pots/trial_chambers/corridor}.json`;
`data/minecraft/{villager_trade/smith/2/iron_ingot_emerald,tags/villager_trade/{common_smith,armorer,toolsmith,weaponsmith}/level_2,trade_set/{armorer,toolsmith,weaponsmith}/level_2}.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/{loot_table/chests/{abandoned_mineshaft,desert_pyramid,jungle_temple,pillager_outpost},villager_trade/armorer/1/iron_ingot_emerald,tags/villager_trade/armorer/{level_1,level_2}}.json`;
`data/minecraft/worldgen/{configured_feature/{ore_iron,ore_iron_small},placed_feature/{ore_iron_upper,ore_iron_middle,ore_iron_small},biome/*.json}`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/{raw_iron,iron_ingot,iron_nugget}.json`;
`assets/minecraft/models/item/{raw_iron,iron_ingot,iron_nugget}.json`;
`assets/minecraft/textures/item/{raw_iron,iron_ingot,iron_nugget}.png`;
`assets/minecraft/equipment/iron.json`;
`assets/minecraft/textures/trims/color_palettes/{iron,iron_darker}.png`;
`EXP-ITM-078`.

**Test vectors:**

Run `EXP-ITM-078` with default, ordinary-patched, removed-trim and alternate-trim-holder variants
through all six tag snapshots. Exercise Golem wrong/full/partially damaged/creative use, Beacon
direct/quick insertion and valid/invalid/close paths, all fourteen admitted repairs and Horse/
Nautilus rejects. Match, complete and extract all eight cooking, 45 crafting and eighteen trim
records across offsets, mirrors, tag changes, component states, result capacity and every unlock.

Break both ores through wrong/correct tools, Silk, Fortune and explosions. Materialize every
container row under both pack states, all four entity tables, Piglin barter and all three
profession sets across variants, candidate selections, economy and restock. Run all three placed
features in every one of 55 biomes, every Iron ore-vein result and the complete 1,212-template
decoded-string census. Persist and synchronize every stack/owner; assert IDs `931/932/1335`,
Ingredients order, generated models and Iron versus Iron-darker trim projection.

**Limits:**

Generic stack/use, menu movement/control, cooking timers/XP, crafting, advancement listeners,
anvil pricing, Smithing commit, Beacon effect validation, merchant economy, Piglin AI, block/
entity/container loot, ore geometry, noise filling, packet encoding and client rendering remain
with `ITM-001`, the container owners, `ITM-FURNACE-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ITM-SMITHING-001`, `BLK-BEACON-001`,
`ITM-LOOT-001`, the entity/mob owners, `BLK-BREAK-HOOK-001`, `BLK-RAW-STORAGE-001`,
`WGEN-PIPELINE-001`, the generic play protocol families and `CLI-006`.
