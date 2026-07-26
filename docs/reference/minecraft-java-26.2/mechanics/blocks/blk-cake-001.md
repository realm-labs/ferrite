# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-CAKE-001` — Cake converts hunger into bites, comparator steps and candle-cake state

**Parent:** `SIM-004`, `SIM-005`, `BLK-001`, `BLK-STATE-001`,
`BLK-002`, `BLK-PLACE-001`, `BLK-BREAK-001`, `BLK-BREAK-HOOK-001`,
`BLK-BREAK-CONTENT-001`, `BLK-UPDATE-001`, `PLY-002`, `PLY-005`,
`PLY-006`, `PLY-INTERACT-001`, `PLY-BREAK-001`, `PLY-COLLISION-001`,
`PLY-AUTOJUMP-001`, `RED-001`, `RED-UPDATE-001`,
`RED-COMPARATOR-001`, `ITM-003`, `ITM-004`, `ITM-006`, `ITM-007`,
`ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-HUNGER-001`, `ENT-001`, `MOB-AI-001`,
`ENV-001`, `ENV-002`, `ENV-003`, `ENV-FIRE-001`, `ENV-LIGHT-001`,
`WGEN-003`, `CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked registration, complete `CakeBlock` override and
exact-identity consumer sweeps, reports, recipe, advancements, loot, trade,
tags and client assets fix all seven states and every Cake-specific branch.
Exhaustive decoded-NBT and constant-pool scans of all 1,212 templates prove the
generation absence. The 17 Candle Cake identities retain their existing
catalog family; this leaf fixes Cake's conversion to and from that boundary.

**Applies when:**

`minecraft:cake` is placed, supported, updated, eaten, read by a comparator,
combined with a Candle, cloned, broken, composted, crafted, traded, selected in
Trial loot, picked up/eaten by a Panda, delivered by an Allay, persisted,
synchronized or rendered.

**Authoritative state:**

Cake is a `CakeBlock` with integer property `bites=0..6`, no block entity and
seven states. Default bites-zero state is ID `7027`; states `7028..7033`
increase monotonically through bites one to six. Protocol block ID is `298`.
Its ordinary `BlockItem` has raw item ID `1114`, common rarity, maximum stack
size `1`, no food/consumable component and no special components. Cake must be
placed before a Player can eat it.

Registration uses otherwise-default properties plus `forceSolidOn`, strength
`0.5`, Wool sounds and piston reaction `DESTROY`. Thus destroy time and
explosion resistance are both `0.5`; map color is `NONE`, note instrument is
Harp, friction is `0.6`, speed/jump factors are `1`, restitution and light
emission are `0`, and lava does not ignite it.

For bites `b`, selection and collision are the box
`[(1+2b)/16,0,1/16]..[15/16,8/16,15/16]`. The west edge therefore retreats by
`2/16` per bite while height, east edge and Z bounds stay fixed. Occlusion,
support and visual shapes follow that nonfull geometry; interaction shape is
empty. The shape makes solid rendering, redstone conduction, view blocking and
suffocation false, propagates skylight with dampening `0`, and emits no light.
`forceSolidOn` nevertheless makes `BlockState#isSolid()` true at every bite.
`isPathfindable` explicitly returns false for every computation type.

Wool sound-event protocol IDs are Break `1858`, Fall `1859`, Hit `1860`, Place
`1861` and Step `1862`. Cake has no block entity, signal, random tick, entity
contact, attack, projectile, fall, block event or client-animation override.
It has no direct block tags.

**Transition and ordering:**

### Placement and support

Cake uses default bites-zero placement with no orientation. It survives exactly
when the block immediately below reports `isSolid()`; this does not ask for a
sturdy upper face. A Cake can consequently support another Cake because
`forceSolidOn` makes its own state solid despite the inset half-height shape.

Only a downward neighbor update performs Cake's special support check. If the
current state no longer survives, `updateShape` returns default Air immediately;
otherwise it returns the base update result. Removal by this returned-state
path has no Cake loot.

### Empty-hand eating

Server eating first calls `Player#canEat(false)`. A Player whose hunger gate
rejects the bite receives `PASS` and no mutation. An accepted bite then orders
effects as follows:

1. award custom statistic `minecraft:eat_cake_slice` (protocol ID `39`);
2. call `FoodData#eat(2,0.1)`, adding two food points and `0.4` saturation
   before normal caps;
