# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-STEW-001` — Bowl foods separate player remainder, suspicious effects, and mob transactions

**Parent:** `PLY-005`, `PLY-006`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`,
`ENT-001`, `ENT-006`, `MOB-001`, `MOB-004`, `MOB-005`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked default components, the generic use-remainder and suspicious-effect
components, mooshroom and wolf call sites, recipes, advancements, loot, trades, tags, registries
and client assets close all five scoped items.

**Applies when:**

A `minecraft:bowl`, `minecraft:mushroom_stew`, `minecraft:rabbit_stew`,
`minecraft:beetroot_soup` or `minecraft:suspicious_stew` stack is crafted, acquired, offered to a
mob, consumed, persisted, reloaded or projected to an unmodified client.

**Authoritative state:**

| Item | Protocol ID | Stack/components |
|---|---:|---|
| bowl | `920` | common, maximum 64, no use component |
| mushroom stew | `975` | maximum 1; food `6/7.2000003`; ordinary consumable; bowl remainder |
| rabbit stew | `1281` | maximum 1; food `10/12.0`; ordinary consumable; bowl remainder |
| beetroot soup | `1319` | maximum 1; food `6/7.2000003`; ordinary consumable; bowl remainder |
| suspicious stew | `1371` | maximum 1; always-edible food `6/7.2000003`; ordinary consumable; ordered suspicious-effect list; bowl remainder |

Every scoped item has common rarity and an otherwise ordinary item component map. The four foods
use the default 1.6-second eat animation and cadence. Bowl is a direct 100-tick furnace fuel; the
four filled foods have burn time zero. None is compostable.

**Transition and ordering:**

#### Player consumption and bowl remainder

Ordinary consumption admission, food/exhaustion changes, particles, sound, used-item stat,
criterion, component-listener iteration, game event and stack shrink remain in `ITM-USE-001` and
`ITM-HUNGER-001`. Mushroom stew, rabbit stew and beetroot soup require the normal player hunger
gate; suspicious stew's `can_always_eat` admits a full player.

All four foods carry `use_remainder={id:minecraft:bowl}`. Finish processing reads that component
from the pre-use copy after item behavior. A finite player consuming the usual count-one stack
empties it and receives the bowl as the returned hand stack. If a legal or externally constructed
overstack retains food, one bowl is instead inserted into inventory or dropped while the shortened
food stack remains in hand. Infinite-material use neither shrinks the food nor creates a bowl.
Direct mob-feeding and other callers that only consume a stack do not run this after-use component.

Suspicious stew's default effect list is empty. During consumable listener iteration, every stored
entry is converted to a level-zero `MobEffectInstance` and offered to the living consumer in list
order; individual `addEffect` results are ignored. This listener runs before the consumable's
server-side consume-effect list, which is empty for all four foods. Thus a default empty-effect
stew still feeds, counts as consumed and returns a bowl.

An effect entry stores a holder plus duration ticks; its data codec defaults omitted duration to
`160`, and its stream duration is a VarInt. The component is visible in the tooltip only under the
creative-tooltip flag, where the client renders the stored effect list and durations through the
ordinary potion tooltip formatter.

#### Crafting and flower effects

The locked recipes are:

| Output | Input and result |
|---|---|
| bowl | shaped `# #` / ` # ` with any `planks`, producing four |
| mushroom stew | shapeless brown mushroom + red mushroom + bowl, producing one |
| rabbit stew | shapeless baked potato + cooked rabbit + bowl + carrot + either brown or red mushroom, producing one in each of two recipes |
| beetroot soup | shapeless bowl + six beetroot, producing one |
| suspicious stew | shapeless bowl + brown mushroom + red mushroom + one of the 17 effect flowers, producing one stack with that flower's single effect entry |

Each of the 22 recipe records has its matching recipe advancement. Generic matching, consumption
of crafting slots, unlock handling and output placement stay with their parent rules.

The 17 flower outputs, in item-registry/creative order, are:

