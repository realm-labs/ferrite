# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-EGG-001` — Three egg identities preserve chicken variant through laying, flight, hatching and recipes

**Parent:** `PLY-005`, `PLY-006`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ENT-001`, `ENT-004`, `ENT-005`, `MOB-004`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, egg item/projectile and chicken bytecode,
recipes, advancement, tag, gift loot table and client assets close all three identities and their
throw, hatch, laying, persistence, reload and projection joins.

**Applies when:**

An ordinary, blue or brown egg is laid, crafted with, thrown by a player or dispenser, flown,
collided, hatched, persisted, reloaded or projected; or its `chicken/variant` component is patched.

**Authoritative state:**

| item | raw item ID | default `minecraft:chicken/variant` | normal laying source |
|---|---:|---|---|
| `egg` | `1060` | `minecraft:temperate` | temperate chicken |
| `blue_egg` | `1061` | `minecraft:cold` | cold chicken |
| `brown_egg` | `1062` | `minecraft:warm` | warm chicken |

All three are common `EggItem` instances with maximum stack size `16`, direct membership in
`#minecraft:eggs`, no use duration/cooldown and otherwise ordinary nondamageable item defaults.
Registration order is ordinary, blue, brown. The shared thrown entity is protocol ID `39`,
belongs directly to `#minecraft:impact_projectiles`, and stores one full egg stack through its
synced item data. A command-patched component therefore participates in hatching independently
of the item's registered identity.

**Transition and ordering:**

### Held throw and dispenser launch

Held use first reads the selected hand, then plays `entity.egg.throw` at the player's position in
the PLAYERS source with volume `0.5` and pitch
`0.4 / (levelRandom.nextFloat() * 0.4 + 0.8)`. Only the server creates the projectile. It starts
at player X/Z and eye Y minus `0.10000000149011612`, records the player as owner, copies the held
stack with count one, then shoots from player rotation with angle offset `0`, power `1.5` and
uncertainty `1.0`.

After the server-only spawn branch, both logical sides invoke exact-item `awardStat`, consume one
through the generic living-entity ability rule, and return success. The base client-side stat sink
is a no-op; the server records `item_used`. Survival prediction/authority shrink one while an
infinite-material player retains the stack; stack, statistic, sound and entity projections
converge through their generic protocol owners.

Every egg implements `ProjectileItem`. `ITM-DISPENSER-001` owns scheduling and slot selection; its
projectile branch calls this item's factory at dispenser center plus `0.7 * facing + (0,0.1,0)`.
The projectile copies one exact stack, shoots along facing with power `1.1` and uncertainty `6.0`,
the source stack shrinks one, and the default projectile dispense event is `1002`.

### Flight, collision and hatch

`ENT-PROJECTILE-001` owns generic shooting, collision, portals and motion. The egg specialization
uses gravity `0.03`, air inertia `0.99` and water inertia `0.8`; water flight emits four bubbles per
tick. An entity hit first invokes the generic projectile hook, then calls the target's thrown
damage route with value `0.0`; block hits skip that damage call. Both then enter the same common
hit transaction.

On the server every hit first consumes `projectileRandom.nextInt(8)`. A nonzero result produces no
chicken. Zero selects an initial count one and consumes `nextInt(32)`; zero changes the count to
four. Therefore no/one/four-chick probabilities are `7/8`, `31/256` and `1/256`.

For each requested chick, entity creation uses spawn reason `TRIGGERED`. A null creation skips only
that iteration. A created chick receives age `-24000`, projectile position, projectile yaw and
pitch zero. If the stored egg has `chicken/variant`, that holder replaces the chick's variant; if
the component is absent, the newly created chick retains its own default. Position correction is
then tested against fixed zero-sized old dimensions. Failure breaks the entire hatch loop before
insertion; success calls `addFreshEntity`, whose Boolean result is ignored before any later
iteration.

