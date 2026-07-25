# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-FERMENTED-SPIDER-EYE-001` — Fermented Spider Eyes craft once, corrupt twelve potion holders and sell to wandering traders

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-BREW-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`,
`ITM-POTION-001`, `ENT-EFFECT-001`, `MOB-WANDERING-TRADER-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, exhaustive code/data references, the sole
recipe and unlock advancement, all twelve feature-filtered potion mixes, potion payloads, the
wandering-trader buying record/set and client assets determine every Eye-specific branch. Generic
crafting, brewing, potion/effect, trade, stack and inventory behavior remains with the cited
owners.

**Applies when:**

A `fermented_spider_eye` stack is created, crafted, moved, renamed, persisted, synchronized,
offered to a Brewing Stand or wandering trader, selected in a tab, rendered or observed before and
after recipe, advancement, mix, trade or resource reload.

**Authoritative state:**

`minecraft:fermented_spider_eye` is raw item ID `1152`. It registers through the plain-item path
with default properties, is common, nondamageable and has max stack `64`.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state. No locked item tag directly contains it.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
starts active use or supplies player food.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, equipment, repair, composting or furnace branch. Its only locked
creation path is crafting; its two locked sinks are brewing and a possible wandering-trader
purchase.

**Crafting and recipe unlock:**

The sole bundled recipe is shapeless. It requires exactly one Spider Eye, one Brown Mushroom and
one Sugar in any three distinct crafting inputs and returns one default Fermented Spider Eye.
Additional occupied inputs prevent a match. The result copies no input component patch and none of
the three ingredients has a crafting remainder.

Manual result take awards and consumes through `ITM-CRAFT-001`; automated recipe users follow
their own owner. No locked recipe consumes Fermented Spider Eye, and no chest, entity, fishing,
gift, barter or other loot table emits it. Administration and custom data can still construct
ordinary stacks through generic boundaries.

The `recipes/brewing/fermented_spider_eye` advancement has two criteria in one OR requirement:
possess a Spider Eye, or unlock this exact recipe. Either awards only the recipe. Brown Mushroom,
Sugar or Fermented-Spider-Eye possession alone does not satisfy it. The advancement has parent
`recipes/root`, no experience or loot reward and no display.

**Brewing graph:**

The vanilla builder registers Fermented Spider Eye as an ordinary potion-mix ingredient on exactly
twelve directed holder edges, in this construction order:

1. Night Vision -> Invisibility;
2. Long Night Vision -> Long Invisibility;
3. Leaping -> Slowness;
4. Long Leaping -> Long Slowness;
5. Swiftness -> Slowness;
6. Long Swiftness -> Long Slowness;
7. Healing -> Harming;
8. Strong Healing -> Strong Harming;
9. Poison -> Harming;
10. Long Poison -> Harming;
11. Strong Poison -> Strong Harming;
12. Water -> Weakness.

Thus Long Poison deliberately loses its extended form, while strength survives only from Strong
Healing or Strong Poison. There is no Fermented-Spider-Eye edge from Awkward, Mundane, Thick,
Strong Leaping, Strong Swiftness, Invisibility, Harming or Weakness. Redstone separately extends
Weakness to Long Weakness; the Eye itself never emits Long Weakness.

The output payloads are amplifier-zero Invisibility for `3600` ticks and Long Invisibility for
`9600`; amplifier-zero Slowness for `1800` and Long Slowness for `4800`; one-tick Instant Damage
at amplifier zero or Strong Harming amplifier one; and amplifier-zero Weakness for `1800`.
Long Weakness, reached later with Redstone, lasts `4800`.

Potion, Splash Potion and Lingering Potion are all registered potion containers, so every listed
edge operates on all three while retaining the input container item. Mix lookup requires a present
base potion holder equal to the source. A holderless or custom-effect-only contents object does not
match and cannot start/continue a brew solely because the ingredient is an Eye.

At commit the mix installs fresh contents for the target holder. It does not preserve custom
color, custom name, custom effects or duration scale from the input contents. The ingredient
predicate tests Eye item identity, so arbitrary Eye component patches remain accepted but are
discarded when one ingredient is consumed.

A completed stand transforms matching bottle slots `0..2` in order, consumes exactly one Eye for
up to three outputs, leaves unmatched bottles unchanged and emits event `1035`. Eye has no
crafting remainder, is not in `brewing_fuel`, is not furnace fuel and cannot prepay stand fuel
uses. Slot admission, fuel, the `400`-tick transaction, cancellation, automation and player take
criterion remain with `ITM-BREW-001` and `ITM-ADVANCEMENT-001`; potion use/projection and effect
application remain with `ITM-POTION-001` and `ENT-EFFECT-001`.

**Wandering-trader sink:**

`wandering_trader/fermented_spider_eye_emerald` wants one Fermented Spider Eye and gives three
default Emeralds. It has maximum uses `2`, reputation discount coefficient `0.05`, inherited
offer XP `1`, no second cost, merchant predicate, output modifier or double-price enchantment.
The absent cost-component predicate is empty, so an Eye with arbitrary ordinary component patches
satisfies the offer; those patches are consumed and never copied to Emeralds.

The buying trade tag contains exactly six records: Water Bottle, Water Bucket, Milk Bucket,
Fermented Spider Eye, Baked Potato and Hay Bale purchases. Its trade set requests two distinct
records (`allow_duplicates=false`) with random sequence
`minecraft:trade_set/wandering_trader/buying`. Therefore a baseline generated trader includes the
Eye purchase with probability `2/6=1/3`.