3. emit `EAT` game event at the Cake with the Player;
4. if old bites are below six, attempt `setBlock` of bites plus one with flags
   `3` and ignore its Boolean result; otherwise attempt
   `removeBlock(position,false)`, ignore that result and emit
   `BLOCK_DESTROY`; and
5. return `SUCCESS`.

Nutrition, statistic and game event therefore commit even when the bite-state
write fails. On the seventh accepted bite, the destroy event is emitted even
when removal fails. Cake plays no dedicated eating sound and creates no
block-owned eating particles.

Client `useWithoutItem` first runs the same `eat` helper as prediction. A
consuming result is projected as `SUCCESS`. If prediction does not consume and
the main hand is empty, it returns `CONSUME`; with a nonempty main hand it falls
through to the helper again and returns that result. The server runs the helper
once and remains authoritative for food, statistic and state.

### Comparator output

Every Cake state declares analog output. The value is
`(7-bites)*2`: bites zero through six produce `14,12,10,8,6,4,2`.
The queried direction is ignored. The flags-`3` bite write and final removal
feed generic comparator-neighbor propagation; no Cake state outputs `15`.

### Candle conversion boundary

`useItemOn` converts only bites-zero Cake. The held stack must belong to the
live `minecraft:candles` item tag, its item-to-block mapping must resolve to a
`CandleBlock`, and that Candle must map to one of the 17 Candle Cake blocks.
Every other item/state returns `TRY_WITH_EMPTY_HAND`.

An admitted conversion consumes one Candle through the generic
Player-aware stack owner, plays `block.cake.add_candle` (sound-event ID `243`)
at volume/pitch `1`, attempts `setBlockAndUpdate` to the matching default
unlit Candle Cake and ignores the result, emits `BLOCK_CHANGE`, awards that
Candle's `ITEM_USED` statistic and returns `SUCCESS`. Consumption and every
listed side effect therefore remain even when the conversion write fails.

The 17 resulting blocks are the uncolored and 16 dyed Candle Cakes, each with
Boolean `lit` and no item registry entry. They preserve the inherited Cake
strength/sound/piston profile, survival predicate and full-cake analog output
`14`; lit state emits light level `3`. Their shape unions the
`[1,0,1]..[15,8,15]` Cake body with a centered width-two candle column from
Y `8..14`.

Successful empty-hand eating of a Candle Cake invokes this leaf's helper with
default bites-zero Cake. It first replaces the position with bites-one Cake,
then `CandleCakeBlock` drops the original block's loot: exactly its
corresponding Candle. If the hunger gate returns `PASS`, neither conversion nor
drop occurs. Every Candle Cake clone/pick result is instead one Cake item.
Breaking it normally drops only its Candle. Lighting/extinguishing and candle
particles remain with the Candle owners.

**Acquisition, consumption and progression:**

The Cake block loot table is empty and only fixes random sequence
`minecraft:blocks/cake`; hand, tools, Silk Touch, Fortune and explosions never
drop Cake. Generic pick-block returns one Cake item at every bite. There is no
FireBlock flammability row, lava ignition or fuel-values entry.

Composter bootstrap assigns Cake chance `1.0`. Every admitted nonfull
Composter insertion therefore advances one level; player/automation admission,
ready-state timing, extraction and stack disposition remain with the
Composter owner.

The sole recipe is a shaped `3x3` grid:

```text
Milk Bucket  Milk Bucket  Milk Bucket
Sugar        #eggs        Sugar
Wheat        Wheat        Wheat
```

The locked egg tag contains Egg, Blue Egg and Brown Egg. A valid grid produces
one default Cake stack and consumes all nine inputs; the three Milk Buckets
return three empty Buckets through their `use_remainder` components. The recipe
advancement is an OR between possessing any `#minecraft:eggs` member and
already having the recipe, and awards `minecraft:cake`. No other recipe
produces or consumes Cake.

Farmer level four selects both tagged entries, including an offer of three
Emeralds for one Cake with `12` maximum uses, `15` villager XP and price
multiplier `0.05`.

Trial Chambers intersection chest has one pool with uniform `1..3` rolls over
total weight `86`. Cake has weight `20`, hence probability `10/43` per roll,
then receives uniform count `1..4`. Its maximum stack size is one; generic loot
filling owns later splitting, shuffling and capacity loss. The random sequence
is `minecraft:chests/trial_chambers/intersection`.