After the hatch branch, including every failure and no-hatch result, the server broadcasts entity
event `3` and discards the projectile. The client handles event `3` only when its synchronized
stored stack is nonempty: it constructs an item particle from that exact stack and emits eight
particles at projectile position, each velocity component
`(projectileRandom.nextFloat() - 0.5) * 0.08`. These client draws do not affect server hatch RNG.

### Chicken laying

A new chicken initializes `eggTime` to `nextInt(6000) + 6000`, hence `6000..11999`. On each server
AI step, only an alive adult non-jockey decrements it; when the decremented value is at most zero,
the chicken evaluates `gameplay/chicken_lay` through the generic gift-loot owner. Its one-roll
ordered alternatives emit exactly one ordinary egg for temperate, one brown egg for warm, or one
blue egg for cold. There is no fallback: a component-patched variant outside those three emits
nothing.

When gift evaluation emitted an item, the chicken then plays `entity.chicken.egg` with volume `1`
and pitch `(nextFloat() - nextFloat()) * 0.2 + 1.0`, and emits `ENTITY_PLACE`. Whether gift
evaluation succeeded or failed, it finally resets `eggTime` with one
`nextInt(6000) + 6000`. Baby, dead, jockey and client ticks neither decrement nor reset the timer.

### Recipes and progression

The locked `#minecraft:eggs` tag contains exactly ordinary, blue and brown egg. Cake's shaped grid
is milk buckets across the top, sugar/egg-tag/sugar in the middle and wheat across the bottom.
Pumpkin pie is shapeless pumpkin, sugar and one egg-tag member. Both emit one default result and
generic crafting owns input consumption and remainders.

Cake's recipe advancement can unlock from direct recipe grant or possession of any egg-tag
member. Pumpkin pie's bundled unlock instead tests pumpkin or carved pumpkin, so egg possession
alone does not unlock it. No bundled recipe emits an egg, and no fuel, compost, trade or chest-loot
record directly consumes or emits one; ordinary acquisition is the chicken gift table.

### Projection

Each identity has a direct like-named generated item model and texture. The common
`ThrownItemRenderer` billboards the synchronized stored stack in ground display context, so blue
and brown throws retain their visible identity and component patch. A projectile without saved
`Item` reconstructs an ordinary egg. For its first two ticks, squared camera distance below
`12.25` suppresses rendering; ordinary distance culling owns all later visibility.

Ingredients and Combat both order snowball, ordinary egg, brown egg, blue egg. Ingredients then
places leather; Combat then places wind charge. This brown-before-blue tab order differs from the
ordinary/blue/brown raw registration order.

**Branches and aborts:**

Ordinary/blue/brown/component-patched stack; player/dispenser; client/server; survival/infinite
materials; block/entity hit; 0/1/4 hatch; null creation; position-fudge failure; insertion failure;
present/absent/alternate variant; alive/dead, adult/baby and jockey/non-jockey layer; all three/no
loot alternatives; recipe/tag/unlock snapshots; saved/missing projectile item; render age/range.

**Constants and randomness:**

Item IDs `1060/1061/1062`; entity ID `39`; stack maximum `16`; held sound volume `0.5`, pitch
`0.4/(U*0.4+0.8)`; held power/uncertainty `1.5/1`; dispenser offset `0.7` and `0.1`,
power/uncertainty `1.1/6`, event `1002`; eye offset `0.10000000149011612`; gravity/inertia
`0.03/0.99/0.8`; hatch draws `nextInt(8)` then conditionally `nextInt(32)`; chick age `-24000`;
event `3`; eight particles, three floats each and scale `0.08`; laying interval `6000..11999`;
lay sound pitch `(U1-U2)*0.2+1`; near-render squared distance `12.25` and age `2`.

**Side effects:**

Held/dispenser stack count, item-used statistic, thrown entity/item data/owner/motion, optional
zero-damage target hook, zero/one/four baby chickens, event/particle/discard state, chicken timer,
laid item, sounds, game event, recipe output/remainders, unlock state and client projection.

**Gates:**