| Flower item ID | Effect | Stored duration |
|---:|---|---:|
| dandelion `256` | Saturation | `7` |
| golden dandelion `257` | Saturation | `7` |
| open eyeblossom `258` | Blindness | `220` |
| closed eyeblossom `259` | Nausea | `140` |
| poppy `260` | Night Vision | `100` |
| blue orchid `261` | Saturation | `7` |
| allium `262` | Fire Resistance | `60` |
| azure bluet `263` | Blindness | `220` |
| red tulip `264` | Weakness | `140` |
| orange tulip `265` | Weakness | `140` |
| white tulip `266` | Weakness | `140` |
| pink tulip `267` | Weakness | `140` |
| oxeye daisy `268` | Regeneration | `140` |
| cornflower `269` | Jump Boost | `100` |
| lily of the valley `270` | Poison | `220` |
| wither rose `271` | Wither | `140` |
| torchflower `272` | Night Vision | `100` |

`FlowerBlock` derives these entries by flooring configured seconds times 20; recipes materialize
the resulting tick count directly. `SuspiciousEffectHolder.tryGet` resolves a block item to its
block before testing the holder interface, so flower stack component mutations do not change the
selected effect.

#### Brown-mooshroom charge and bowl interaction

An adult mooshroom checks bowl before shears, flower charging and inherited cow interaction. A
normal finite bowl stack shrinks by one: if that empties the hand, the output replaces it;
otherwise the output is inserted or dropped. An infinite-material player retains the bowl and
receives the output by inventory insertion or drop.

Red and uncharged brown adults produce ordinary mushroom stew and play Mooshroom Milk sound
`987`, volume/pitch `1/1`. A brown adult with non-null stored effects instead creates suspicious
stew, copies the complete ordered component to it, clears the entity field before inventory
delivery, and plays Suspicious Milk sound `988` at `1/1`. A baby does not enter this branch.

Only a brown adult accepts an item whose underlying block implements `SuspiciousEffectHolder`.
When its stored field is null, the caller consumes one flower, emits four Effect particles, stores
that flower's effect list, then plays Mooshroom Eat sound `986` at volume `2`, pitch `1`. Each
particle consumes three entity-RNG doubles: positive X and Z offsets divided by two and positive Y
velocity divided by five.

When the field is already non-null, a valid flower instead emits two Smoke particles with the same
three-double geometry, consumes nothing, preserves the old list, plays no sound and still returns
success. Invalid items, babies and red variants fall through to inherited interaction. The
nullable list persists under `stew_effects`; it is server-private until particles or a milked
result stack expose it.

#### Wolf use, loot, trades and progression

Rabbit stew is directly in `wolf_food`. Using it on a tamed, injured wolf enters that branch before
the wolf's remaining owned interactions, consumes one through `usePlayerItem`, heals
`2 * nutrition = 20` health and plays the wolf eating sound. This is not item-use completion, so
the bowl remainder is not created. A full-health wolf does not take this food branch.

Configured noncrafting acquisition is exact:

- a lightning-tagged turtle death emits one bowl; fishing junk gives bowl weight 10 within a
  one-roll table whose conditional bamboo entry changes the admitted denominator by biome;
- snowy-village house loot gives beetroot soup weight 1 over `3..8` first-pool rolls;
- butcher level one chooses two distinct trades from four, one of which sells one rabbit stew for
  one emerald with 12 uses and discount `0.05`;
- farmer level four selects both entries, including one emerald for one suspicious stew with 12
  uses, 15 villager XP and discount `0.05`;
- desert-well archaeology gives suspicious stew weight 1 of 8 in one roll; shipwreck supply gives
  it weight 10 of 84 over `3..10` first-pool rolls; ancient-city ice-box gives it weight 1 of 9
  over `4..10` rolls and then sets count `2..6` despite the item's maximum stack size one.

The three suspicious-stew acquisition paths with effect modifiers uniformly select one listed
entry. Shipwreck and desert-well use Night Vision `7..10` seconds, Jump Boost `7..10`, Weakness
`6..8`, Blindness `5..7`, Poison `10..20` or Saturation `7..10`. Ancient-city ice-box selects
Night Vision `7..10` or Blindness `5..7`. The farmer trade selects fixed Night Vision `5`, Jump
Boost `8`, Weakness `7`, Blindness `6`, Poison `14` or Saturation `7`.

