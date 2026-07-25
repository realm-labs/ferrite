# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-STEERING-STICK-001` — Food-on-a-stick items commit mount boost before durability and convert a broken stack to a patched fishing rod

**Parent:** `PLY-005`, `PLY-006`, `ITM-001`, `ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`,
`ITM-007`, `ENT-001`, `ENT-002`, `ENT-005`, `MOB-004`, `MOB-005`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item registration, shared use hook, pig/strider controller and temptation
code, durability conversion, recipes, advancements, tags and client assets close both
`FoodOnAStickItem` identities.

**Applies when:**

Carrot on a stick or warped fungus on a stick is held-used, used to control or tempt its exact
mount, damaged, broken, crafted, enchanted, persisted, reloaded or projected.

**Authoritative state:**

| item | item ID | exact boost target | boost damage | maximum damage | temptation |
|---|---:|---|---:|---:|---|
| `carrot_on_a_stick` | `887` | saddled pig controlled by its first passenger | `7` | `25` | exact code-built pig goal |
| `warped_fungus_on_a_stick` | `888` | saddled strider controlled by its first passenger | `1` | `100` | live `strider_tempt_items` membership |

Both are common, maximum-stack-one items with default damage zero, the ordinary item-break sound,
no family-specific attribute modifier and direct durability-enchantable membership. The item-to-
target mapping and damage costs are code-built, not tag-selected.

**Transition and ordering:**

#### Held boost use

The generic spectator and cooldown gates precede the item hook. Once invoked, the client hook
always returns pass and performs no prediction. The server obtains the used-hand stack and the
player's controlled vehicle. A controlled vehicle exists here only when the player is the first
passenger of a saddled pig/strider and is holding that vehicle's exact controller in either hand.
The hook additionally requires passenger state, an `ItemSteerable` vehicle, and the item instance's
exact pig/strider target.

An eligible target calls `boost`. If that mount is not already boosting, it first commits
`nextInt(841) + 140` as the synchronized total, sets boosting, and resets its local elapsed clock.
Only after this commit does the used stack attempt damage `7` or `1`. Durability enchantments may
reduce the processed change; infinite-material players take no damage. A nonzero processed change
triggers item-durability-changed before writing the new damage.

When the resulting damage breaks the stack, the original item is removed, the hand-slot equipment-
break event and `item_broken` statistic occur, and the now-empty stack's component patch is copied
onto one fishing rod. If that result is damageable its damage is then forced to zero. Thus custom
name, lore, enchantments and other patch entries can survive the identity conversion, while damage
does not. The server returns success with that same damaged stick or transformed fishing rod as the
held-item result. A successful boost does not award `item_used`.

If passenger/controller/type admission fails, or `boost` rejects because a prior boost is still
active, the server instead awards this stick's `item_used` statistic and returns pass. It consumes
no durability and no vehicle RNG in that path. This makes the apparent failed/repeated use the
stat-awarding branch, while the successful boost is not.

#### Controller, boost and temptation joins

Pig and strider controller selection is exact identity rather than a live tag: saddle, first
passenger/player and either-hand possession of the matching stick select the controller. While
selected, ridden input is constant forward `(0,0,1)`, yaw follows player yaw, pitch follows half
player pitch, and the mount ticks its boost clock. Pig speed is movement speed times `0.225`; strider
uses `0.55` when warm and `0.35` when suffocating. Both multiply by
`1 + 1.15 * sin(pi * elapsed / total)` while boosting.

Dropping or switching away from the exact stick removes the controlling passenger result and stops
the ridden boost-clock callback. The in-memory boost remains active at its current elapsed value;
reacquiring the correct stick resumes it, and another use remains rejected until the callback
finishes. Dismounting likewise pauses rather than clears a process-continuous boost. Entity
reload/recreation instead cancels it because neither the active flag, elapsed clock nor synchronized
total is saved.

Pig installs two priority-four speed-`1.2` temptation goals: live `pig_food` and exact
`carrot_on_a_stick`. Strider installs one priority-three speed-`1.4` goal over live
`strider_tempt_items`, whose locked value expands `strider_food` and adds
`warped_fungus_on_a_stick`. The sticks themselves are not pig/strider food, so they do not breed or
feed the mount. A saddled empty mount can therefore use its ordinary nonsecondary interaction to
mount the player while the matching stick is held.

#### Crafting and progression

Each shaped recipe is a trimmed two-by-two diagonal: fishing rod above-left and carrot or warped
fungus below-right, with the horizontal mirror also admitted by the generic matcher. Any accepted
rod component/damage patch is consumed and the output is the default corresponding stick; crafting
does not preserve that patch. Direct recipe unlock or possession of the exact food ingredient
unlocks its recipe.

The Nether `ride_strider` advancement listens for item-durability-changed on exact warped fungus on
a stick while the player vehicle is a strider. It therefore completes after a successful
noncreative boost only when durability processing returns a nonzero change; an enchantment that
prevents the sole point suppresses that trigger. The child Overworld-lava-distance advancement uses
this item only as its display icon and retains its independent ride-distance owner. No bundled loot
table or trade directly emits either scoped item.

#### Client projection and creative inventory

Each item selects one direct model using `item/handheld_rod` and its like-named texture. There is no
cast/pulled/boosting model override. Generic damage state alone controls the durability bar; a
break projects the hand-slot break event before the replacement fishing rod converges. The
synchronized boost total starts the client's local boost curve, while authoritative mount motion
and corrections retain their entity/protocol owners.