Logical side, player ability, dispenser scheduled selection, projectile collision, hatch draws,
entity creation/position/insertion, egg component, chicken life/age/jockey/timer and current gift
table/variant/tag/recipe/advancement/resource snapshots.

**State read/written:**

Reads held stack/hand, player position/rotation/ability, dispenser/facing, projectile owner/item/
position/motion/RNG, collision target, chicken variant/timer/life/age/jockey state and current
data/resources. Writes stack count/statistic, projectile state, optional target damage call,
chick entities, event/discard/particles, laying timer/item/sound/event, crafting and progress.

**Failure behavior:**

Client held use does not spawn an authoritative projectile. No hatch still emits event `3` and
discards. Null chick creation skips one iteration; position correction failure aborts all
remaining chicks; failed world insertion is ignored. Missing egg variant retains the chick
default. A nonmatching chicken variant lays nothing but still resets its timer. Invalid recipe
inputs and nonmatching unlock criteria commit nothing.

**Persistence boundary:**

Stacks persist identity and component patch generically. A thrown egg persists owner/motion and
stores one stack under `Item` through `ItemStack.CODEC`; missing `Item` defaults to ordinary egg.
Chicken persists its variant and `EggLayTime`. Flight, collision, hatch draws/loop, laying
evaluation, sounds, particles and crafting attempts never resume. Data reload replaces the egg
tag, cake/pumpkin-pie recipes, cake advancement, chicken-lay gift table and chicken-variant
registry snapshot without replaying completed transactions; resource reload replaces the three
item assets independently.

**Boundary cases and quirks:**

The item identity and hatch variant are separable: any of the three can carry a patched variant,
and removing the component retains a newly created chick's default. Entity hits make a zero-value
thrown-damage call before the common hatch/discard path. Hatch insertion failure does not stop
later iterations, but position-fudge failure does. All impacts consume the first hatch draw and
emit the same eight-item-particle event. A custom chicken variant lays no egg yet resets its
timer. Creative-tab order deliberately disagrees with raw registration order.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.EggItem`;
`net.minecraft.world.item.ProjectileItem`;
`net.minecraft.core.dispenser.ProjectileDispenseBehavior`;
`net.minecraft.world.entity.projectile.Projectile`;
`net.minecraft.world.entity.projectile.ThrowableProjectile`;
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrowableItemProjectile`;
`net.minecraft.world.entity.projectile.throwableitemprojectile.ThrownEgg`;
`net.minecraft.world.entity.animal.chicken.Chicken`;
`net.minecraft.client.renderer.entity.ThrownItemRenderer`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{egg,blue_egg,brown_egg}.json`;
`data/minecraft/tags/item/eggs.json`;
`data/minecraft/recipe/{cake,pumpkin_pie}.json`;
`data/minecraft/advancement/recipes/food/cake.json`;
`data/minecraft/loot_table/gameplay/chicken_lay.json`;
`assets/minecraft/{items,models/item,textures/item}/{egg,blue_egg,brown_egg}.*`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-DISPENSER-001`; `ITM-RECIPE-001`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ENT-PROJECTILE-001`;
`ENT-DAMAGE-001`; `MOB-AI-001`; `CLI-EFFECT-001`; `EXP-ITM-026`.

**Test vectors:**

Throw all three and component-patched stacks from both hands in survival/infinite-material modes
and from every dispenser facing. Replay block/entity impacts at exact 0/1/4 hatch draws, including
null creation, fudge failure and failed insertion; verify zero damage, variant, age, event,
particle and discard order. Sweep every chicken life/age/jockey/timer/variant branch and gift
result. Craft/unlock with all tag members, persist missing/present projectile `Item` and chicken
timer, reload data/resources, and inspect flight plus Ingredients/Combat projection.

**Limits:**

This leaf does not duplicate generic use-packet convergence, dispenser scheduling, projectile
collision/motion, entity damage admission, gift-loot evaluation, entity/stack codecs, crafting,
advancement or resource-pack algorithms. Those remain with the cited owners; this rule fixes the
three identities, their exact variant-preserving joins and observable ordering.
