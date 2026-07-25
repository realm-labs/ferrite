# Items, inventories and progression mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-FIREWORK-STAR-001` — Firework Star is a component-bearing two-stage craft whose explosion record feeds Rocket effects and client tint/tooltips

**Parent:** `SIM-004`, `SIM-005`, `PLY-005`, `PLY-006`, `PLY-INPUT-001`,
`PLY-INTERACT-001`, `ITM-001`, `ITM-002`, `ITM-003`, `ITM-004`, `ITM-005`,
`ITM-006`, `ITM-007`, `ITM-USE-001`, `ITM-CONTAINER-001`, `ITM-RECIPE-001`,
`ITM-RECIPE-SERIALIZER-001`, `ITM-CRAFT-001`, `ITM-ADVANCEMENT-001`,
`ITM-ANVIL-001`, `ENT-001`, `ENT-PROJECTILE-001`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, all three special Firework recipe records and
serializers, explosion/fireworks component codecs, complete loot/trade/template absences and exact
client tint/model/language resources determine every Firework-Star-specific branch. Generic
crafting transaction, Rocket item/entity, persistence, packet, tooltip and renderer algorithms
retain the cited owners.

**Applies when:**

`minecraft:firework_star` is created by the base special recipe, recolored by the fade special
recipe, consumed by Firework Rocket crafting, moved, renamed, component-patched, persisted,
synchronized, inspected or rendered before and after recipe or resource reload.

**Authoritative state:**

Firework Star is raw item ID `1273`, a common nondamageable plain `Item` with maximum stack `64`
and no direct item tag. Its default/creative component map does **not** contain
`firework_explosion`; it has no food, consumable, remainder, fuel, compost, equipment, durability,
projectile, cooldown, trim, repair, inventory-tick or identity-specific use branch.

An optional `minecraft:firework_explosion` patch carries:

- shape enum `small_ball`, `large_ball`, `star`, `creeper` or `burst`, with stream IDs `0..4`;
- ordered integer primary-color and fade-color lists, each defaulting empty; and
- `has_trail` and `has_twinkle`, each defaulting false.

`FireworkExplosion.DEFAULT` is small ball, both lists empty and both flags false. The component is
the behavioral payload; item identity alone does not imply it.

**Transition and ordering:**

### Base Firework-Star special recipe

`firework_star` is always available and has no recipe advancement. Its locked ingredients are one
exact Gunpowder fuel, live `dyes`, exact Diamond trail, exact Glowstone-Dust twinkle, and these
shape selectors:

| Shape | ingredient |
|---|---|
| `small_ball` | no shape ingredient |
| `large_ball` | exact Fire Charge |
| `star` | exact Gold Nugget |
| `creeper` | live `skulls` tag |
| `burst` | exact Feather |

Match first requires at least two occupied slots. Cropped row-major scanning classifies each
nonempty stack in this order: twinkle, trail, fuel, component-bearing dye, then shape. A second
twinkle, trail, fuel or shape rejects immediately; any unclassified stack rejects. Match succeeds
only with exactly one fuel and at least one stack that both matches live `dyes` and actually has a
`DYE` component. Any number of such dyes fits subject to the `3x3` grid.

Assembly begins with small ball, false flags and an empty primary-color list. It scans row-major,
resolves a shape first, then twinkle, trail and dye; each dye contributes its
`DyeColor.getFireworkColor()` integer in encounter order. Fuel contributes no payload field. The
result template creates one default Firework Star and sets a new explosion record with that shape,
ordered primary colors, empty fade colors and flags. Input patches are otherwise not copied.

The match/assembly classifier orders intentionally differ, although locked ingredients are
disjoint. Recipe or tag reload that creates overlap remains governed by
`ITM-RECIPE-SERIALIZER-001`; completed results are not revisited.

### Fade special recipe

`firework_star_fade` is also always available and has no advancement. It requires exactly one
identity-matching Firework Star and at least one live `dyes` member carrying `DYE`; multiple dyes
are allowed and every other identity rejects. Match scans dye before target and rejects a second
target.

