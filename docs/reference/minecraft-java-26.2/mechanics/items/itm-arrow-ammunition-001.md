# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ARROW-AMMUNITION-001` — Arrow stacks select pickup identity before tipped and spectral impact state diverges

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `ITM-001`, `ITM-003`, `ITM-004`,
`ITM-005`, `ITM-006`, `ITM-007`, `ITM-DISPENSER-001`, `ITM-RECIPE-001`,
`ITM-CRAFT-001`, `ITM-LOOT-001`, `ITM-ENCHANT-001`, `ITM-ADVANCEMENT-001`,
`ENT-001`, `ENT-004`, `ENT-005`, `MOB-001`, `MOB-004`, `CLI-001`, `CLI-006`,
`CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration/components, projectile-weapon and dispenser bytecode,
arrow/spectral entity bytecode, recipes, loot/trade/progression data and client assets determine
all three stack identities, their projectile and pickup mapping, potion/glow divergence,
acquisition, persistence and projection. Generic projectile motion/collision/damage remains owned
by `ENT-PROJECTILE-001`.

**Applies when:**

An `arrow`, `spectral_arrow` or `tipped_arrow` stack is selected as ammunition, copied into a
projectile, shot by a player or mob, dispensed, hits or rests in a block, is picked up, crafted,
looted, bartered, traded, persisted, reloaded, described or rendered.

**Authoritative state:**

| stack | raw item ID | registered components beyond common max-64 defaults | projectile | default pickup |
|---|---:|---|---|---|
| `arrow` | `923` | none | entity `arrow` protocol ID `6` | `arrow` |
| `spectral_arrow` | `1322` | none | entity `spectral_arrow` protocol ID `123`, glow duration `200` | `spectral_arrow` |
| `tipped_arrow` | `1323` | `potion_contents={}`, `potion_duration_scale=0.125` | entity `arrow` protocol ID `6` carrying a count-one source copy | carried source stack |

All are common, nondamageable and direct members of item tag `#minecraft:arrows`; that tag has
exactly these three entries. Entity tag `#minecraft:arrows` has exactly `arrow` and
`spectral_arrow`; tipped ammunition deliberately has no third entity identity.

The registered tipped prototype has empty potion contents, but
`TippedArrowItem#getDefaultInstance` overrides the general item API default by installing Poison.
A raw stack constructed from the registered item retains empty contents. Never substitute one
meaning for the other.

**Transition and ordering:**

A projectile weapon asks for a supported projectile. Player selection checks a supported offhand
stack, then supported main-hand stack, then inventory slots in ascending order. With no stack,
infinite-material players receive a new ordinary arrow and other players receive empty. Bows'
held/all predicates are arrows only; crossbows additionally admit a held firework rocket but their
inventory fallback predicate remains arrows.

One draw begins with enchantment-selected projectile count, base `1`. The first projectile uses
the selected stack; each extra uses a copy. Unless the player has infinite materials, first-ammo
cost is the weapon's `ammo_use` result and a positive cost splits that many from the source.
Cost `0`, every extra and every infinite-material shot instead returns a count-one copy with
`intangible_projectile`; the source is not shrunk. A cost larger than count returns empty.
Infinity changes ammo use to zero only for item identity `arrow`: tipped and spectral arrows are
consumed in survival. Creative and multishot preserve any identity and component patch.

`ArrowItem#createArrow` and inherited tipped creation copy one source item into a new `Arrow`;
`SpectralArrowItem` creates `SpectralArrow`. The projectile first copies that stack as its pickup
stack and applies generic projectile components, then removes `intangible_projectile` from the
caller-supplied stack. Presence of the removed component sets pickup `CREATIVE_ONLY`; its earlier
stored copy still contains the component. The fired weapon is copied separately and piercing
state is resolved by the generic weapon/enchantment hook.

Assigning a player owner upgrades pickup `DISALLOWED` to `ALLOWED` but preserves
`CREATIVE_ONLY`. An ominous-item-spawner owner forces `DISALLOWED`; every other owner preserves
the current mode. Thus ordinary player ammunition is recoverable, creative/multishot copies are
creative-only, and ordinary mob ammunition is disallowed.

**Dispenser and pickup transaction:**

Arrow-item dispenser creation uses one count-one copy, position
`dispenser output at distance 0.7 + (0,0.1,0)`, facing direction, power `1.1` and uncertainty `6`.
It spawns through the generic enchantment hook, shrinks the slot by one and emits level event
`1002`. Both arrow item subclasses explicitly force pickup `ALLOWED`, overriding constructor
ownership/intangible results; all copied source components remain in the stored pickup stack.

