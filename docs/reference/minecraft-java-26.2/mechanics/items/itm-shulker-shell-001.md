# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SHULKER-SHELL-001` — Shulker Shells convert one Looting-scaled mob drop into an empty Shulker Box

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`,
`ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`,
`ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`, `ITM-CRAFT-001`,
`ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `ITM-ANVIL-001`, `ENT-001`, `ENT-005`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-SPAWN-001`,
`WGEN-STRUCTURE-END-CITY-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked plain-item registration/components, exhaustive code/data
references, Shulker entity table, one Shulker Box recipe and unlock, icon-only End advancement and
direct client assets determine every Shell-specific branch. Generic Shulker lifecycle/death, loot,
crafting, progression, stacks and inventories remain with the cited owners.

**Applies when:**

A `shulker_shell` stack is created, looted, moved, renamed, persisted, synchronized, offered to
crafting, selected in a tab, rendered or observed before and after loot, recipe or resource reload.

**Authoritative state:**

`minecraft:shulker_shell` is raw item ID `1334`. It registers through the plain-item path with
default properties, is common, nondamageable and has max stack `64`. It belongs to no direct item
tag.

Its registered components are only the common empty modifiers/enchantments/lore, item-break sound,
translated name, direct item-model key, repair cost, swing animation, tooltip display and use
effects. It has no food, consumable, cooldown, remainder, tool, equipment, repairable or
identity-specific glint state.

**Transition and ordering:**

The identity does not override hand use or block use. A prototype stack's air use returns generic
`PASS`; a block click participates only in ordinary block-first interaction and fallback handling.
A component-patched stack can activate a generic component owner, but the identity itself never
consumes a stack, starts active use, emits a sound/game event/particle, increments item use or
changes the world.

Container movement, pickup, dropping, anvil naming and component patching use their generic owners.
The identity adds no dispenser, mob-interaction, equipment, repair, fuel, brewing, composting or
villager branch. Shulker loot and the recipe consumer own the operational joins below.

**Shulker acquisition:**

The only locked baseline source is the single pool of `entities/shulker`. Every admitted table
evaluation makes one pool roll and evaluates an enchanted-chance condition; there is no
`killed_by_player` condition.

With no positive Looting level the chance is `0.5`. If the attacking entity is living and has
Looting level `L > 0`, the configured linear value is
`0.5625 + 0.0625 * (L - 1)`, equivalently `0.5 + 0.0625L`. The condition consumes one float and
passes exactly when `nextFloat() < chance`. A pass emits one default Shell with no count function;
a failure emits none. At `L = 8` the chance is exactly one, so every possible `nextFloat()` passes.

The table uses random sequence `minecraft:entities/shulker`. Player kill credit is neither required
nor consulted by this entry. An absent or nonliving attacking entity supplies `L = 0` and retains
the 50-percent branch. Generic death/loot admission can still suppress the entire table before its
condition, while a table reload can replace it for future deaths.

End-city template markers create a baseline population route under
`WGEN-STRUCTURE-END-CITY-001`. Shulker spawning, projectile attacks and duplication can change
which entities later reach death, but do not directly create item stacks; their behavior is
outside this material rule. Death context, table invocation, empty-stack filtering and world-drop
placement remain with `ENT-DEATH-001`, `ENT-ENTITY-DROPS-001` and `ITM-LOOT-001`.

No bundled chest, fishing, block-drop, trade or other entity table directly emits a Shell.
Administration and custom data can still create ordinary stacks through generic item/loot
boundaries.

**Recipe and progression:**

The sole bundled recipe consuming Shell is a one-column shaped `shulker_box` recipe: one Shell,
one Chest, then one Shell vertically. It returns one default, undyed Shulker Box. The output is
empty; the Chest's inventory/name/component patch is not copied, neither Shell has a remainder and
the inputs are consumed by the generic crafting transaction.

The grid can be horizontally translated within any admitted crafting grid, but its top-to-bottom
order cannot be rotated or inverted into another pattern. Later dyeing recipes consume the
resulting box plus dye and are not additional Shell recipes. Placement, inventory retention,
breaking, dispenser placement and rendering of the resulting box remain with their respective
block/container owners.

The recipe advancement places Shell possession and exact `shulker_box` recipe-unlocked criteria
in one two-entry OR requirement; either awards only the Shulker Box recipe.

The separate `end/levitate` challenge uses a Shell only as its display icon. Its criterion requires
the Levitation trigger to record at least 50 vertical distance, rewards 50 experience and sends its
configured telemetry event. It neither checks for nor rewards a Shell, and a Shell stack cannot
satisfy that criterion.

**Persistence and reload boundary:**

Stacks persist and synchronize identity, count and arbitrary ordinary component patches. They
store no death context, attacker, Looting level, chance draw/table cursor, recipe identity or
advancement progress. Those values belong to the death/loot transaction, recipe manager and player
progression state.