Tools & Utilities orders carrot on a stick then warped fungus on a stick immediately after all
sixteen harnesses and before oak boat. Both use ordinary parent-and-search visibility.

**Branches and aborts:**

Generic spectator/cooldown gate; client/server; passenger/no passenger; saddled/unsaddled; first/
other passenger; correct/wrong/offhand controller; pig/strider/other steerable; idle/active boost;
normal/infinite materials; zero/nonzero enchantment-processed damage; intact/broken stack; held/
removed/dismounted/reloaded controller; exact/live-tag temptation; base/mirrored/missing recipe.

**Constants and randomness:**

Item IDs `887/888`; maximum stack `1`; maximum damage `25/100`; boost damage `7/1`; duration
`nextInt(841)+140`, hence `140..980` inclusive from the mount RNG; boost amplitude `1.15` and
`pi`; pig/strider steering factors `0.225`, `0.55/0.35`; fixed forward input `(0,0,1)`;
half-pitch factor `0.5`; temptation priority/speed pig `4/1.2`, strider `3/1.4`. Rejected boost
consumes no RNG; durability enchantments retain their owning RNG stream.

**Side effects:**

Mount synchronized boost total and transient boost clock; stack damage or patched fishing-rod
replacement; durability criterion, break event and statistics; controller-selected rotation and
motion; temptation target/navigation; recipe unlocks; durability bar, item model and creative-tab
projection.

**Gates:**

Spectator/cooldown; server side; player passenger and exact controlled vehicle; saddle, first
passenger and exact either-hand item; `ItemSteerable`; inactive boost; durability/infinite-material
and enchantment result; recipe/advancement/tag snapshots; client resources.

**State read/written:**

Reads used-hand stack/components, both held items, passenger/vehicle/saddle/type state, boost flag,
mount RNG, player material mode, enchantments, recipes/advancements/tags and client assets. Writes
boost total/flag/elapsed state, stack damage or identity, break/stat/progression state, mount
rotation/motion and client projection.

**Failure behavior:**

Client use always passes without prediction. Server controller or active-boost rejection awards
`item_used` and passes without damage/RNG. Successful boost commits before durability, so creative
or fully prevented damage does not roll back the boost. A break returns a patched zero-damage
fishing rod rather than an empty hand. Invalid recipes retain inputs; reload-removing the strider
tempt membership stops later temptation but not exact mounted control.

**Persistence boundary:**

Stick/fishing-rod identity, count, damage and component patch persist through generic stack
encoding. Pig/strider saddle, passengers and ordinary entity fields persist separately. Active
boost flag, elapsed clock and total do not save: process-continuous loss of the controller merely
pauses them, while entity reload resets them.

Data reload changes later durability-enchantable and strider-tempt membership, recipes and
advancements without rewriting existing stacks, enchantments, mounts or progress. Pig's exact
carrot-stick lure and both controller/type/damage mappings stay code-built. Resource reload replaces
item models, textures and language.

**Boundary cases and quirks:**

All client uses pass. A rejected server use awards `item_used`, while a successful boost does not.
Boost commits before damage and survives creative/no-damage processing. Fully prevented warped-stick
damage can suppress `ride_strider` despite starting a boost. Broken sticks preserve their component
patch on a fishing rod but force damage zero. Removing the controller pauses an active boost; only
entity reload cancels it. Reload may disable warped-stick temptation without disabling exact
strider steering.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.FoodOnAStickItem`;
`net.minecraft.world.item.ItemStack#hurtAndConvertOnBreak`;
`net.minecraft.world.entity.ItemSteerable`;
`net.minecraft.world.entity.ItemBasedSteering`;
`net.minecraft.world.entity.animal.pig.Pig`;
`net.minecraft.world.entity.monster.Strider`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{carrot_on_a_stick,warped_fungus_on_a_stick}.json`;
`data/minecraft/tags/item/{enchantable/durability,strider_food,strider_tempt_items,pig_food}.json`;
`data/minecraft/recipe/{carrot_on_a_stick,warped_fungus_on_a_stick}.json`;
`data/minecraft/advancement/recipes/transportation/{carrot_on_a_stick,warped_fungus_on_a_stick}.json`;
`data/minecraft/advancement/nether/{ride_strider,ride_strider_in_overworld_lava}.json`;
`assets/minecraft/{items,models/item,textures/item}/{carrot_on_a_stick,warped_fungus_on_a_stick}.*`;
`PLY-INTERACT-001`; `PLY-INPUT-001`; `PLY-MOVE-SPECIAL-001`; `ITM-USE-001`;
`ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-ADVANCEMENT-001`; `ITM-ENCHANT-001`;
`MOB-AI-001`; `CLI-EFFECT-001`; `EXP-ITM-023`.

**Test vectors:**

Use both items on client/server across spectator/cooldown, saddle/passenger/order, hand, target,
idle/active boost, creative, every damage threshold and deterministic enchantment result; assert
boost-before-damage, RNG, result, statistic, criterion, break event and fishing-rod patch order.
Remove/reacquire each controller, dismount/remount and reload mid-boost. Exercise exact versus
live-tag temptation, both recipe orientations/offsets with patched rods, unlock/Nether advancement
conditions, persistence/reload and every damage/model/tab projection.

**Limits:**

This leaf does not duplicate generic packet use admission, durability-enchantment arithmetic,
recipe allocation, advancement evaluation, animal interaction, temptation navigation, ridden
movement integration, entity persistence framing or client reconciliation. Those retain the cited
owners; this rule fixes the two item identities, their exact mount joins and observable ordering.