Assembly collects dye firework colors in row-major order, creates a one-count result while
preserving the target Star's original components, then replaces only
`firework_explosion.fade_colors` through `withFadeColors`. Existing shape, primary colors, trail
and twinkle remain unchanged. Existing fade colors are replaced, not appended.

A target with no explosion component still matches. Update begins from
`FireworkExplosion.DEFAULT`, producing small ball with empty primary colors, the new fade list and
false flags while preserving unrelated target patches. Thus default/creative Firework Stars are
valid fade inputs even though their pre-craft tooltip has no explosion payload.

### Firework Rocket special recipe

`firework_rocket` is the only bundled recipe that consumes Firework Star. It is always available,
has no advancement, and requires exactly one Paper shell, `1..3` exact Gunpowder fuel stacks, any
number of exact Firework Stars including zero, and no other identity. Match tests shell before
fuel before Star; a second shell or fourth fuel rejects.

Assembly scans row-major, counts fuel and appends each Star's explosion component only when it is
present. It emits three default Firework Rockets with
`FIREWORKS.flight_duration=fuelCount` and the ordered copied explosion list. A componentless Star
still matches and is consumed but contributes no explosion; unrelated Star patches never copy.
With one/two/three fuel, a `3x3` grid can contain at most seven/six/five Stars.

Downstream Rocket behavior reads the copied records, not the consumed Stars. Flight duration
selects lifetime `10*(1+flight)+nextInt(6)+nextInt(7)`; explosion-list size contributes to
line-of-sight radius-five damage `5 + 2*explosionCount`. Shape, colors, fade, trail and twinkle
drive client explosion geometry/color/trail/sound effects through the Rocket/entity/effect owners.
Componentless consumed Stars increase neither explosion count nor those effects.

### Acquisition and absence

The base recipe is the sole bundled ordinary acquisition path; fade returns the same identity and
Rocket crafting is its sole bundled sink. No loot, entity, chest, fishing, archaeology, gift,
barter or merchant record emits or consumes Firework Star. An exact UTF scan finds zero Firework
Star identity strings across all `1,212` structure templates. Creative/admin/component commands
remain generic creation paths.

**Persistence and reload boundary:**

Stacks persist item ID, count and arbitrary component patches, including the complete explosion
record when present. Equal identity is insufficient for stacking when patches differ. Recipe
reload changes future match/classification/result templates only; existing default, exploded or
faded Stars and already crafted Rockets retain their records. Resource reload independently
changes names, model, tint and tooltip projection only.

Generic stack/dynamic-registry codecs encode shape, ordered color lists and both flags. The shape
stream uses IDs `0..4`; component defaults apply only when fields/components are absent as
described above. Decode validation and malformed-component failure remain generic.

**Wire and client projection:**

Generic stack publication uses item ID `1273` plus patches; no Firework-Star-specific packet
exists. The English item name is `Firework Star`.

The model is a two-layer `item/generated` flat: untinted base `firework_star` and overlay
`firework_star_overlay`. Layer zero has constant white tint. The overlay uses the current
explosion's primary colors only:

- absent component or empty primary list: opaque default `#8A8A8A`;
- one primary color: that RGB made opaque; or
- multiple primary colors: opaque per-channel integer average of every ordered RGB value.

Shape, fade colors, trail and twinkle do not affect item tint. A default/creative Star and a
fade-only Star are therefore gray.

When the explosion component is visible under generic tooltip policy, its gray lines appear in
this order: localized shape; primary color names when nonempty; `Fade to` plus fade names when
nonempty; `Trail` when true; `Twinkle` when true. The five shape names are `Small Ball`,
`Large Ball`, `Star-shaped`, `Creeper-shaped` and `Burst`. Exact DyeColor RGB values map back to
their localized color names; arbitrary values display `Custom`. A componentless default Star adds
none of these lines.

Ingredients orders Book, Firework Star, Glass Bottle, Nether Wart, Redstone and Glowstone Dust.
The tab publishes the componentless gray default stack once. There is no conditional geometry,
animation or special renderer for the item itself.

**Branches and aborts:**