`SetStewEffectFunction` returns a non-stew stack or an empty configured list unchanged. Otherwise
it uniformly chooses one entry, samples its integer duration, multiplies noninstantaneous durations
by 20, leaves instantaneous duration unscaled, and appends the resulting entry to the existing
component. Loot/trade selection, provider arithmetic, pool rolls and inventory insertion retain
their generic owners.

All four filled foods are independent criteria in the 40-entry `husbandry/balanced_diet`
advancement. Generic hunger, criterion completion and the 100-experience reward stay with
`ITM-HUNGER-001` and `ITM-ADVANCEMENT-001`.

**Client projection:**

All five items use untinted generated flat models. Ingredients orders bowl after the dye
collection and before brick. Food & Drinks orders mushroom stew, beetroot soup and rabbit stew
after spider eye, then generates exactly the 17 component-bearing suspicious-stew variants in the
table order before milk bucket. Those variants are exposed to both the parent and search tabs;
the default empty-effect suspicious stew is not separately inserted.

**Branches and aborts:**

Hunger/full/always-edible; finite/infinite material; count one/overstack and remainder
insertion/drop; suspicious list empty/one/many and individual effect admission; recipe/flower;
mooshroom variant/age/stored state/held item/inventory; wolf tame/health/tag; loot table, effect
list/provider and trade selection; tooltip flag and tab context.

**Constants and randomness:**

Consumption `1.6` seconds; food `6/7.2000003`, `10/12.0`, `6/7.2000003`,
`6/7.2000003`; bowl fuel `100`; effect codec default `160`; mooshroom charge/reject particle
counts `4/2`, three doubles each; sounds `986..988`; recipe count `22`; flower variants `17`;
balanced-diet entries `40`; exact loot weights, rolls, counts and effect ranges above.

**Side effects:**

Stacks, food/exhaustion/effects, stats/criteria, inventory or dropped remainders/results,
mooshroom stored effects, wolf health, recipe/trade/progression state, particles, sounds and client
item/tooltip/tab projection.

**Gates:**

Item identity/default components; hunger and abilities; active-use revalidation; effect holders
and component list; recipe/advancement/loot/trade/tag snapshots; mooshroom variant/age/state;
wolf ownership/health/tag; inventory admission and client tooltip/tab selectors.

**State read/written:**

Reads stack identity/count/components, player hunger/abilities/inventory, living effects,
mooshroom variant/age/effect payload/RNG, wolf tame/health, active data snapshots and client
context. Writes stack/inventory/drop state, food/effects, mooshroom payload, wolf health,
progression and visible effects.

**Failure behavior:**

Denied consumption leaves state unchanged; failed extra-remainder insertion drops the bowl;
individual effect offers ignore their result; a charged mooshroom rejects replacement after
particles without consuming the flower; output insertion failure drops the stew after the
mooshroom payload has already cleared; inapplicable loot-function inputs remain unchanged.

**Persistence boundary:**

Item counts and components persist with their stacks. A mooshroom stores nullable
`stew_effects` through the same component codec; absence reloads as null. Active-use progress,
particle RNG and loot/trade/crafting draws do not persist or catch up. Reload replaces recipes,
advancements, loot, trades and tags without changing existing stacks, entity payloads, food or
effects; code-built default components, remainder behavior and client assets do not reload with
server data.

**Boundary cases and quirks:**

