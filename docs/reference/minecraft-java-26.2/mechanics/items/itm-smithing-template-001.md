# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SMITHING-TEMPLATE-001` — Smithing templates bind exact upgrade or trim patterns to renewable acquisition identities

**Parent:** `PLY-005`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`, `ENT-001`,
`WGEN-003`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, template subclass behavior, recipes, trim-pattern records,
loot, advancements and client assets close all 19 smithing-template identities. Generic crafting,
smithing, loot, progression, entity-drop and world-generation algorithms retain their owners.

**Applies when:**

A scoped stack is created, duplicated, inserted into a smithing template slot, selected by loot,
persisted, reloaded, shown in a tooltip or smithing screen, or projected in an item/tab context.

**Authoritative state:**

| Item ID | Identity | Rarity | Configured initial acquisition |
|---:|---|---|---|
| `1458` | netherite upgrade | uncommon | bastion treasure guaranteed; bridge, hoglin-stable or other chest `1/10` |
| `1459` | sentry armor trim | uncommon | pillager-outpost chest `1/4`, count two |
| `1460` | dune armor trim | uncommon | desert-pyramid chest `1/7`, count two |
| `1461` | coast armor trim | uncommon | each shipwreck map, supply or treasure chest `1/6`, count two |
| `1462` | wild armor trim | uncommon | jungle-temple chest `1/3`, count two |
| `1463` | ward armor trim | rare | ancient-city chest `1/20` |
| `1464` | eye armor trim | rare | stronghold corridor `1/10`; library guaranteed |
| `1465` | vex armor trim | rare | woodland-mansion chest `1/2` |
| `1466` | tide armor trim | uncommon | elder-guardian entity loot `1/5` |
| `1467` | snout armor trim | uncommon | every bastion chest family `1/12` |
| `1468` | rib armor trim | uncommon | Nether-fortress chest `1/15` |
| `1469` | spire armor trim | rare | End-city treasure `1/15` |
| `1470` | wayfinder armor trim | uncommon | rare trail-ruins archaeology `1/12` |
| `1471` | shaper armor trim | uncommon | rare trail-ruins archaeology `1/12` |
| `1472` | silence armor trim | epic | ancient-city chest `1/80` |
| `1473` | raiser armor trim | uncommon | rare trail-ruins archaeology `1/12` |
| `1474` | host armor trim | uncommon | rare trail-ruins archaeology `1/12` |
| `1475` | flow armor trim | uncommon | ominous-vault reward `9/40` |
| `1476` | bolt armor trim | uncommon | normal-vault reward `1/16` |

Each full ID appends `_smithing_template`. All 19 are maximum-stack-64
`SmithingTemplateItem` instances with the table rarity and otherwise ordinary default components.
They add no use, use-on, inventory-tick, consumption, durability, equipment, projectile or
crafting-remainder hook.

**Transition and ordering:**

#### Duplication

Every identity has the same shaped duplication layout:

```text
#S#
#C#
###
```

`#` is diamond, `S` is one matching template and `C` is its identity-specific core. Seven diamonds,
one source template and one core produce two default matching templates:

| Core | Templates |
|---|---|
| netherrack | netherite upgrade, rib |
| cobblestone | sentry, coast, vex |
| sandstone | dune |
| mossy cobblestone | wild |
| cobbled deepslate | ward, silence |
| End stone | eye |
| prismarine | tide |
| blackstone | snout |
| purpur block | spire |
| terracotta | wayfinder, shaper, raiser, host |
| breeze rod | flow |
| copper block or waxed copper block | bolt |

The transaction therefore has a net gain of one template and consumes the source stack's ordinary
components rather than copying them to the two default results. Pattern matching, grid consumption
and output publication remain `ITM-RECIPE-001`/`ITM-CRAFT-001`.

Each duplication recipe has one recipe advancement whose sole OR requirement accepts either
inventory possession of that exact template or prior unlock of the same recipe, then rewards that
recipe. Each of the 18 trim recipes has an analogous exact-template possession/unlock advancement.
Each of the twelve Netherite transform recipes also has an advancement: its one OR requirement
accepts either a live `netherite_tool_materials` member or prior unlock of that transform, then
rewards the matching recipe.

#### Smithing selection

Netherite upgrade is the required template in exactly twelve `smithing_transform` recipes:
diamond axe, boots, chestplate, helmet, hoe, horse armor, leggings, nautilus armor, pickaxe,
shovel, spear and sword. Each requires the like-kind diamond base and live
`netherite_tool_materials` addition, and returns the like-kind netherite result while the serializer
preserves the base component patch.