Componentless versus arbitrary explosion/patched Star; five base shapes, ordered primary colors
and two modifiers; fade absent/replaced/default-synthesized; Rocket `1..3` fuel, zero/multiple/
componentless/component-bearing Stars; tooltip visibility; zero/one/multiple primary-color tint;
persistence/reload/wire/client paths are distinct.

**Constants and randomness:**

Firework Star ID `1273`; stack `64`; shapes/stream IDs `5/0..4`; special recipes/listeners
`3/0`; base result `1`, required fuel/dyes `1/>=1`; Rocket fuel `1..3`, result `3`, maximum Stars
`7/6/5`; Rocket lifetime `10*(1+flight)+nextInt(6)+nextInt(7)` and damage
`5+2*explosionCount`; templates/matches `1212/0`; default tint `#8A8A8A`.

**Side effects:**

One base or faded Star result; component patches and tooltip/tint projection; three Rocket results
with flight/explosion payload; crafting consumption; later Rocket lifetime, damage and client
effects through owners; durable stack/Rocket state and synchronization.

**Gates:**

Recipe availability; exact/live ingredient tests; `DYE` component; duplicate modifier/fuel/shape/
target limits; target component state; grid/result capacity; stack/component decode; tooltip
visibility and client resources.

**State read/written:**

Reads all gates above and writes only crafted Star/Rocket, explosion/fireworks components,
consumption, durable, wire, tooltip and projection state listed above.

**Failure behavior:**

Missing fuel/dye/shell, excess duplicate/shape/fuel, non-component dye or foreign identity rejects
without a result. Fade with a componentless Star succeeds via default explosion rather than
failing. Rocket crafting consumes a componentless Star but omits it from the explosion list.
Reload affects future evaluation only; decode/resource failure follows generic policy.

**Boundary cases and quirks:**

The ordinary item has no default explosion component despite its name and colored overlay.
Firework-Star fade requires identity plus dye, not a preexisting explosion. Base crafting replaces
all defaults with one new record, fade preserves unrelated patches and replaces only fades, while
Rocket crafting copies only non-null explosion records. Fade colors never tint the inventory
icon. None of the three special recipes publishes knowledge or an advancement.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-REPORT-001`; `OFF-DATA-001`;
`net.minecraft.world.item.Items`;
`net.minecraft.world.item.crafting.FireworkStarRecipe`;
`net.minecraft.world.item.crafting.FireworkStarFadeRecipe`;
`net.minecraft.world.item.crafting.FireworkRocketRecipe`;
`net.minecraft.world.item.crafting.TransmuteRecipe#createWithOriginalComponents`;
`net.minecraft.world.item.component.FireworkExplosion`;
`net.minecraft.world.item.component.FireworkExplosion$Shape`;
`net.minecraft.world.item.component.Fireworks`;
`net.minecraft.client.color.item.Firework`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,recipe,recipe_serializer}`;
`reports/minecraft/components/item/firework_star.json`;
`data/minecraft/recipe/{firework_star,firework_star_fade,firework_rocket}.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/items/firework_star.json`;
`assets/minecraft/models/item/firework_star.json`;
`assets/minecraft/textures/item/{firework_star,firework_star_overlay}.png`;
`assets/minecraft/lang/en_us.json`;
`ITM-RECIPE-SERIALIZER-001`; `ENT-PROJECTILE-001`; `EXP-ITM-086`.

**Test vectors:**

Run `EXP-ITM-086` across default and arbitrary-patched Stars; all five shapes, every dye ordering,
modifier duplicate and malformed grid; absent/existing/repeated fade records; Rocket fuel
`0..4`, zero/multiple/componentless/component-bearing Stars and capacity boundaries. Scan every
loot/trade/template source, reload recipes/resources, persist/synchronize all Star/Rocket records
and assert ID, tooltip ordering, tint arithmetic, model layers and tab order.

**Limits:**

Generic crafting transaction, special-recipe framework, Rocket item/entity lifetime, damage,
particle/sound behavior, stack/component codec, packet, tooltip and renderer control flow remains
with cited owners. Gunpowder, Dye, Diamond, Glowstone Dust, shape inputs, Paper and Firework Rocket
retain their own leaves. This leaf fixes exact Firework Star identity, component transitions,
source/sink joins, absences and projection.