An empty-effect suspicious stew is still always edible. Loot can create overstacked count-2..6
suspicious stew even though its maximum is one; player consumption then creates an extra bowl
rather than replacing the remaining food. Rabbit stew fed to a wolf produces no bowl. A charged
brown mooshroom consumes neither a second flower nor its new effects, and its stored component is
cleared before a milk result is inserted or dropped. Creative tabs show 17 effect variants but no
separate default stew.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.ItemStack#finishUsingItem(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity)`;
`net.minecraft.world.item.component.Consumable#onConsume(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack)`;
`net.minecraft.world.item.component.UseRemainder#convertIntoRemainder(net.minecraft.world.item.ItemStack,int,boolean,net.minecraft.world.item.component.UseRemainder$OnExtraCreatedRemainder)`;
`net.minecraft.world.item.component.SuspiciousStewEffects#onConsume(net.minecraft.world.level.Level,net.minecraft.world.entity.LivingEntity,net.minecraft.world.item.ItemStack,net.minecraft.world.item.component.Consumable)`;
`net.minecraft.world.item.component.SuspiciousStewEffects$Entry#createEffectInstance()`;
`net.minecraft.world.level.block.SuspiciousEffectHolder#tryGet(net.minecraft.world.level.ItemLike)`;
`net.minecraft.world.level.block.SuspiciousEffectHolder#getAllEffectHolders()`;
`net.minecraft.world.level.block.FlowerBlock#makeEffectList(net.minecraft.core.Holder,float)`;
`net.minecraft.world.entity.animal.cow.MushroomCow#mobInteract(net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`;
`net.minecraft.world.entity.animal.cow.MushroomCow#addAdditionalSaveData(net.minecraft.world.level.storage.ValueOutput)`;
`net.minecraft.world.entity.animal.cow.MushroomCow#readAdditionalSaveData(net.minecraft.world.level.storage.ValueInput)`;
`net.minecraft.world.entity.TamableAnimal#feed(net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand,net.minecraft.world.item.ItemStack,float,float)`;
`net.minecraft.world.entity.animal.wolf.Wolf#mobInteract(net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)`;
`net.minecraft.world.level.storage.loot.functions.SetStewEffectFunction#run(net.minecraft.world.item.ItemStack,net.minecraft.world.level.storage.loot.LootContext)`;
`net.minecraft.world.item.ItemUtils#createFilledResult(net.minecraft.world.item.ItemStack,net.minecraft.world.entity.player.Player,net.minecraft.world.item.ItemStack,boolean)`;
`net.minecraft.world.level.block.entity.FuelValues#vanillaBurnTimes(net.minecraft.core.HolderLookup$Provider,net.minecraft.world.flag.FeatureFlagSet,int)`;
`net.minecraft.world.item.CreativeModeTabs#generateSuspiciousStews(net.minecraft.world.item.CreativeModeTab$Output,net.minecraft.world.item.CreativeModeTab$TabVisibility)`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap(net.minecraft.core.Registry)`;
`reports/registries.json#minecraft:{item,sound_event}`;
`reports/minecraft/components/item/{bowl,mushroom_stew,rabbit_stew,beetroot_soup,suspicious_stew}.json`;
`data/minecraft/recipe/{bowl,mushroom_stew,rabbit_stew_*,beetroot_soup,suspicious_stew_*}.json`;
`data/minecraft/advancement/{recipes/{misc/bowl,food/{mushroom_stew,rabbit_stew_*,beetroot_soup,suspicious_stew_*}},husbandry/balanced_diet}.json`;
`data/minecraft/loot_table/{archaeology/desert_well,chests/{ancient_city_ice_box,shipwreck_supply,village/village_snowy_house},entities/turtle,gameplay/fishing/junk}.json`;
`data/minecraft/{villager_trade/{butcher/1/emerald_rabbit_stew,farmer/4/emerald_suspicious_stew},trade_set/{butcher/level_1,farmer/level_4},tags/{villager_trade/{butcher/level_1,farmer/level_4},item/wolf_food}}.json`;
`assets/minecraft/{items,models/item}/{bowl,mushroom_stew,rabbit_stew,beetroot_soup,suspicious_stew}.json`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-HUNGER-001`; `ITM-CRAFT-001`;
`ITM-ADVANCEMENT-001`; `ITM-LOOT-001`; `MOB-AI-001`; `CLI-001`; `CLI-006`;
`EXP-ITM-016`.

**Test vectors:**

Consume every food hungry/full at count one/overstack with finite/infinite abilities, empty and
multi-effect components, accepted/rejected effects and full inventory. Craft all 22 recipes.
Charge/milk red/brown and baby/adult mooshrooms with every flower, existing/null payload and
inventory boundary across save/reload. Feed rabbit stew to tamed injured/full wolves. Exercise
every loot/trade effect list, provider endpoint, acquisition pool, tag/reload boundary, creative
tooltip, five models and both tab placements.

**Limits:**

Generic active-use, hunger, effect conflict, crafting, loot, trade, advancement, entity
interaction, persistence, protocol and rendering engines remain with their cited owners. This leaf
owns the five item identities' components, exact selectors, values, concrete mooshroom/wolf joins,
data records and client variants.