Wandering Trader appends this buying set before its uncommon and common sets. Selection creates an
offer but consumes no Eye; only a successful generic merchant transaction converts one player Eye
to three Emeralds until the offer exhausts. This path is an Eye sink, not an acquisition source.
Trader spawning/despawn remains with `MOB-WANDERING-TRADER-001`; trade selection, price adjustment,
commit, player XP-orb reward and merchant-menu synchronization remain generic trade behavior.

**Persistence and reload boundary:**

Eye stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no recipe match, advancement progress, brewing slot/fuel/timer/source/target holder or
trade-set draw/offer uses. Those values belong to the recipe manager, player progression, Brewing
Stand, potion contents and merchant offer owners.

Recipe/advancement reload can replace future crafting and unlock behavior. A rebuilt baseline mix
retains the twelve Eye edges while source, ingredient and target are feature-enabled; already
completed brews are not replayed. Trade reload can replace the record, buying tag or set before
future offer generation without rewriting existing offers or stacks. Resource reload independently
controls name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1152` plus the stack's component patch. Its
common-rarity name uses locked English text `Fermented Spider Eye`; the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/fermented_spider_eye` and its
same-named texture. It appears exactly once and only in Ingredients, ordered Dragon's Breath,
Fermented Spider Eye, Blaze Powder.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; shapeless input placement and
recipe knowledge; advancement listener state; stand fuel/timer/container/holder/custom-content and
the twelve mix sources; trade-set membership/order and offer lifecycle; save, recipe/advancement/
mix/trade/resource reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1152`; common rarity; max stack `64`; recipe one each of Spider Eye/Brown Mushroom/
Sugar to one Eye; twelve potion edges; payloads `3600/9600` Invisibility, `1800/4800` Slowness,
`1@0/1@1` Instant Damage and `1800` Weakness; owner brew duration `400`; trade one Eye to three
Emeralds, uses `2`, offer XP `1`, discount `0.05`; two distinct buying records from six and Eye
inclusion `1/3`. Only trade-set selection uses the named RNG here; recipe and brewing add no RNG.

**Side effects:**

Crafting inputs/result and recipe knowledge; Brewing Stand ingredient/bottles/timer/event and
potion/effect state; merchant offer/use/economy/player output state; ordinary stack persistence/
wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; exact three-ingredient shapeless match; Spider Eye
possession or recipe-unlocked criterion; valid Brewing Stand fuel plus a present listed source
holder in a registered potion container; selected nonexhausted trader offer plus valid cost;
registry/stack decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, crafting inputs and recipe knowledge, advancement listeners,
brewing slots/fuel/timer/mix/contents, trader registry/set/offer/economy, persistence and client
resources. Writes only the crafting, progression, brewing, trade, stack and client state listed
above.

**Failure behavior:**

Use has no subtype success or mutation. Missing/extra/wrong crafting inputs produce no result;
unknown recipe only affects applicable generic recipe-book paths. Missing fuel, holderless
contents or any unlisted source prevents brewing; unmatched slots stay unchanged. An unselected,
missing or exhausted trade emits no Emeralds. Missing/replaced recipe, advancement, mix or trade
data removes future paths without rewriting completed state. Client resource absence follows
generic fallback and cannot grant authority.

**Boundary cases and quirks:**

The crafted result cannot unlock its own recipe; Spider Eye can. One patched ingredient Eye can
corrupt three differently containerized matching potions while all its patches and all custom
source-content details are discarded. Long Poison collapses to ordinary Harming. Water is the sole
base/start-state source, directly producing Weakness without Awkward. The trader buys rather than
sells the item, and only one third of baseline offer generations contain that purchase.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.alchemy.PotionBrewing#addVanillaMixes`;
`net.minecraft.world.item.alchemy.PotionBrewing#hasPotionMix`;
`net.minecraft.world.item.alchemy.PotionBrewing#mix`;
`net.minecraft.world.item.alchemy.PotionBrewing$Builder#addMix`;
`net.minecraft.world.item.alchemy.Potions`;
`net.minecraft.world.item.trading.TradeSet#calculateNumberOfTrades`;
`net.minecraft.world.item.trading.VillagerTrade#getOffer`;
`net.minecraft.world.item.trading.ItemCost#test`;
`net.minecraft.world.entity.npc.wanderingtrader.WanderingTrader#updateTrades`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:{item,potion,mob_effect,recipe,advancement,villager_trade,trade_set}`;
`reports/minecraft/components/item/fermented_spider_eye.json`;
`data/minecraft/recipe/fermented_spider_eye.json`;
`data/minecraft/advancement/recipes/brewing/fermented_spider_eye.json`;
`data/minecraft/{villager_trade/wandering_trader/fermented_spider_eye_emerald,tags/villager_trade/wandering_trader/buying,trade_set/wandering_trader/buying}.json`;
`assets/minecraft/{items,models/item,textures/item}/fermented_spider_eye.*`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-BREW-001`; `ITM-ADVANCEMENT-001`;
`ITM-POTION-001`; `ENT-EFFECT-001`; `MOB-WANDERING-TRADER-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-049`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Match the shapeless
recipe across permutations, extra/absent inputs and component patches; trigger each unlock
criterion separately. For Potion/Splash/Lingering containers, brew every listed source and every
unlisted, holderless and custom-content control while tracing three-slot commit and stripped
fields. Build the buying set across every selection/order, use/exhaust/reload its Eye offer with
plain/patched inputs, and trace the named sequence. Persist/synchronize state and capture raw ID,
name, tooltip, model and exact Ingredients position before/after reload.

**Limits:**

This leaf does not duplicate generic recipe matching/result take, Brewing Stand transaction,
potion consumption/projection/effect behavior, merchant pricing/commit, wandering-trader lifecycle
or stack/resource codecs. Those remain with their cited owners; this rule fixes the Eye identity
and its exact crafting, progression, potion-corruption, trade and presentation joins.