Player contact is server-only and requires `(inGround || noPhysics)` plus `shakeTime<=0`.
`ALLOWED` adds a copy of the pickup stack to inventory, `CREATIVE_ONLY` succeeds only for a player
with infinite materials, and `DISALLOWED` fails. Success calls take and discards the entity.
If a flying arrow slows below squared speed `1e-7` after an unsuccessful hit/deflection, the server
spawns its pickup item only for `ALLOWED`, then discards it.

**Potion-bearing arrow behavior:**

Every ordinary `Arrow`, not only one created from `tipped_arrow`, reads potion contents and
duration scale from its stored pickup stack. Missing values mean empty and scale `1`. A
component-patched ordinary arrow therefore applies full-duration potion effects; registered tipped
ammunition applies one eighth duration.

On a successful living hit, the arrow first completes generic damage/post-hit handling, then
iterates base-potion effects followed by custom effects. A finite positive duration becomes
`max(1,floor(duration*scale))`; infinite/zero duration is unchanged. Each clone is offered to the
target with the projectile's effect source, and rejection does not roll back the hit.
`SpectralArrow` does not execute this potion path.

An `Arrow` synchronizes color `-1` for exactly empty contents. Otherwise it uses explicit custom
color or the amplifier-weighted visible-effect color, falling back to decimal `-13083194`.
While flying, a colored arrow emits two `ENTITY_EFFECT` particles per client tick at independent
random offsets; while grounded it emits one whenever `inGroundTime mod 5 == 0`. Rendering selects
the tipped entity texture only when color is strictly positive, otherwise the ordinary texture.

After the superclass increments grounded time, a server `Arrow` whose potion contents are not
exactly empty converts at `inGroundTime>=600`: it broadcasts entity event `0`, replaces its pickup
stack with a new default ordinary arrow and synchronizes color `-1`. This erases the complete
carried component patch. The client event emits 20 effect-color puff particles using the old color.
The rule applies to potion-patched ordinary arrows as well as tipped-origin arrows; an empty tipped
arrow never converts.

**Spectral behavior:**

A new `SpectralArrow` has duration `200`. Each airborne client tick emits one color-`-1`,
alpha-`1` `EFFECT` particle at exact entity position with zero velocity; grounded spectral arrows
emit none. After a successful living hit and generic post-hit handling it offers Glowing,
duration equal to the entity field, amplifier `0`, from the projectile effect source. Rejection is
ignored. Its renderer always selects `textures/entity/projectiles/arrow_spectral.png`.

Spectral save data adds integer `Duration`, defaulting to `200` when absent. Pickup item components
cannot change this field, although entity NBT can. `Arrow` instead derives all potion state from
its persisted pickup stack.

**Crafting and unlocks:**

- The shaped vertical flint/stick/feather recipe yields four default arrows.
- The plus-shaped recipe places one arrow between four glowstone dust and yields two default
  spectral arrows. Component patches on the center arrow satisfy identity matching but do not
  transfer.
- `crafting_imbue` requires all nine slots nonempty: a center lingering potion and eight exact
  arrow identities. It yields eight registered-default tipped arrows, then copies only the
  center's `potion_contents`; absence removes that component. It never copies other center or
  arrow components and retains tipped scale `0.125`.

Recipe unlocks use OR: already has recipe, or feather/flint for arrow; already has recipe, or
glowstone dust for spectral; already has recipe, or lingering potion for tipped.

**Loot, trade and mob acquisition:**

Generic pool evaluation belongs to `ITM-LOOT-001`; these are the complete standard-pack direct
arrow records and their identity-specific functions:

- Ominous trial-spawner item drops are five equal one-roll entries: plain arrow, Poison-patched
  arrow, Strong Slowness-patched arrow, fire charge `1..3`, wind charge `1..3`. Each arrow entry
  has count one and the patched ordinary arrows retain default scale `1`.
- Bastion bridge has spectral weight `1/13`, count `10..28`, across `1..2` pool rolls and arrow
  weight `1/5`, count `5..17`, across `2..4`; treasure has spectral `1/9`, count `12..25`, across
  `3..4`; other has spectral `10/89`, count `10..22`, in one roll and arrow `2/13`, count `5..17`,
  across `3..4`; hoglin stable has arrow `1/14`, count `5..17`, across `3..4`.
- Jungle-temple dispenser has only arrow, count `2..7`, across `1..2` rolls. Pillager-outpost
  chest has arrow `4/22`, count `2..7`, across `2..3`; piglin barter has spectral `40/469`,
  count `6..12`, in one roll.
- Trial-chamber dispenser/corridor dispenser/entrance/corridor-pot records respectively provide
  arrow `4/29` count `4..8`; guaranteed arrow `4..8`; arrow `10/36` count `5..10` across `2..3`;
  and arrow `100/351` count `2..8`. Supply, across `3..5` rolls, has arrow `2/18` count `4..14`,
  Poison tipped `1/18` count `4..8`, and Slowness tipped `1/18` count `4..8`.