Loot reload can replace the Shulker table for future deaths without rewriting stacks.
Recipe/advancement reload can independently replace crafting, unlock and icon records. Existing
Shulker Boxes neither retain nor reconstruct the consumed Shells. Resource reload independently
controls the Shell name and model.

**Client and wire projection:**

Generic item-stack encoding projects raw item ID `1334` plus the stack's component patch. Its
common-rarity name uses locked English text `Shulker Shell` and the plain class adds no subtype
tooltip or forced glint.

The direct item definition selects generated model `minecraft:item/shulker_shell` and its
same-named texture. It appears exactly once and only in Ingredients, ordered Ender Eye, Shulker
Shell, Popped Chorus Fruit.

**Branches and aborts:**

Identity/count/components; generic hand/block/container/anvil path; admitted Shulker death/table,
attacking entity, Looting level and chance draw; shaped grid translation/order/counts; Shell
possession versus recipe-unlocked and icon-only Levitation criterion; save, loot/recipe/resource
reload, wire, language, model and tab context.

**Constants and randomness:**

Raw item ID `1334`; common rarity; max stack `64`; base drop chance `0.5`; positive-Looting chance
`0.5 + 0.0625L`; one Shell on success; recipe two Shells plus one Chest to one default Shulker Box;
Levitation icon challenge minimum vertical distance `50` and reward `50` experience. The entity
table chance float is the only Shell-specific random draw.

**Side effects:**

Possible loot stack and named-sequence cursor; generic world drop/pickup; crafting inputs and empty
box result; advancement and recipe known/highlight/experience/telemetry state where applicable;
ordinary stack persistence/wire state; name, direct model and one Ingredients-tab entry.

**Gates:**

Generic stack/container/anvil admission; valid Shulker death/table and enchanted-chance condition;
exact crafting grid and ingredient counts; exact inventory or recipe-unlocked criterion; owner
Levitation trigger for the icon-only advancement; valid registry/stack decode; client
language/model and tab bootstrap.

**State read/written:**

Reads stack identity/count/components, interaction/container state, Shulker death/attacker context,
loot random sequence, recipe/advancement registries and player progression state, persisted stack
and client resources. Writes only the loot, crafting, progression, stack and client projection
listed above.

**Failure behavior:**

Use has no subtype success or mutation. A suppressed/missing table or failed chance emits no Shell.
Invalid or insufficient crafting leaves inputs unchanged, and input patches are never copied to
the default result. Shell possession does not complete the Levitation challenge. Missing/replaced
loot, recipe or advancement data removes those future paths without rewriting stacks. Client
resource absence follows generic missing translation/model fallback and cannot grant authority.

**Boundary cases and quirks:**

The loot entry lacks a player-kill gate even though Looting still depends on a living attacking
entity. Its configured chance reaches certainty at Looting VIII. The recipe output is a new empty
container rather than a wrapper around the input Chest. The challenge icon is presentation only,
not another acquisition or possession path.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.world.level.storage.loot.predicates.LootItemRandomChanceWithEnchantedBonusCondition#test`;
`net.minecraft.data.loot.packs.VanillaEntityLoot`;
`net.minecraft.data.recipes.packs.VanillaRecipeProvider`;
`net.minecraft.data.advancements.packs.VanillaTheEndAdvancements`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/shulker_shell.json`;
`data/minecraft/loot_table/entities/shulker.json`;
`data/minecraft/recipe/shulker_box.json`;
`data/minecraft/advancement/{recipes/decorations/shulker_box,end/levitate}.json`;
`assets/minecraft/{items,models/item,textures/item}/shulker_shell.*`;
`ITM-LOOT-001`; `ITM-RECIPE-001`; `ITM-ADVANCEMENT-001`; `ENT-DEATH-001`;
`ENT-ENTITY-DROPS-001`; `WGEN-STRUCTURE-END-CITY-001`; `CLI-UI-001`;
`CLI-EFFECT-001`; `EXP-ITM-042`.

**Test vectors:**

Exercise default/patched stacks through hands, blocks, containers and anvil. Evaluate Shulker
deaths with and without generic loot admission across absent/nonliving/living attackers, every
Looting level and chance boundary while tracing the named sequence. Match/craft every translated
and invalid one-column pattern with default and patched Chest/Shell inputs; trigger both recipe
criteria and the independent Levitation challenge. Persist/synchronize stacks and capture raw ID,
name, tooltip, model and exact Ingredients position before/after data/resource reload.

**Limits:**

This leaf does not duplicate End-city generation, Shulker spawning/AI/duplication/death, generic
loot emission, crafting consumption, Shulker Box runtime, or recipe-book/advancement state. Those
remain with their cited owners; this rule fixes the Shell identity, exact loot chance, recipe,
progression and presentation joins.