Cake appears once in Food and Drinks, directly after Cookie and before Pumpkin
Pie.

### Panda ground-food consumer

Cake item is directly in `panda_eats_from_ground`. A Panda with empty main hand
accepts a matching alive Item Entity without pickup delay, equips its entire
stack, marks the slot guaranteed-drop, records the pickup and discards the
entity. Cake's maximum stack size makes that stack one.

While sitting, not scared and holding an item, a non-eating Panda starts its
eating state when `nextInt(80)==1`. Every fifth eating counter produces the
generic Panda Eat sound (ID `1198`) and six held-item particles. Once the
counter is above `80`, each server tick draws `nextInt(20)` and terminates on
value `1`. A termination at counter at most `100` consumes nothing. Above
`100`, a still-tagged held Cake is cleared, emits `EAT`, unsits the Panda and
then stops eating; a tag change before this check makes it stop without
consuming.

### Allay advancement

Hidden advancement `husbandry/allay_deliver_cake_to_note_block`, parented by
`allay_deliver_item_to_player`, has one `allay_drop_item_on_block` criterion.
Its location condition requires exact Note Block and its match-tool predicate
requires exact Cake. Completion has no item/experience reward, displays the
Note Block icon and sends its telemetry event.

**World and client projection:**

No configured/placed feature, biome, structure, processor or pool resource
references Cake. Exhaustive scans of all 1,212 decoded templates find zero raw
Cake block cells and zero UTF occurrences, including stored item identities.
Trial acquisition is therefore loot-table-driven rather than stored in a
template.

The blockstate selects `block/cake` for bites zero and
`block/cake_slice1..6` for the corresponding later states. Each model matches
the runtime box: X begins at `1+2*bites`, ends at `15`, Y is `0..8`, and Z is
`1..15`. The exposed west face of a bitten model uses `cake_inner`; other
horizontal faces use `cake_side`, with distinct top/bottom textures. Only the
down face has a cullface. Particle texture is the side.

The item definition selects a separate generated flat `item/cake` model with
the item Cake texture; it does not render the block geometry. There is no tint,
conditional model, special renderer or texture metadata. The English name is
`Cake`.

**Branches and aborts:**

Support direction/solidity; hunger admission; client prediction and main-hand
emptiness; old bite/final removal and write result; comparator state; held
Candle tag/class/color and conversion write; Candle Cake hunger/light/hit/
loot boundary; block break versus clone; compost admission; shaped recipe/egg
identity/remainders/unlock; trade and chest selection/count/splitting; Panda
pickup/sitting/fear/start/termination/tag state; Allay trigger predicates;
persistence/resource context and bite model are distinct.

**Constants and randomness:**

States `7027..7033`, block/item IDs `298/1114`, bites `0..6`; X minimum
`1+2b`, bounds Y `0..8`, Z `1..15`; strength `0.5/0.5`; signals
`14,12,10,8,6,4,2`; nutrition `2`, modifier `0.1`, saturation gain `0.4`;
writes `3`; add-Candle sound `243`; custom stat `39`; Candle Cake light `3`;
compost `1`; recipe inputs `3/2/1/3` and three Bucket remainders; trade
`3:1`, uses/XP/discount `12/15/0.05`; chest rolls `1..3`, chance `10/43`,
count `1..4`; Panda start `nextInt(80)==1`, termination after `80` on
`nextInt(20)==1`, consumption only above `100`, particles every `5`.

**Side effects:**

Placement/item consumption; hunger/saturation/statistics/game events; bite,
Air and Candle Cake writes; comparator notifications; Candle sound/stat and
later Candle drop; block/loot/pick outcomes; Composter level; crafting
knowledge, inputs and remainders; offers/chest stacks; Panda equipment, state,
sound/particles and event; Allay criterion/telemetry; persistence, packets,
sounds and rendering.

**Gates:**

Loaded state and interaction/write/break authority; live below state, hunger,
hand and tags; Candle mapping; comparator read; loot/recipe/advancement/trade
snapshots and inventory capacity; Panda item/entity/equipment/fear/sitting/RNG/
counter/tag state; Allay dropped item and exact block; registry, resource,
sound and render context.

**Boundary cases and quirks:**