- Each normal reward-common call has arrow `4/25` count `2..8` and Poison tipped `4/25`
  count `2..8`; each ominous reward-common call has Strong Slowness tipped `3/15`
  count `4..12`. Vault table call counts/ejection order remain with `BLK-VAULT-001`.
- Village fletcher chest has arrow `2/23`, count `1..3`, across `1..5` rolls.

Skeleton, stray, bogged and parched base loot has arrow count `0..2` plus the generic uniform
`0..1` looting increase per level. Player-killed bogged, stray and parched separately offer a
count-`0..1`, looting-increased-but-limit-`1` tipped arrow with Poison, Slowness and Weakness,
respectively.

Hero fletcher gifts choose one of total weight `39`: arrow has weight `26`, count one, and each of
13 tipped potion entries has weight one and count `0..1`. The potion set is Swiftness, Slowness,
Strength, Healing, Harming, Leaping, Regeneration, Fire Resistance, Water Breathing, Invisibility,
Night Vision, Weakness and Poison.

Default fletcher trade sets select two distinct records from three at levels one and five. The
level-one arrow record costs one emerald and gives 16 default arrows, max uses `12`, multiplier
`0.05`. The level-five record costs two emeralds plus five identity-matching arrows and gives five
tipped arrows with one uniform potion from the 41-entry `#minecraft:tradeable` potion tag, max uses
`12`, XP `30`, multiplier `0.05`. Its random-potion function updates the output's existing potion
contents. The standard baseline excludes the optional `trade_rebalance` pack.

Mobs select supported offhand then main-hand ammunition and otherwise construct an ordinary arrow;
they do not consume the source. Skeleton-family launch uses the selected `ArrowItem` subtype.
Bogged, stray and parched add Poison `100`, Slowness `600` and Weakness `600` only when the result
is an `Arrow`; selected spectral ammunition therefore receives no subtype potion.

**Progression and enchantment joins:**

`shoot_arrow` requires successful projectile-tag damage whose direct entity is in entity
`#minecraft:arrows`; all three ammunition identities qualify because tipped uses entity `arrow`.
`sniper_duel` is a generic projectile skeleton kill at horizontal distance at least `50`, not an
arrow-only predicate. Power and Punch target entity `#minecraft:arrows`; Flame, Piercing and
Multishot join at generic weapon/projectile hooks. Infinity's ordinary-arrow-only ammo predicate is
the identity exception specified above.

**Persistence boundary:**

Abstract arrow save data includes life, in-ground block state, shake, in-ground flag, pickup enum,
damage, critical flag, pierce level, hit sound, full pickup `item` stack and nullable `weapon`.
Load defaults missing pickup item to the subtype's default. Stack persistence separately retains
identity, count and component patch; active projectiles resume from entity state.

Arrow and spectral item models are direct generated one-texture models. Tipped item projection has
a potion-tinted head layer over the base layer; tint uses potion custom/effect/fallback color.
Tipped name uses custom potion name, then registered potion name, then `.effect.empty`; its potion
tooltip uses component scale `0.125` and client tick rate.

Combat-tab order is ordinary arrow, spectral arrow, then one tipped arrow for every enabled potion
registry holder in registry order. The locked registry contributes exactly 46 tipped entries.
The entity textures are ordinary, color-positive tipped and spectral as specified above.

**Branches and aborts:**

Source hand/slot and supported predicate; no-ammo ability; projectile count/cost/count; item
subtype; intangible component; owner kind/current pickup; dispenser versus weapon; logical side;
ground/noPhysics/shake/inventory/ability; target living and effect acceptance; contents exact
empty/color; duration/scale; grounded age; recipe shape/components; pool/condition/weight/count;
trade-set selection/potion; mob-held subtype; renderer context.

**Constants and randomness:**

Stack size `64`; spectral glow `200`; tipped scale `0.125`; conversion age `600`; potion particles
`2` flying or `1/5` grounded; conversion event particles `20`; dispenser distance/power/uncertainty
`0.7/1.1/6`; slow-discard squared speed `1e-7`. Craft outputs `4/2/8`; creative tipped entries
`46`; tradeable potions `41`. Loot weights, rolls and counts are exact above.

**Side effects:**

Ammo count/inventory; projectile entity, owner, pickup and carried/weapon stacks; damage/effects;
pickup insertion/discard/item spawn; entity data/events/particles/sounds; recipes/loot/trades/
advancements; saved data; names/tooltips/models/tab entries.

**Gates:**

Supported item tag or held firework exception; weapon release and enchantment hooks; source
count/cost/ability; spawn; projectile hit; living target; effect admission; grounded age; pickup
mode/contact/inventory; recipe and unlock predicates; loot/trade conditions; client entity/item
context.

