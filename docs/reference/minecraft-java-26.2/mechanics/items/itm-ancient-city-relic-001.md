# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ANCIENT-CITY-RELIC-001` — Ancient-city relics are inert nine-part ingredients with one shared loot source

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `WGEN-JIGSAW-ANCIENT-CITY-001`,
`CLI-001`, `CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations/components, exhaustive code/data references, the
ancient-city chest record, both recipes/advancements and direct client assets determine every
identity-specific source, use, ingredient and projection boundary. Generic stack/inventory
behavior, loot execution, crafting and advancement evaluation remain with their cited owners.

**Applies when:**

An `echo_shard` or `disc_fragment_5` stack is created, looted, moved, renamed, persisted,
synchronized, selected in a tab, used, offered to crafting, rendered or observed before and after
loot/recipe/resource reload.

**Authoritative state:**

`minecraft:echo_shard` is raw item ID `1456`; `minecraft:disc_fragment_5` is raw item ID `1361`.
Both are uncommon, nondamageable, max stack `64`, and belong to no direct item tag. Their registered
components are only the common empty modifiers/enchantments/lore, item-break sound, translated
name, direct item-model key, repair cost, swing animation, tooltip display and use effects. Neither
has food, consumable, cooldown, remainder, tool, jukebox-playable or other operational component.

Echo shard is a base `Item`. Disc fragment is a `DiscFragmentItem`, but its only subtype override
is tooltip construction. It is not a playable record, does not enter the creeper music-disc tag
and has no jukebox interaction; crafting converts nine fragments into the separately owned
playable output.

**Transition and ordering:**

Neither identity overrides hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A stack patched with generic consumable/equippable components may instead activate those component
owners, but no identity-specific branch consumes a stack, starts active use, applies a cooldown,
emits a sound/game event/particle, increments item use, or changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identities add no fuel, brewing, composter, dispenser, equipment, repair, enchantment,
villager, mob or block predicate. Their only server gameplay consumers are the two ingredient
records below.

**Ancient-city acquisition:**

The only standard data source for either item is `chests/ancient_city`, whose placement and chest
seed ownership are fixed by `WGEN-JIGSAW-ANCIENT-CITY-001`. Its first pool draws an integer roll
count uniformly from `5..10`, then selects among 27 entries of total weight `84` on every roll.

Echo shard and disc fragment are distinct adjacent entries, each weight `4`. Selection of either
therefore has probability `4/84 = 1/21` per first-pool roll before branch-specific RNG consumption.
Its selected entry applies exactly one uniform integer `set_count` draw from `1..3` and emits that
default identity with no extra component function. Repeated selection may emit multiple stacks;
generic loot/container handling owns later merging, splitting and slot placement.

The table uses random sequence `minecraft:chests/ancient_city`. Other selected entries can consume
their own function draws between rolls, so later results must continue the actual shared sequence
rather than independently sampling the relic entries. The second one-roll trim/empty pool occurs
after this relic pool and cannot retroactively change its output.

No other locked baseline loot table, structure payload, trade, mob drop, recipe result or
advancement reward creates either relic. Administration and custom data can still create ordinary
stacks through generic item/loot boundaries. The optional Trade Rebalance pack is outside the
default baseline; enabling it replaces the named ancient-city table and must use that pack's full
RNG program.

**Recipes and progression:**

Recovery compass is a shaped equipment-category recipe with pattern `SSS/SCS/SSS`: `S` is echo
shard and `C` compass. It consumes eight shards around one compass and returns one default
`minecraft:recovery_compass`. Input component patches are not copied into the fixed result.

Music Disc 5 is a shapeless recipe whose ingredient list contains nine separate
`minecraft:disc_fragment_5` entries. A valid crafting grid therefore contributes exactly nine
fragments in any shapeless arrangement and returns one default `minecraft:music_disc_5`. Neither
recipe has a remainder.

Each recipe advancement has the same two-criterion OR matrix: possession of its corresponding
relic or prior unlock of that exact recipe grants the advancement, whose reward unlocks that same
recipe. Echo shards do not unlock Music Disc 5, and fragments do not unlock Recovery Compass.
Generic recipe matching, consumption, result transfer, recipe-book packets and reentrant
recipe-unlocked criteria remain with `ITM-RECIPE-001`, `ITM-CRAFT-001` and
`ITM-ADVANCEMENT-001`.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. No
subtype continuation, RNG cursor, recipe identity or ancient-city provenance is stored on the
item. Loot seeds/random-sequence state belong to the container/loot transaction, and recipe-book
known/highlight state belongs to the player.

Loot reload can replace the ancient-city table and thereby its selection/count behavior for
unevaluated chests. Recipe/advancement reload can independently replace ingredient/output/unlock
records without changing already existing relic stacks. Resource reload independently controls
names, tooltip translation and models.

**Client and wire projection:**

Generic item-stack encoding projects raw item IDs `1456` and `1361` plus each stack's component
patch. Both default names use uncommon rarity styling. Echo shard adds no subtype tooltip.
Disc fragment appends one gray translatable line from
`item.minecraft.disc_fragment_5.desc`—locked English text `Music Disc - 5`—and ignores stack,
tooltip-context, tooltip-display and flag values in that override. Its ordinary item-name line
remains separate.

Both direct item definitions select generated models and same-named textures:
`minecraft:item/echo_shard` and `minecraft:item/disc_fragment_5`. Both appear only in Ingredients,
where Echo Shard immediately follows Popped Chorus Fruit, Disc Fragment immediately follows Echo
Shard, and the dye sequence follows the fragment.

**Branches and aborts:**

Identity/count/components; hand/block/container/anvil path; ancient-city chest placement and
deferred evaluation; `5..10` rolls, all 84 weight units, repeated selection and `1..3` count;
baseline versus optional pack; shaped/shapeless grid and count; inventory-versus-recipe-unlocked
criterion; save/reload, raw stack, name/tooltip/model/tab context.

**Constants and randomness:**

Raw IDs echo/fragment `1456/1361`; rarity uncommon; max stack `64`; first-pool rolls `5..10`;
total weight `84`; each relic weight `4` and per-roll selection `1/21`; selected count `1..3`;
recipe inputs echo `8+1 compass`, fragment `9`; outputs `1`; one fragment subtype tooltip line.
There is no item-use randomness.

**Side effects:**

Loot stacks/container slots and named-sequence cursor; crafting inputs/results; recipe advancement,
known/highlight and recipe-book projection under generic owners; ordinary stack persistence/wire
state; uncommon names, one fragment description line, direct models and Ingredients-tab entries.

**Gates:**

Generic inventory/container/anvil admission; retained ancient-city chest and valid loot table/seed;
first-pool roll and weighted selection; count provider; exact recipe ingredients/grid; either
advancement criterion; valid stack/registry decode; client language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, ancient-city loot registry/
seed/random sequence, recipe/advancement registries and player recipe state, persisted stack and
client resources. Writes only the loot, crafting/progression, stack and client projection listed
above.

**Failure behavior:**

Use has no subtype success or mutation. A relic not selected by loot emits nothing; failed chest
placement never evaluates its table. Invalid or insufficient crafting leaves inputs unchanged
under the generic owner. Missing/replaced recipe and advancement data remove those future paths
without rewriting stacks. Client resource absence follows generic missing translation/model
fallback and cannot grant item authority.

**Boundary cases and quirks:**

The similarly sourced relics are not interchangeable ingredients. Nine disc fragments make a
record but are not themselves jukebox-playable. Eight echo shards require the center compass; nine
shards alone do not craft. Per-roll relic selection is `1/21`, but whole-chest probability is not
an independent fixed-count binomial because the roll count is random and other selected functions
can advance the shared sequence. The gray fragment description is class behavior, not a component
or lore entry.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.DiscFragmentItem`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.data.loot.packs.VanillaChestLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/{echo_shard,disc_fragment_5}.json`;
`data/minecraft/loot_table/chests/ancient_city.json`;
`data/minecraft/recipe/{recovery_compass,music_disc_5}.json`;
`data/minecraft/advancement/recipes/{tools/recovery_compass,misc/music_disc_5}.json`;
`assets/minecraft/items/{echo_shard,disc_fragment_5}.json`;
`assets/minecraft/models/item/{echo_shard,disc_fragment_5}.json`;
`assets/minecraft/textures/item/{echo_shard,disc_fragment_5}.png`;
`WGEN-JIGSAW-ANCIENT-CITY-001`; `ITM-LOOT-001`; `ITM-RECIPE-001`;
`ITM-ADVANCEMENT-001`; `CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-037`.

**Test vectors:**

Exercise default/patched stacks through both hands, blocks, containers and anvil at all count
boundaries. Evaluate retained ancient-city chests across all roll counts, every entry, repeated
relic selections/counts and baseline/optional-pack reload boundaries while recording the exact
named-sequence cursor. Craft both recipes across grid sizes/arrangements/counts and both unlock
criteria, then reload data. Persist/synchronize stacks and capture raw IDs, rarity names, fragment
description, direct models and exact Ingredients order before/after resource reload.

**Limits:**

This leaf does not duplicate generic loot selection/container placement, stack/inventory/anvil
semantics, crafting consumption, recipe-book/advancement state or the behavior of Recovery Compass
and Music Disc 5 outputs. Those remain with their cited owners; this rule fixes the two relic
identities and their exact acquisition, ingredient and presentation joins.