Each armor-trim identity is the required template in one same-named `smithing_trim` recipe. The
other roles are live `trimmable_armor` and `trim_materials` members, and the record selects its
same-named `trim_pattern` data record. Those 18 records all have `decal=false`, a same-named asset ID
and description. The serializer returns a count-one base copy with the resolved material/pattern
pair, or empty when the base already has that exact pair.

`ITM-SMITHING-001` owns first-key match, preview/error state, template/base/addition consumption and
event `1044`; `ITM-RECIPE-SERIALIZER-001` owns transform component transfer and trim assembly. This
leaf fixes which template identity admits which records and pattern.

#### Initial acquisition

All table probabilities are per evaluation of the named one-roll pool. Outputs not explicitly
listed as count two are default count one. Exact weighted branches are:

- sentry uses empty weight three versus template weight one; dune uses empty six versus template
  one; coast uses empty five versus template one in each of three shipwreck tables; wild uses
  empty two versus template one;
- ward and silence share ancient-city weights `4/80` and `1/80`; eye uses empty nine versus
  template one in corridors and a template-only library pool; vex uses equal empty/template
  weights; tide uses empty four versus template one;
- snout uses empty eleven versus template one in bridge, hoglin-stable, other and treasure bastion
  tables; netherite upgrade independently uses empty nine versus template one in the first three
  and a template-only treasure pool;
- rib and spire each use empty fourteen versus template one; rare trail archaeology has twelve
  equal entries, four of which are wayfinder, raiser, shaper and host;
- an ominous vault reaches its unique table behind chance `0.75`, where flow has weight three of
  ten, for `9/40`; a normal vault reaches its unique table behind chance `0.25`, where bolt has
  weight three of twelve, for `1/16`.

The trade-rebalance overlay keeps the four affected template odds and counts unchanged: outpost and
jungle pools are identical, while ancient-city and desert-pyramid book alternatives replace equal
empty weight without changing totals `80` and `7`. Structure/container installation, archaeology,
vault state/ejection and elder-guardian death retain their named owners; `ITM-LOOT-001` owns pool
evaluation and stack emission.

No other bundled loot, recipe output, trade or mob drop initially emits a scoped identity. None is
fuel or compostable.

#### Advancement joins

`trim_with_any_armor_pattern` has one OR requirement containing all 18 exact trim recipe IDs, so one
successful trim completes it. Its child `trim_with_all_exclusive_armor_patterns` has eight ANDed
single-criterion groups: rib, silence, snout, spire, tide, vex, ward and wayfinder. The child awards
150 experience. Both send telemetry. Recipe-crafted listeners, requirement persistence, reward and
publication remain `ITM-ADVANCEMENT-001`.

**Client projection:**

All 19 item definitions directly select like-named generated models and textures, without tint or a
special item renderer. Each template appends exactly six hover lines after the ordinary item name:
gray “Smithing Template”, blank, gray “Applies to:”, a leading-space blue applicability line, gray
“Ingredients:”, and a leading-space blue ingredient line. Armor trims say “Armor” and
“Ingots & Crystals”; netherite upgrade says “Diamond Equipment” and “Netherite Ingot”.

With a scoped template in slot zero, empty smithing inputs use identity-family hints. Armor trims
cycle helmet/chestplate/leggings/boots for base and
ingot/redstone-dust/lapis-lazuli/quartz/diamond/emerald/amethyst-shard for addition; hovered empty
slots say “Add a piece of armor” or “Add ingot or crystal”. Netherite upgrade cycles
helmet/sword/chestplate/pickaxe/leggings/axe/boots/hoe/shovel/nautilus-armor/spear for base and
ingot for addition; its empty-slot text says “Add diamond armor, weapon, or tool” or
“Add Netherite Ingot”.

The Ingredients tab places the family after all 23 pottery sherds and before experience bottle in
this exact order: netherite, sentry, vex, wild, coast, dune, wayfinder, raiser, shaper, host, ward,
silence, tide, snout, rib, eye, spire, flow, bolt. All use ordinary parent/search visibility.
Trimmed humanoid armor selects the current pattern/material textures through the generic equipment
renderer; the 18 pattern records each supply same-named humanoid and leggings pattern assets.

**Branches and aborts:**

All 19 identities and rarities; ordinary/patched/overstacked source; valid/mirrored/malformed
duplication; every exact core including both bolt alternatives; valid/invalid/equal smithing result;
all initial-acquisition pools and overlay; every unlock/trim advancement requirement; template
tooltip, base/addition slot and parent/search rendering contexts.

**Constants and randomness:**

