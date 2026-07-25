# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-NETHERITE-MATERIAL-001` — Netherite Scrap and Netherite Ingots join Ancient-Debris processing and Bastion loot to upgrades, repair, Beacon payment and armor trim

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-CONTAINER-MOVE-001`,
`ITM-CONTAINER-CLOSE-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-FURNACE-001`, `ITM-SMITHING-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ENCHANT-001`,
`ITM-ANVIL-001`, `ENT-001`, `BLK-BEACON-001`, `BLK-ANCIENT-DEBRIS-001`,
`BLK-BEACON-STORAGE-001`, `WGEN-PIPELINE-001`, `WGEN-JIGSAW-BASTION-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the two locked registrations/component maps, four Ingot-only direct tag roles,
Netherite tool/armor constants, 35 recipe and recipe-advancement joins, four Bastion loot rows,
Ancient-Debris processing/generation, Beacon and trim data, the complete structure census and exact
client resources determine every Scrap- and Ingot-specific branch. Generic stack, dropped-item
damage, cooking, crafting, Smithing, anvil, Beacon, advancement, loot, structure, worldgen and
rendering algorithms remain with the cited owners.

**Applies when:**

A `minecraft:netherite_scrap` or `minecraft:netherite_ingot` stack is created, damaged as a dropped
item, matched, crafted, used as a Smithing addition, used as repair/Beacon/trim material, moved,
persisted, synchronized or rendered; or when Ancient Debris processing, a Netherite Block or one
of the three producing Bastion tables supplies the family before and after recipe, tag,
advancement, loot, trim-material or resource reload.

**Authoritative state:**

Netherite Ingot and Netherite Scrap have raw item IDs `937` and `938`. Both are common,
nondamageable, maximum-`64` plain `Item` stacks with the ordinary empty modifiers, enchantments
and lore, break sound, translated name, direct item-model key, repair cost, swing animation,
tooltip display and use effects.

Both additionally have
`minecraft:damage_resistant={types:"#minecraft:is_fire"}`. Netherite Ingot alone has
`minecraft:provides_trim_material=minecraft:netherite`. Neither has food, consumable, remainder,
durability, equipment, tool, projectile, cooldown, inventory-tick or identity-specific air use.
Arbitrary valid ordinary patches persist through generic stack owners.

The complete direct item-tag memberships are:

| Item | Direct tags |
|---|---|
| Netherite Ingot | `beacon_payment_items`, `netherite_tool_materials`, `repairs_netherite_armor`, `trim_materials` |
| Netherite Scrap | none |

The two Netherite material/repair tags each contain only Netherite Ingot. The payment and trim
tags have five and eleven locked values. Tag identity and the Ingot's trim-material component are
independent reloadable selectors.

**Transition and ordering:**

Prototype stacks return ordinary `PASS` in air and participate in block-first interaction.
Operational behavior enters only through the component, exact identity, live tag, recipe, loot,
Beacon and Ancient-Debris joins below.

### Fire-resistant dropped stacks

The locked `is_fire` damage-type closure is exactly `in_fire`, `campfire`, `on_fire`, `lava`,
`hot_floor`, `sulfur_cube_hot`, `unattributed_fireball` and `fireball`.
`ItemEntity#hurtServer` asks the stack's damage-resistance component before changing dropped-item
health. Either family stack therefore rejects every source in that live tag and follows the
ordinary item-entity damage/removal transaction for other admitted damage. This is item damage
resistance, not fireproof inventory, crafting, container or placed-block state.

Changing the damage-type tag changes future damage admission. Removing/replacing the component
changes that individual stack; it does not change exact recipe or tag matching. Finished damage
events are not replayed.

### Material repair, upgrades, Beacon and trim

`ToolMaterial.NETHERITE` uses `incorrect_for_netherite_tool`, durability `2031`, mining speed
`9.0`, attack-damage bonus `4.0`, enchantment value `15` and `netherite_tool_materials`.
Netherite Pickaxe, Shovel, Axe, Hoe, Sword and Spear therefore store that live repair set.