**State read/written:**

Reads the source and weapon stacks, player hands/inventory/abilities, enchantment results,
projectile owner/pickup/motion/ground state, carried potion contents/scale, spectral duration,
target effect admission, recipes, loot/trade/progression data and client registries/resources.
Writes source counts/inventory, projectile carried/weapon stacks and entity data, target effects,
pickup/discard/item entities, saved fields and client presentation events.

**Failure behavior:**

Unsupported or absent ammunition yields no projectile unless infinite materials supplies an
ordinary arrow. Excess ammo cost yields empty. Failed spawn, damage or effect admission is never
backfilled by a subtype-specific retry. Pickup gate or inventory-add failure leaves the projectile.
Wrong recipe shapes do not assemble; failed loot/trade/advancement conditions emit nothing.
Missing saved fields use the stated subtype defaults, and missing resources follow generic client
resource failure rather than an arrow-specific fallback.

**Boundary cases and quirks:**

Tipped ammunition is an ordinary arrow entity. Potion-patched ordinary arrows apply full duration
and also convert after 600 grounded ticks. Empty tipped arrows neither color nor convert.
`getDefaultInstance` Poison differs from the registered empty prototype. Color-positive, not stack
identity, selects the tipped entity texture. Multishot's stored pickup retains the intangible
component even though the transient source copy loses it. Dispensers make all arrow subtypes
survival-pickable. Player assignment preserves creative-only pickup. Spectral duration is entity
state, not a stack component. Imbuing copies only potion contents.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.ArrowItem`;
`net.minecraft.world.item.SpectralArrowItem`; `net.minecraft.world.item.TippedArrowItem`;
`net.minecraft.world.item.ProjectileWeaponItem`; `net.minecraft.world.item.BowItem`;
`net.minecraft.world.item.CrossbowItem`; `net.minecraft.world.entity.player.Player#getProjectile`;
`net.minecraft.world.entity.projectile.arrow.AbstractArrow`;
`net.minecraft.world.entity.projectile.arrow.Arrow`;
`net.minecraft.world.entity.projectile.arrow.SpectralArrow`;
`net.minecraft.core.dispenser.ProjectileDispenseBehavior`;
`net.minecraft.world.item.crafting.TippedArrowRecipe`;
`net.minecraft.world.item.CreativeModeTabs`;
`net.minecraft.client.renderer.entity.TippableArrowRenderer`;
`net.minecraft.client.renderer.entity.SpectralArrowRenderer`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{arrow,spectral_arrow,tipped_arrow}.json`;
`data/minecraft/tags/{item,entity_type}/arrows.json`;
`data/minecraft/recipe/{arrow,spectral_arrow,tipped_arrow}.json`;
`data/minecraft/advancement/adventure/{shoot_arrow,sniper_duel}.json`;
`data/minecraft/loot_table/{chests,dispensers,entities,gameplay,spawners}/**`;
`data/minecraft/{trade_set,tags/villager_trade,villager_trade}/fletcher/**`;
`data/minecraft/tags/potion/tradeable.json`;
`assets/minecraft/{items,models/item,textures/item}/{arrow,spectral_arrow,tipped_arrow}*`;
`assets/minecraft/textures/entity/projectiles/{arrow,arrow_tipped,arrow_spectral}.png`;
`ITM-DISPENSER-001`; `ITM-RECIPE-001`; `ITM-CRAFT-001`; `ITM-LOOT-001`;
`ITM-ENCHANT-001`; `ITM-ADVANCEMENT-001`; `ENT-PROJECTILE-001`;
`ENT-EFFECT-001`; `MOB-AI-001`; `BLK-VAULT-001`; `CLI-EFFECT-001`;
`EXP-ITM-032`.

**Test vectors:**

Cross all three identities and ordinary/tipped component patches through both hands, inventory
fallback, survival/Infinity/creative and one/many projectile counts. Assert source shrink,
intangible removal/stored patch, owner pickup transitions and dispenser override. Hit living and
nonliving targets with empty/base/custom/infinite potion effects at scales `0/0.125/1`, accepted
and rejected effects; tick flying/grounded through `599/600`, then pick up with all modes and
inventory outcomes. Exercise spectral hit/save duration. Exhaust recipes, every direct loot/trade/
mob source and progression predicate. Persist stacks/projectiles, reload data/resources, and
capture every name, tooltip, item/entity model and Combat-tab entry.

**Limits:**

This leaf does not duplicate generic bow/crossbow charge, projectile motion/collision/damage,
effect merging, dispenser scheduling, loot evaluation, villager economics, recipe execution,
advancement dispatch, entity/stack codecs or renderer resource loading. Those remain with the
cited owners; this rule fixes the three ammunition identities and every exact subtype join.