Item IDs `1458..1476`; stack maximum `64`; 19 duplication recipes, each seven diamonds plus one
template/core to two results; 18 trim recipes and pattern records; 12 netherite transforms; 49
recipe unlock advancements; two gameplay trim advancements; exclusive reward 150 XP. Loot,
archaeology, vault and structure selection consume their owning RNG streams; item behavior,
duplication and smithing selection consume none.

**Side effects:**

Craft inputs/results and unlocks; smithing preview/result, components, input decrements, stat,
criterion and level event; loot/entity-drop/vault output; advancement progress/reward/telemetry;
tooltip, screen-hint, item, tab and trimmed-equipment projection.

**Gates:**

Exact template identity and rarity; duplication shape/core; live recipe, tag, trim-pattern, loot and
advancement snapshots; smithing role and equal-trim admission; structure/table/entity/vault source;
client tooltip, slot, resource and tab context.

**State read/written:**

Reads stack identity/count/components, crafting and smithing inputs, live data snapshots, loot
context/RNG, advancement progress and client resources. Writes ordinary stacks, recipe unlocks,
trim or transformed result components, consumed inputs, loot/drop/inventory state, advancement
progress/reward and client projection.

**Failure behavior:**

An invalid or missing duplication cell does not craft; a wrong template cannot match another
pattern; an absent trim material or equal material/pattern produces no takeable result; loot
nonselection emits empty or the alternative; failed structure/container/vault/entity paths emit no
template; one exclusive trim criterion cannot substitute for another.

**Persistence boundary:**

Scoped stacks persist count and generic components. Trimmed outputs persist their trim component and
transformed outputs persist the serializer-selected component patch; recipe transactions, loot and
vault draws do not persist. Recipe unlock and advancement criteria persist independently. Data
reload replaces recipes, tags, trim-pattern records, loot and advancements without rewriting
existing template stacks; code-built template class, tooltip text keys and slot icon lists remain
fixed, while resource reload replaces models, item textures, language and trim-pattern textures.

**Boundary cases and quirks:**

Creative order differs substantially from raw item-ID order. Rarity differs by identity even though
all share one subclass: ward, eye, vex and spire are rare; silence alone is epic. The template item
contains UI description/icon behavior but no held-use behavior. Duplication is renewable only after
one template exists. Coast has three independent shipwreck table paths; netherite and snout occupy
independent pools in every bastion table; library and treasure-bastion netherite pools are
guaranteed. Normal/ominous vault headline odds include both the outer unique-pool chance and inner
item weight.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.SmithingTemplateItem`;
`net.minecraft.world.item.SmithingTemplateItem#appendHoverText`;
`net.minecraft.client.gui.screens.inventory.SmithingScreen#getTemplateItem`;
`net.minecraft.client.gui.screens.inventory.SmithingScreen#extractRenderState`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/*_smithing_template.json`;
`data/minecraft/recipe/{*_smithing_template*,netherite_*_smithing}.json`;
`data/minecraft/trim_pattern/*.json`;
`data/minecraft/loot_table/{archaeology,entities,chests}/**/*.json`;
`data/minecraft/datapacks/trade_rebalance/data/minecraft/loot_table/chests/*.json`;
`data/minecraft/advancement/{recipes/misc/*smithing_template*,recipes/{combat,tools}/netherite_*_smithing,adventure/trim_with_*}.json`;
`assets/minecraft/{items,models/item,textures/item}/*_smithing_template.*`;
`assets/minecraft/textures/trims/entity/{humanoid,humanoid_leggings}/*.png`;
`ITM-RECIPE-001`; `ITM-RECIPE-SERIALIZER-001`; `ITM-CRAFT-001`; `ITM-SMITHING-001`;
`ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `BLK-VAULT-001`; `EXP-ITM-020`.

**Test vectors:**

Query every item ID, class, rarity and default. Duplicate every identity with exact, mirrored,
patched-source, wrong-core, both bolt-core and malformed grids. Exercise all 18 trim recipes across
every material/base plus equal-trim rejection and all twelve netherite transforms. Evaluate every
loot weight endpoint, structure/table installation, archaeology result, elder-guardian drop and
normal/ominous vault outer/inner branch with overlay on/off. Complete 49 unlock records and both
trim advancements. Persist/reload stacks, trimmed results, unlock/progress state and render every
tooltip, slot-icon/text, item, tab and equipment-pattern context.

**Limits:**

Generic item stacks, crafting/smithing allocation and commit, loot evaluation, advancement
machinery, entity death, archaeology, vault runtime, structure generation, equipment trim assembly
and rendering stay with cited owners. This leaf fixes the 19 template identities, exact data joins,
acquisition/absence boundaries and observable cross-system consequences.
