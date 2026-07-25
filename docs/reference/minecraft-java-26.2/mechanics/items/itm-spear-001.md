# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-SPEAR-001` — Spears split a minimum-charge piercing stab from a speed-gated held kinetic charge

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ENT-001`, `ENT-002`,
`ENT-005`, `MOB-001`, `MOB-004`, `MOB-005`, `CLI-001`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registrations, component reports, packet ingress, shared piercing and
kinetic components, entity combat, mob AI, tags, enchantment, recipes, loot and client assets close
all seven spear identities without an item subclass.

**Applies when:**

A wooden, stone, copper, iron, golden, diamond or netherite spear is stabbed, held-used, moved into
an entity, wielded by a mob, damaged, repaired, enchanted, crafted, smelted, looted, persisted,
reloaded or projected.

**Authoritative state:**

All seven are common, maximum-stack-one ordinary `Item` instances with damage zero, break sound
`entity.item.break`, spear damage type, a weapon component whose configured per-attack damage is
`1`, full `1.0` minimum attack charge, common attack range and tier-selected components. The spear
stab transaction invokes only the pre-post `hurtEnemy` stage, however, so that configured point is
not applied by either spear scan.

| item | ID | durability | enchantability | attack damage modifier | stab ticks | held delay | kinetic multiplier |
|---|---:|---:|---:|---:|---:|---:|---:|
| `wooden_spear` | `1326` | `59` | `15` | `+0` | `13` | `15` | `0.7` |
| `stone_spear` | `1327` | `131` | `5` | `+1` | `15` | `14` | `0.82` |
| `copper_spear` | `1328` | `190` | `13` | `+1` | `17` | `13` | `0.82` |
| `iron_spear` | `1329` | `250` | `14` | `+2` | `19` | `12` | `0.95` |
| `golden_spear` | `1330` | `32` | `22` | `+0` | `19` | `14` | `0.7` |
| `diamond_spear` | `1331` | `1561` | `10` | `+3` | `21` | `10` | `1.075` |
| `netherite_spear` | `1332` | `2031` | `15` | `+4` | `23` | `8` | `1.2` |

Attack-speed modifiers are respectively `-2.4615384340286255`,
`-2.666666626930237`, `-2.8235294818878174`, `-2.9473683834075928`,
`-2.9473683834075928`, `-3.0476189851760864` and `-3.13043475151062`.
All use minimum reach `2.0`, maximum reach `4.5`, creative maximum `6.5`, hitbox margin `0.125`
and mob factor `0.5`. Use permits sprinting, emits no interaction vibration and applies speed
multiplier `1`. Netherite alone resists fire damage as an item entity.

**Transition and ordering:**

#### Minimum-charge piercing stab

The client sends the dedicated `STAB` player action instead of an entity-specific attack. The
server requires a loaded client, nonspectator player and `cannotAttackWithItem(stack, 5)` to pass.
With minimum charge `1.0`, admission is exactly
`(attackStrengthTicker + 5) / currentAttackStrengthDelay >= 1`, so the packet has a five-tick
forgiveness margin. The server takes
the main-hand piercing component and ray-collects every admissible entity along the spear attack
range, but a collider hit yields an empty entity list. Targets must be alive, not piercing-immune,
projectile-hittable unless they are an interaction entity, harmable under player-versus-player
rules, and outside the attacker's vehicle graph.

Every collected target receives the current attack-damage attribute through spear damage and
enchantment modification, ordinary hurt admission, fixed `0.4` extra knockback plus the attacker's
enchantment knockback, and no forced dismount. A living target invokes `hurtEnemy`; for a player
attacker this awards the spear's `item_used` statistic once per living target even if damage was
rejected. It does not invoke `postHurtEnemy`, so the weapon component's configured point does not
damage the spear. Successful damage runs post-attack enchantment effects. Any knockback or accepted
damage sets last-hurt target and plays the attack sound.

After the entire ordered list, the attacker runs ordinary attack bookkeeping and all
`post_piercing_attack` enchantment effects. Lunge levels `1..3`, when the attacker is unmounted,
not fall-flying, not in water, and either nonplayer, creative, or at food level at least `7`, then
damage the held item by one, add exhaustion `4/8/12`, apply horizontal look-direction impulse
`0.458/0.916/1.374`, and select one of three lunge sounds. This post effect runs even when the ray
found no target. A hit sound plays once if any target was affected; the attack sound and a
main-hand nonbroadcast swing always follow.

The ordinary entity-attack packet excludes piercing weapons, so clicking a particular entity
cannot additionally invoke the single-target player attack path.

#### Held kinetic charge

Generic held use starts the selected hand for `72000` ticks, publishes the start-use flag and game
event, allocates a fresh contact-time map on the server, plays the tier use sound, and consumes the
interaction. Each later server use tick waits for the tier delay, then scans the same block-clipped
attack range.

Motion is known velocity times `20`. A player uses their own motion even while mounted; a mounted
nonplayer instead uses root-vehicle motion. Both attacker and target motion are projected onto the
attacker's look vector. Relative speed is `max(0, attacker projection - target projection)`.
Player thresholds use factor `1`; nonplayer thresholds are multiplied by `0.2`.

Each new target is entered into the contact map before condition evaluation. The same target is
therefore suppressed for the next `10` ticks even if no threshold succeeds. Ender Dragon parts
converge to their parent before this check. Conditions are inclusive in elapsed duration and speed:

| tier | damage through held tick | knockback through | dismount through | dismount attacker speed |
|---|---:|---:|---:|---:|
| wood | `300` | `200` | `100` | `14` |
| stone | `275` | `180` | `90` | `13` |
| copper | `250` | `165` | `80` | `12` |
| iron | `225` | `135` | `50` | `11` |
| gold | `275` | `170` | `70` | `13` |
| diamond | `200` | `130` | `60` | `10` |
| netherite | `175` | `110` | `50` | `9` |

Damage additionally requires relative speed at least `4.6`; knockback requires attacker speed at
least `5.1`. Damage is base attack-damage attribute plus
`floor(relativeSpeed * tierMultiplier)`. Dismount merely requires the tier attacker-speed threshold.
The shared stab transaction applies exactly the conditions that passed; it can therefore knock
back or dismount without dealing damage. A living target still invokes `hurtEnemy` and awards a
player attacker `item_used` whenever at least one condition passed, but again skips the
post-attack durability stage.

If any target was affected, the server broadcasts entity event `2` and evaluates
`spear_mobs` against the number of distinct living entities retained in this held-use contact map,
including earlier threshold-failing contacts. Client feedback accepts event `2` only when more
than `10` ticks have elapsed since its previous feedback, then plays the local tier hit sound and
animates the recoil. Releasing, changing the used item, swapping hands, or stopping use clears the
contact map and emits the finish-use event.

#### Mob selection and acquisition

Zombie and zombified-piglin goal AI selects held kinetic use only with a live target, a main-hand
kinetic component and no current use. Both approach within `10`, charge at speed `1`, reposition at
speed `1`, turn within `30` degrees and retreat/re-engage around radii `6..7` and `9..11`, each
extended by `2` while mounted. Engagement lasts `delay + damage-window` ticks. Piglin brain AI uses
the corresponding approach, charge and retreat states. Root-vehicle `chargeSpeedModifier` scales
mob navigation, while the nonplayer threshold factor remains `0.2`.

Ordinary zombies first pass their inherited equipment population, then on a `1%` non-Hard or `5%`
Hard weapon roll choose one of six results: iron sword at index zero, iron spear at index one, and
iron shovel for the other four. A naturally spawned zombie horse replaces its created zombie
jockey's main hand with iron spear. The one-per-group natural husk camel-jockey attempt has a
`10%` branch that equips the husk with iron spear before creating its mount and parched passenger.
Zombified piglins choose golden spear with `1/20`, otherwise golden sword. Adult piglin spawn
weapons choose crossbow with probability `1/2`; the other half chooses golden spear with `1/10`,
otherwise golden sword.

Locked chest loot can select stone spear in both underwater-ruin tables, copper or iron spear in
village weaponsmith, iron spear in buried treasure, enchanted diamond spear in End city, and
damaged/enchant-randomly or plain diamond spear in bastion treasure. No locked direct chest table
emits wood, gold or netherite spear.

#### Crafting, repair, recycling and tags

Wood through diamond each use the same diagonal full-grid shaped recipe: material in top-right and
sticks at center and bottom-left; the horizontal mirror is also accepted. Inputs use the tier
`*_tool_materials` tag and outputs are default stacks. Wood unlocks from exact stick; the other
five unlock from their material tag. Netherite uses the ordinary transform recipe from netherite
upgrade template, diamond spear and `netherite_tool_materials`, preserving the base's eligible
component patch through the generic smithing owner; its unlock watches the addition tag.

Repair uses the same tier material tag. Copper, iron and golden spears are also inputs to their
respective nugget smelting and blasting recipes. Wooden spear is code-built furnace fuel for
`200` ticks. The `spears` tag nests into durability-, melee-weapon- and lunge-enchantable tags.
Golden spear additionally belongs directly to `piglin_loved` and `piglin_preferred_weapons`,
selecting admiration/protection and adult weapon preference through their existing mob owners.

#### Client projection and creative inventory

Every item selects its flat like-named generated texture in GUI, ground, fixed and shelf contexts;
all other contexts select its like-named in-hand texture through the shared `spear_in_hand` display
transforms. Item swap animation scale is `1.95`. The `STAB` swing component drives the tier
`13..23`-tick first/third-person thrust. Held use drives the staged raise, sway and lower animation,
uses common forward offset `0.38`, and overlays the event-2 recoil from ticks `1..10`. Generic
damage state controls the durability bar.

Combat orders all seven wood-to-netherite spears immediately after the seven swords and before the
seven axes. Each has ordinary parent-and-search visibility.

**Branches and aborts:**

Loaded/unloaded client; spectator/survival; exact five-tick-forgiven attack charge; main/offhand held use;
entity/collider ray result; alive/dead, immune, projectile-hittable, interaction, PvP and
same-vehicle targets; zero/one/many targets; accepted/rejected damage; enchantment knockback and
post effects; mounted/water/fall-flying/food Lunge gates; use delay; player/nonplayer/root-vehicle
motion; threshold duration/speed edges; new/recent contact; dragon part/parent; damage/knockback/
dismount combinations; per-living-target statistics; Lunge intact/broken durability; every mob equipment roll; recipe mirror,
material/unlock/smithing/recycling/loot branch; reload and render context.

**Constants and randomness:**

Item IDs `1326..1332`; maximum stack `1`; attack charge `1.0`; reach `2.0..4.5`, creative maximum
`6.5`, margin `0.125`, mob factor `0.5`; held duration `72000`; contact cooldown `10`; feedback
strictly more than `10`; velocity scale `20`; nonplayer factor `0.2`; forward animation `0.38`;
damage relative threshold `4.6`; knockback speed `5.1`; base knockback `0.4`; tier tables above.
Lunge level/sound selection and mob/chest/weapon enchantment draws retain their owning RNG streams.

**Side effects:**

Damage, knockback, dismount, last-hurt target, per-living-target item-used statistics, Lunge item
damage/break, enchantment effects, exhaustion and impulse; held-use flags/contact map and
start/finish game events; sound, swing, local recoil,
criterion and advancement; mob navigation/equipment; recipes, unlocks, loot, furnace burn,
durability bar, item model and creative-tab projection.

**Gates:**

Packet/client-loaded, spectator and five-tick-forgiven attack charge; component presence; block-clipped range and
target predicate; server damage admission; held-use side/delay/contact/condition gates; Lunge
requirements; durability/infinite-material/enchantment processing; AI target/equipment state;
recipes, tags, enchantment, loot and advancement snapshots; client resources.

**State read/written:**

Reads main/used-hand stack and components, attack charge/attributes/enchantments, entity bounds,
motion, vehicle graph, game time, target and damage state, mob brain/goal/equipment/RNG, recipes,
tags, loot and client assets. Writes target combat/mount state, Lunge stack damage, attacker
combat/statistic and use/contact state, progression, mob navigation/equipment and client effects.

**Failure behavior:**

An attack below the five-tick-forgiven charge threshold or a spectator stab does nothing. A collider before entities makes the piercing or
kinetic entity result empty. Rejected targets are not processed. A kinetic target admitted to the
ray is remembered before thresholds, so a failed threshold prevents an immediate retry. Lunge can
consume durability and food after a targetless stab; ordinary spear hits do not consume durability
because their transaction skips `postHurtEnemy`. No successful condition means no feedback
event or criterion evaluation. Releasing use clears contacts; invalid recipes and loot branches
retain their owners' ordinary no-result behavior.

**Persistence boundary:**

Item identity, count, damage, enchantments and component patch persist through generic stack
encoding; equipped mob stacks persist through entity equipment. Attack swings, held-use flags,
contact times, hit feedback, AI engagement/retreat and navigation do not resume after reload.
Completed recipe/advancement progress persists separately.

Data reload atomically replaces spear/enchantable/material/piglin tags, Lunge, recipes, loot and
advancements. It does not rewrite existing stacks or equipment; later matching, enchanting,
repair, mob preference, crafting, smelting, loot and criteria use the new snapshot. Registration,
component defaults, packet split, mob equipment probabilities, fuel and client transforms remain
code-built; resource reload replaces item models, textures, language and sounds.

**Boundary cases and quirks:**

Piercing ignores the clicked entity packet and attacks all eligible entities before the first
collider. Its `1.0` minimum charge includes the packet gate's five-tick forgiveness. Each living
target awards player `item_used`, but neither spear scan applies the weapon component's configured
durability point because `postHurtEnemy` is skipped; Lunge can damage the item after a targetless
stab. Kinetic speed is look-projected blocks per second,
not total velocity; a fast sideways mover can fail. Players retain their own motion while mounted,
but mounted mobs inherit root-vehicle motion. Contacts are remembered before conditions and the
five-mob criterion counts retained living contacts, so a later successful hit can count earlier
threshold failures. A knockback- or dismount-only living hit still awards `item_used` without
damaging the spear. Feedback uses a strict `>10` clock. Wooden spear is fuel; netherite alone is
fire resistant.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.Item$Properties#spear`;
`net.minecraft.server.network.ServerGamePacketListenerImpl`;
`net.minecraft.world.item.component.PiercingWeapon`;
`net.minecraft.world.item.component.KineticWeapon`;
`net.minecraft.world.entity.LivingEntity#stabAttack`;
`net.minecraft.world.entity.ai.goal.SpearUseGoal`;
`net.minecraft.world.entity.ai.behavior.{SpearApproach,SpearAttack,SpearRetreat}`;
`net.minecraft.world.entity.monster.zombie.{Zombie,ZombifiedPiglin,Husk}`;
`net.minecraft.world.entity.animal.equine.ZombieHorse`;
`net.minecraft.world.entity.monster.piglin.{Piglin,PiglinAi}`;
`net.minecraft.world.level.block.entity.FuelValues`;
`net.minecraft.client.model.effects.SpearAnimations`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,damage_type,sound_event}`;
`reports/minecraft/components/item/{wooden,stone,copper,iron,golden,diamond,netherite}_spear.json`;
`data/minecraft/tags/item/{spears,enchantable/{durability,lunge,melee_weapon},piglin_loved,piglin_preferred_weapons}.json`;
`data/minecraft/enchantment/lunge.json`;
`data/minecraft/recipe/{wooden,stone,copper,iron,golden,diamond}_spear.json`;
`data/minecraft/recipe/netherite_spear_smithing.json`;
`data/minecraft/recipe/{copper,iron,gold}_nugget_from_{smelting,blasting}.json`;
`data/minecraft/advancement/{adventure/spear_many_mobs,recipes/combat/*spear*}.json`;
`data/minecraft/loot_table/chests/{underwater_ruin_small,underwater_ruin_big,buried_treasure,end_city_treasure,bastion_treasure,village/village_weaponsmith}.json`;
`data/minecraft/damage_type/spear.json`;
`assets/minecraft/{items,models/item,textures/item}/*spear*`;
`PLY-INTERACT-001`; `PLY-INPUT-001`; `ITM-USE-001`; `ITM-RECIPE-001`;
`ITM-CRAFT-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`; `ITM-ENCHANT-001`;
`ENT-DAMAGE-001`; `ENT-KNOCKBACK-001`; `MOB-AI-001`; `CLI-EFFECT-001`;
`EXP-ITM-024`.

**Test vectors:**

Stab all seven tiers at exact `(ticker+5)/delay` charge boundaries through zero/one/many admissible
and rejected entities with intervening colliders; vary PvP, vehicle, immunity, damage admission,
per-living-target statistics, absent hit durability and every
Lunge gate/result. Hold-use both hands across delay, elapsed-window and exact projected/relative
speed boundaries for players, free mobs and mounted mobs; revisit contacts at ticks `9/10/11`,
mix failed and successful contacts, dragon parts, breaks, release/swap/reload and feedback clocks.
Enumerate every mob equipment roll, recipe mirror/material/patch, repair/enchantment/smithing/
recycling/fuel/loot/unlock/criterion path, persistence/reload and every model/bar/animation/tab
context.

**Limits:**

This leaf does not duplicate generic attack charge arithmetic, ray geometry, damage reduction,
knockback/enchantment internals, durability processing, mob spawning, recipe/smithing/smelting,
loot evaluation, advancement storage, entity persistence or client reconciliation. Those retain
the cited owners; this rule fixes the seven item identities and their component-selected joins.