`ArmorMaterials.NETHERITE` has durability multiplier `37`, defense
Boots/Leggings/Chestplate/Helmet/Body `3/6/8/3/19`, enchantment value `15`, Netherite equip sound,
toughness `3.0`, knockback resistance `0.1` and `repairs_netherite_armor`. The four humanoid
Netherite armor pieces use that repair set. Netherite Horse and Nautilus Armor have no repairable
component and no recycling recipe.

Anvil material admission therefore accepts any live Netherite-Ingot member for six tools and four
humanoid armor pieces. Ordinary Ingot patches and its trim component do not affect that holder
test. Scrap and the two mount armors are rejected. Damage removal, cost, output copying and commit
remain `ITM-ANVIL-001`.

Exactly twelve `smithing_transform` records require Netherite Upgrade Smithing Template, the
like-kind Diamond base and live `netherite_tool_materials` addition:

| Recipe/result family |
|---|
| Axe, Hoe, Pickaxe, Shovel, Spear and Sword |
| Boots, Chestplate, Helmet and Leggings |
| Horse Armor and Nautilus Armor |

Each emits the like-kind Netherite identity with the base component patch applied over the
result identity's defaults. The twelve matching recipe advancements independently unlock from
possession of a live Netherite-tool-material member or prior recipe unlock. Template/base/addition role checks,
first matching recipe, preview, component transfer, consumption order and level event `1044`
remain with the Smithing owners.

Beacon payment slot zero admits any live `beacon_payment_items` member and caps at one. A valid
effect commit consumes one Ingot; close returns/drops an unspent payment. Direct insertion,
single-count quick movement, power validation and the nonempty commit boundary remain
`BLK-BEACON-001` and the container owners.

Netherite Ingot's trim component resolves `minecraft:netherite`: asset `netherite`, description
`Netherite Material` colored `#625859`, with Netherite-equipment override
`netherite_darker`. Each of the 18 armor-trim templates has one `smithing_trim` record whose
addition is live `trim_materials`; the corresponding recipe advancement unlocks from its template,
not from the Ingot. Admission can therefore succeed while assembly fails if the Ingot has no
trim-material component. An alternate valid holder changes the produced trim; an identical
existing trim returns empty.

### Processing and crafting graph

Ancient Debris supplies Netherite Scrap through exactly two records:

| Recipe | Machine | Output | Time | XP |
|---|---|---|---:|---:|
| `netherite_scrap` | Furnace | one Scrap | `200` | `2.0` |
| `netherite_scrap_from_blasting` | Blast Furnace | one Scrap | `100` | `2.0` |

Both omit a group and cooking time, so the concrete serializers supply the listed defaults.
Smoker and Campfire reject them. Each recipe advancement has one OR requirement: exact
Ancient-Debris possession or prior unlock of the matching recipe. Input patches are discarded and
the output is a default Scrap stack.

Three ordinary crafting records close the pair:

- `netherite_ingot` is shapeless group `netherite_ingot`; exactly four Netherite Scraps and four
  Gold Ingots in any eight cells emit one default Netherite Ingot;
- `netherite_block` is a full shaped `3×3` of nine exact Netherite Ingots and emits one Netherite
  Block;
- `netherite_ingot_from_netherite_block` is shapeless group `netherite_ingot`; one exact Netherite
  Block emits nine default Netherite Ingots.

Their recipe advancements use the ordinary two-way OR form. Ingot construction unlocks from exact
Scrap, block compression from exact Ingot, and decompression from exact Netherite Block; prior
unlock of the same recipe is the alternative in each case. The block-side properties, loot,
Beacon-base role and recipe join remain `BLK-BEACON-STORAGE-001`.

Together, two cooking, three crafting, twelve transform and eighteen trim records make 35 family
recipes. There are also 35 matching recipe advancements: two cooking, three crafting, twelve
transform and eighteen template-selected trim unlocks. Machine/grid/role matching uses identity or
the live tags stated above; ordinary component patches do not affect exact ingredients. Every
ordinary crafting/cooking output is default and copies no input patch.

### Initial acquisition and worldgen join

Four direct Bastion chest entries emit the loose family:

| Table, pool | Rolls / direct-entry total weight | Family entry |
|---|---|---|
| `chests/bastion_treasure`, pool 0 | `3` / `112` | Ingot count one, weight `15` |
| `chests/bastion_treasure`, pool 0 | `3` / `112` | Scrap count one, weight `8` |
| `chests/bastion_other`, pool 0 | `1` / `89` | Scrap count one, weight `4` |
| `chests/bastion_hoglin_stable`, pool 0 | `1` / `100` | Scrap count one, weight `8` |

Set-count writes explicit one in all four rows. The named table supplies the random sequence;
weighted competitors, roll order, container filling and Bastion placement remain generic.
Trade Rebalance does not replace these records.

Ancient Debris is the only processing input. Its large size-`3`, trapezoid-Y `8..24` and small
size-`2`, above/below-bottom/top-`8` scattered-ore placements both occur in Nether Wastes,
Crimson Forest, Warped Forest, Soul Sand Valley and Basalt Deltas. Target, exposure, attempt and
write semantics are exactly `BLK-ANCIENT-DEBRIS-001`/`WGEN-PIPELINE-001`; worldgen emits no loose
Scrap or Ingot until a later player-controlled cooking/crafting transaction.

An exhaustive string census of all 1,212 locked structure NBT files finds no Netherite-Ingot or
Netherite-Scrap identity. The four table rows are therefore the only direct initial loose-stack
sources. No locked mob drop, archaeology, fishing, gift, merchant, Piglin, compost, fuel, brewing
or dispenser branch emits or consumes either identity.

**Persistence and reload boundary:**

Stacks persist identity, count and patches. They do not own item-entity health, machine progress/
XP, recipe knowledge, anvil/Smithing preview, Beacon state, loot cursor or worldgen state.
Recipe/tag/advancement/loot/trim reload changes only future evaluation in its domain. Completed
damage, cooking, crafting, Smithing, repair, payment, loot and chunks are not replayed. Resource
reload independently changes names, models, textures and trim palettes.

**Wire and client projection:**

Generic stack codecs publish raw IDs `937/938`, count and component patches. No family packet
exists. The Ingredients tab places Netherite Scrap then Netherite Ingot after Gold Ingot and before
Stick; each appears once and in no other ordinary tab.

English names are `Netherite Scrap` and `Netherite Ingot`. Each item definition selects a
like-named `item/generated` model and texture with no tint, condition or special renderer.
Netherite trim uses `netherite`/`netherite_darker` palettes and equipment layers rather than the
loose Ingot texture.

**Branches and aborts:**

Two identities; shared/patched fire resistance; four Ingot-only tag roles; ten admitted and two
rejected repair targets; twelve transform and eighteen trim/component paths; Beacon payment; two
cooking and three crafting records; four weighted chest rows; two placed features in five biomes;
zero template identities; persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

IDs `937/938`; stack `64`; eight fire damage types; tool `2031/9/4/15`; armor
`37`, `3/6/8/3/19`, `15/3/0.1`; cooking `200/100`, XP `2`; construction `4+4:1`,
compression `9:1`, decompression `1:9`; twelve transforms; eighteen trims; loot rolls, totals,
weights and counts above; Ancient-Debris sizes `3/2`; trim `#625859`.

**Side effects:**

Dropped-item damage rejection; cooking, crafting, recipe knowledge, transformation, repair, trim,
Beacon payment, chest loot, stack persistence/synchronization and client projection.

**Gates:**

Selected identity/component patch; live damage/item tags; damage source; machine/grid/Smithing
roles and capacity; repair holder set; trim component/existing trim; Beacon menu; advancement
listeners; loot table/pool/weight; biome/feature; client resources.

**State read/written:**

Reads the stack, live tags, damage, processing/grid/Smithing/anvil/Beacon/knowledge/loot/worldgen
and resource states named above. Writes only the dropped-item, processing, result, knowledge,
equipment result, repair, trim, payment, loot, stack, wire and projection state listed above.

**Failure behavior:**

Fire-tagged damage leaves the item entity unchanged. Wrong machines/grids/roles, missing tag
members, full outputs, invalid/unchanged trim and rejected repair do not commit or consume. Invalid
Beacon admission/effect selection does not spend payment. Failed loot weight emits nothing; failed
Ancient-Debris worldgen produces no later material. Reload affects future evaluation only.

**Boundary cases and quirks:**