- Cake is a non-edible item whose placed block provides seven separate bites.
- Full Cake outputs `14`, not `15`; the final visible bite still outputs `2`.
- Accepted eating commits nutrition, stat and `EAT` before an ignored write,
  and the final branch emits `BLOCK_DESTROY` after an ignored removal.
- `forceSolidOn` lets half-height inset Cake satisfy `isSolid`, including as
  support for another Cake, without making its shape a full sturdy cube.
- Candle conversion only accepts bites zero and commits consumption, sound,
  event and statistic even if its state write fails.
- Eating a Candle Cake produces bites-one Cake plus its Candle; ordinary
  breaking produces only the Candle, while clone produces Cake.
- Panda termination can occur between counters `81..100` without consuming.
- The zero-template census includes UTF item occurrences; Trial Cake comes only
  from the chest loot table.

**Failure semantics:**

Unsupported downward updates return Air without Cake loot. A rejected hunger
gate changes nothing. Failed bite, removal or Candle writes do not roll back
earlier effects. Invalid/output-blocked crafting consumes nothing. Rejected
loot/trade/chest/Panda/Allay gates retain only generic earlier owner effects.

**Client/server authority split:**

The server owns support, hunger, state, comparator truth, candle conversion,
loot, composting, crafting/knowledge, trades, chests, Panda behavior,
advancement and persistence. The client predicts interaction and projects
state/item IDs, geometry, textures, name, tab order and sounds/particles.

**Observability:**

Commands/state packets, shape/light/support/path probes, hunger/saturation,
statistics/game-event listeners, comparators, inventory/candle/drop results,
Composters, crafting/recipe book, offers/chests, Panda equipment/animation,
advancement UI/telemetry, template scans, maps, sounds, tabs and rendering
expose every branch.

**Persistence and reload:**

Placed Cake persists identity and bites; stacks persist ordinary components.
Recipes, advancements, loot, item/block/trade tags and trade records are
reload-selected. Registration, state/eating/comparator control flow, Composter
row, Panda control flow and creative order are code-built. Reload does not
retroactively alter bites or convert Cake.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-REPORT-001`, `OFF-DATA-001`;
`net.minecraft.world.level.block.Blocks`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.level.block.CakeBlock`;
`net.minecraft.world.level.block.CakeBlock#useItemOn`;
`net.minecraft.world.level.block.CakeBlock#useWithoutItem`;
`net.minecraft.world.level.block.CakeBlock#eat`;
`net.minecraft.world.level.block.CakeBlock#getOutputSignal`;
`net.minecraft.world.level.block.CakeBlock#updateShape`;
`net.minecraft.world.level.block.CandleCakeBlock`;
`net.minecraft.world.level.block.CandleCakeBlock#useWithoutItem`;
`net.minecraft.world.level.block.CandleCakeBlock#getCloneItemStack`;
`net.minecraft.world.level.block.ComposterBlock#bootStrap`;
`net.minecraft.world.entity.animal.panda.Panda#pickUpItem`;
`net.minecraft.world.entity.animal.panda.Panda#handleEating`;
`net.minecraft.world.item.trading.VillagerTrades`;
`net.minecraft.world.item.CreativeModeTabs#bootstrap`;
block/item/sound/custom-stat reports and components; Cake/Candle-Cake loot,
Cake recipe/advancement, Farmer trade/tag, Panda/Candle tags, Trial intersection
chest and Allay advancement; all 1,212 templates; exact blockstate, seven block
models, item definition/model/texture and language resource. Complete compiled
exact-field and data-reference searches found no other runtime path.

**Test vectors:**

Run `EXP-BLK-109` across all bites, support/hunger/client/write outcomes,
comparators and every Candle conversion/eat/break/clone boundary; loot,
Composter, recipe/unlock/remainders, Farmer/Trial, Panda and Allay paths; all
templates, persistence/reload and exact projection. Assert exact ordering,
constants, absences and vanilla convergence.

**Limits:**

Generic placement, state writes, hunger, comparator propagation,
breaking/loot/explosion, crafting, composting, trading/chest filling, mob item
pickup, advancement triggers, packet encoding and rendering remain with their
named owners. Candle and Candle Cake lighting/rendering retain their existing
families. This leaf fixes Cake identity, all hooks, outgoing conversion, exact
acquisition/consumer joins, locked absences and projection.