Scrap and Ingot are both fire-resistant, but Scrap has no direct tag role. Ingot's four tag roles
and trim component can diverge independently. Netherite upgrades have twelve ordinary recipe
advancements even though their template identity owns the template-side selection. Mount armor can
be upgraded to Netherite but cannot be repaired with Ingot. Worldgen creates only Ancient Debris;
all loose family acquisition is a later processing, crafting or Bastion-loot event.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ToolMaterial`;
`net.minecraft.world.item.equipment.ArmorMaterials`;
`net.minecraft.world.entity.item.ItemEntity#hurtServer`;
`net.minecraft.world.inventory.BeaconMenu$PaymentSlot`;
`net.minecraft.world.item.crafting.SmithingTransformRecipe`;
`net.minecraft.world.item.crafting.SmithingTrimRecipe`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer,loot_table,advancement,trim_material}`;
`reports/minecraft/components/item/{netherite_ingot,netherite_scrap,netherite_pickaxe,netherite_spear,netherite_helmet,netherite_horse_armor,netherite_nautilus_armor}.json`;
`data/minecraft/tags/damage_type/is_fire.json`;
`data/minecraft/tags/item/{beacon_payment_items,netherite_tool_materials,repairs_netherite_armor,trim_materials}.json`;
`data/minecraft/trim_material/netherite.json`;
`data/minecraft/recipe/{netherite_ingot,netherite_block,netherite_ingot_from_netherite_block,netherite_scrap,netherite_scrap_from_blasting,netherite_*_smithing,*_armor_trim_smithing_template_smithing_trim}.json`;
`data/minecraft/advancement/recipes/{building_blocks,combat,misc,tools}/netherite*.json`;
`data/minecraft/advancement/recipes/misc/*_armor_trim_smithing_template_smithing_trim.json`;
`data/minecraft/loot_table/chests/{bastion_treasure,bastion_other,bastion_hoglin_stable}.json`;
`data/minecraft/worldgen/{configured_feature/{ore_ancient_debris_large,ore_ancient_debris_small},placed_feature/{ore_ancient_debris_large,ore_debris_small},biome/{nether_wastes,crimson_forest,warped_forest,soul_sand_valley,basalt_deltas}}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/{netherite_ingot,netherite_scrap}.json`;
`assets/minecraft/models/item/{netherite_ingot,netherite_scrap}.json`;
`assets/minecraft/textures/item/{netherite_ingot,netherite_scrap}.png`;
`assets/minecraft/equipment/netherite.json`;
`assets/minecraft/textures/trims/color_palettes/{netherite,netherite_darker}.png`;
`EXP-ITM-080`.

**Test vectors:**

Run `EXP-ITM-080` with default, ordinary-patched, removed/replaced fire-resistance and trim
components and baseline/removed/broadened damage/item tags. Apply all eight fire types and
nonmembers. Exercise all ten admitted repairs, both mount rejects, Beacon direct/quick/commit/close
paths, every transform base and near miss, and all trim materials/existing-trim states.

Cook both Ancient-Debris records; craft all three ordinary records; test every exact, duplicate,
offset, capacity and unlock boundary across all 35 recipes and advancements. Materialize all four
Bastion rows at controlled cursors, run both Ancient-Debris placements in all five Nether biomes
and scan all 1,212 templates. Persist/synchronize every stack and owner; assert IDs `937/938`,
Ingredients order, generated models and Netherite versus Netherite-darker trim projection.

**Limits:**

Generic stack/use, item-entity damage, cooking timers/XP, crafting, advancement listeners,
Smithing selection/assembly/commit, anvil pricing, Beacon effect validation, loot evaluation,
Bastion placement, Ancient-Debris feature execution, packet encoding and client rendering remain
with `ITM-001`, `ENT-001`, `ITM-FURNACE-001`, `ITM-RECIPE-001`,
`ITM-ADVANCEMENT-001`, `ITM-SMITHING-001`, `ITM-RECIPE-SERIALIZER-001`,
`ITM-ANVIL-001`, `BLK-BEACON-001`, `ITM-LOOT-001`, `WGEN-JIGSAW-BASTION-001`,
`BLK-ANCIENT-DEBRIS-001`, `WGEN-PIPELINE-001` and `CLI-001`.
