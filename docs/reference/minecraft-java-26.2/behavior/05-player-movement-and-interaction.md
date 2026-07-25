# 05 — Player Movement, Collision, Targeting, and Interaction

This page describes server gameplay results. See `CLI-*` for client input, prediction, and
correction. The two cross-reference one another without merging ownership.

## `PLY-001` Input forms movement intent; the server entity owns movement truth

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed` for input shaping, automatic-jump detection, ordinary ground/air
  dynamics and packet validation; special movement modes remain open

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.player.KeyboardInput#tick()`;
`net.minecraft.client.player.LocalPlayer#aiStep()`;
`net.minecraft.client.player.LocalPlayer#tick()`; `net.minecraft.world.entity.player.Player#tick()`;
`net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`; `COM-WIKI-PLY-001`

### Applies when

A client owns a local player and the user changes directional, jump, sneak, sprint, or flight input.

### Behavior and timing

`PLY-INPUT-001` maps seven sampled movement booleans through conflict cancellation, float
normalization, posture/item modifiers, sprint/flight transitions and independently cadenced
intent/action messages. `PLY-MOVE-001` then specifies ordinary dynamics through jump, acceleration,
collision, gravity and drag. After actual movement, `PLY-AUTOJUMP-001` probes two look-ahead lines
through ordered entity/block collision AABBs and may schedule a synthetic jump for the next input
pass. `PLY-MOVE-VALIDATE-001` separately specifies coordinate/status message selection, server
collision probing and teleport convergence. OS/focus/event-to-key state belongs to
`CLI-PREDICT-001`. `PLY-SPECTATOR-CHUNKS-001` specifies the later chunk-distance reconciliation
reached from accepted movement: spectator status and its live rule change loading/simulation
sources without gating the independently maintained client chunk view.

### Boundaries and quirks

Shift-derived movement slowdown uses the previous input sample while tail pose selection uses the
current sample. Cardinal intent remains `0.98f` before travel while the square remap restores an
unmodified diagonal to unit magnitude. Auto-jump does not require a horizontal collision, uses raw
input as its slow-motion fallback, retains the last intersecting candidate rather than the highest
and rejects exactly half-block rises. Input, sprint, ability and coordinate messages have
independent change detectors and cadence. Packet loss, latency and client FPS do not change server
gameplay tick progression. More than five coordinate packets changes the anti-cheat multiplier to
one; finite vertical moved-wrongly residual is discarded by an OR-condition quirk; pending movement
may rotate but not translate the player.

### Verification

**Owners:** `PLY-INPUT-001`, `PLY-AUTOJUMP-001`, `PLY-MOVE-001`, `PLY-MOVE-VALIDATE-001`,
`PLY-SPECTATOR-CHUNKS-001`; `EXP-PLY-001`, `EXP-PLY-005`, `EXP-PLY-006`, `EXP-PLY-007`,
`EXP-PLY-008`

Input shaping, auto-jump scheduling/consumption, ordinary dynamics and coordinate
validation/convergence are all source-specified; experiments are regression probes. The client
event-to-KeyMapping boundary remains intentionally owned by `CLI-PREDICT-001`.

## `PLY-002` Collision clips displacement by axis and shape

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`;
`net.minecraft.world.entity.Entity#move(net.minecraft.world.entity.MoverType,net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.Entity#collide(net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.Entity#collideBoundingBox(net.minecraft.world.entity.Entity,net.minecraft.world.phys.Vec3,net.minecraft.world.phys.AABB,net.minecraft.world.level.Level,java.util.List)`

### Applies when

A player or other entity requests nonzero displacement outside a branch that bypasses ordinary
collision.

### Behavior and timing

`PLY-COLLISION-001` specifies swept-shape collection, Y-first/dominant-horizontal axis clipping,
ascending step-candidate selection, position recording, collision/support flags, restitution and
post-move emission/speed effects. Concrete registry content supplies collision shapes and block
properties without changing the generic transaction.

### Boundaries and quirks

Shape clipping and collision flags intentionally use different epsilons. Equal absolute X/Z selects
X before Z; step-up accepts the first ascending height that strictly improves horizontal squared
displacement rather than a globally best candidate.

### Verification

**Owners:** `PLY-COLLISION-001`, `BLK-SCULK-SENSOR-001`, `BLK-SLIME-001`, `BLK-HONEY-001`,
`BLK-NETHER-SPROUTS-001`,
`BLK-NETHER-ROOTS-001`,
`BLK-FLOWER-POT-001`,
`BLK-COPPER-FULL-001`,
`BLK-SAPLING-001`,
`BLK-BAMBOO-001`,
`BLK-STEM-CROP-001`,
`BLK-TORCHFLOWER-CROP-001`,
`BLK-PITCHER-CROP-001`,
`BLK-SWEET-BERRY-BUSH-001`,
`BLK-NETHER-WART-001`,
`BLK-NETHER-STEM-001`,
`BLK-SOUL-SAND-001`, `BLK-MAGMA-001`, `BLK-LAVA-CAULDRON-001`; `EXP-PLY-001`,
`EXP-BLK-020`, `EXP-BLK-035`, `EXP-BLK-036`, `EXP-BLK-037`, `EXP-BLK-038`, `EXP-BLK-039`,
`EXP-BLK-066`, `EXP-BLK-067`, `EXP-BLK-068`, `EXP-BLK-069`, `EXP-BLK-072`, `EXP-BLK-073`,
`EXP-BLK-074`, `EXP-BLK-075`, `EXP-BLK-077`, `EXP-BLK-081`

The source-specified transaction owns axis order, epsilons, edge backoff, step selection,
simultaneous shapes, piston restriction and bounce state.
`BLK-MAGMA-001` additionally owns the post-movement step caller: only noncareful living entities
submit one `1.0` hot-floor hit before the base step hook. Immunity and health consequences remain
with the damage/enchantment owners.
`BLK-LAVA-CAULDRON-001` owns the hollow collision/contact boundary and its complete held-item
dispatcher, including server-only hand/stat/write/sound/event order and both above-water gates.
`BLK-SAPLING-001` fixes empty collision for all sixteen states despite the centered selection
cross. Entities and AIR pathfinding pass through; stage and species add no movement/contact hook.
Generic movement, item destination rules and authoritative correction remain with their parents.
`BLK-BAMBOO-001` fixes empty sapling collision and the stalk's offset full-height diameter-3
collider, with diameter-6/10 selection independent of collision. Neither form adds a contact hook;
the stalk explicitly rejects pathfinding.
`BLK-STEM-CROP-001` fixes empty collision and AIR pathfinding for all 24 states. Stem selection is
a centered 2/16 column whose height is `2 + 2*age` pixels; attached selection is a horizontally
rotated 4/16-wide, 10/16-tall fruit-facing arm. Neither form adds movement or contact hooks.
`BLK-OVERWORLD-CROP-001` fixes empty collision, nonoccluding crossed-plant selection shapes and
AIR pathfinding for all 28 states. Wheat, carrot and potato heights follow their eight exact age
shapes; beetroot uses four. Only the Ravager contact override adds mutation, and its
`mobGriefing`-gated destroy-with-drops transaction remains server-authoritative.
The sensor leaf owns the concrete post-move `stepOn` callback's Warden gate and forced-vibration
path, while this parent retains whether movement reaches the callback.
The slime leaf supplies restitution 1.0, the zero-multiplier/omitted fall-damage hook and the
noncareful slow-vertical `stepOn` horizontal multiplier; this parent retains the restitution formula,
bounce event/synchronization and movement-to-callback order.
The honey leaf supplies zero restitution, the reloadable bounce-suppression identity, multiplier-0.2
fall handling and inset-side slide velocity/fall-reset hooks; this parent retains generic collision.
The nether-sprouts leaf supplies empty collision plus the player-only combination-step tag branch:
dry walking emits the sprouts step before a muffled supporting-block step. This parent retains
movement cadence, support selection and the separate water path.
The nether-roots leaf supplies the same empty-collision combination branch for both root colors:
dry walking emits roots step sound 689 before the muffled supporting-block step. Its potted states
instead use their centered 6-by-6-pixel collision column and ordinary stone material sound.
The flower-pot leaf extends that centered `(5,0,5)..(11,6,11)` Stone collider to the empty pot and
all 36 other filled forms; no content identity adds its unpotted contact or movement callback.
The full-copper leaf supplies an ordinary full-cube collision/support shape for every age, wax and
collection. No identity adds contact, fall, speed, jump or entity-inside behavior; each item instead
joins the independent `slow_flat` sulfur-cube equipment matcher.
The soul-sand leaf supplies its 14/16-high collider and ground speed factor 0.4 without adding a
contact callback. Its reloadable Soul Speed membership selects separate enchantment attributes,
durability and effects; this parent retains collision resolution and movement integration.

## `PLY-003` Ground, water, lava, fall flying, and flight share an entry point but not dynamics

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-SERVER-001`; `net.minecraft.world.entity.LivingEntity#travel(net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.LivingEntity#jumpFromGround()`;
`net.minecraft.world.entity.LivingEntity#travelInAir(net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.LivingEntity#travelInFluid(net.minecraft.world.phys.Vec3)`;
`net.minecraft.world.entity.player.Player#aiStep()`; `COM-WIKI-PLY-001`

### Applies when

A living entity advances velocity from input and its current medium.

### Behavior and timing

`travel` dispatches ordinary ground/air to `PLY-MOVE-001` and special
fluid/swimming/fall-flying/ability-flight modes to `PLY-MOVE-SPECIAL-001`; all colliding modes reuse
`PLY-COLLISION-001`. The ordinary leaf fixes jump cooldown, input normalization, friction
acceleration, gravity/effects and drag order.

### Boundaries and quirks

Crossing a medium boundary in one tick, eye-in-fluid versus bounding-box-in-fluid, swimming pose,
elytra launch/landing and ability-flight transitions remain explicit special-mode work and may not
inherit ordinary constants.

### Verification

**Owners:** `PLY-MOVE-001`, `PLY-MOVE-SPECIAL-001`; `EXP-PLY-001`, `EXP-PLY-004`

Ordinary dynamics are source-specified; the special-mode leaf owns the remaining tick-by-tick
trajectory and side-effect matrix.

## `PLY-004` View targeting compares block and entity hits along the camera ray

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#pick(float)`;
`net.minecraft.world.entity.projectile.ProjectileUtil#getHitResultOnViewVector(net.minecraft.world.entity.Entity,java.util.function.Predicate,double)`;
`net.minecraft.world.entity.player.Player#blockInteractionRange()`;
`net.minecraft.world.entity.player.Player#entityInteractionRange()`

### Applies when

The client refreshes its crosshair target or prepares attack/use interaction.

### Behavior and timing

It clips block shapes from the eye/camera along the view vector and ray-tests expanded entity boxes
within entity interaction range. The nearest eligible result becomes a miss, block hit, or entity
hit. The server still validates against its own position, range, and target state.

### Boundaries and quirks

Block outline/collision/interaction shape, fluid mode, entity pick radius, passenger relations, and
reach attributes change candidates. Integer-block DDA without entity-distance comparison is
insufficient.

### Verification

**Owners:** `PLY-INTERACT-001`; `EXP-PLY-002`

Lock exact ties, the eye starting inside a shape, just-over-reach positions, and moving-target
client/server disagreement.

## `PLY-005` Hit type and InteractionResult govern attack/use priority

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Cross-checked`

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`; `net.minecraft.client.Minecraft#startAttack()`;
`net.minecraft.client.Minecraft#startUseItem()`;
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand,net.minecraft.world.phys.BlockHitResult)`;
`net.minecraft.server.level.ServerPlayerGameMode#useItem(net.minecraft.server.level.ServerPlayer,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.InteractionHand)`;
`net.minecraft.world.entity.player.Player#interactOn(net.minecraft.world.entity.Entity,net.minecraft.world.InteractionHand,net.minecraft.world.phys.Vec3)`

### Applies when

The local player presses attack or use while UI, cooldown, spectator, and related gates allow an
action.

### Behavior and timing

Attack chooses entity attack, block-break start, or miss swing from the crosshair result. Use first
attempts the matching entity or block interaction; its `InteractionResult` controls action
consumption, swing, and fallback to the item's own use or the other hand. The server reruns the
rules and synchronizes final item/world state.

### Boundaries and quirks

Main/offhand, sneak bypass of block use, interactable entities, empty items, and “successful without
swing” results make a simple “block first” model inaccurate.

### Verification

**Owners:** `PLY-INTERACT-001`, `ITM-ENDER-CHEST-001`, `ITM-BARREL-001`, `ITM-BOOKSHELF-001`,
`ITM-JUKEBOX-001`, `BLK-COPPER-GOLEM-STATUE-001`, `BLK-BELL-001`, `BLK-ENCHANTING-TABLE-001`,
`BLK-LECTERN-001`, `BLK-BANNER-001`, `BLK-SHELF-001`, `BLK-DECORATED-POT-001`,
`BLK-BRUSHABLE-001`, `BLK-JIGSAW-001`, `BLK-STRUCTURE-001`, `BLK-TEST-BLOCK-001`, `BLK-COMMAND-001`,
`BLK-BEACON-001`, `BLK-SIGN-001`, `BLK-SKULL-001`, `BLK-NETHER-WART-BLOCK-001`,
`BLK-WARPED-WART-BLOCK-001`,
`BLK-NETHER-SPROUTS-001`,
`BLK-NETHER-ROOTS-001`,
`BLK-FLOWER-POT-001`,
`BLK-COPPER-FULL-001`,
`BLK-SAPLING-001`,
`BLK-BAMBOO-001`,
`BLK-STEM-CROP-001`,
`BLK-NETHER-WART-001`,
`BLK-NETHER-STEM-001`,
`ITM-HONEYCOMB-001`, `ITM-STEW-001`, `ITM-BUNDLE-001`, `ITM-BOAT-001`,
`ITM-HARNESS-001`, `ITM-MINECART-001`, `ITM-STEERING-STICK-001`, `ITM-SPEAR-001`,
`ITM-NAUTILUS-ARMOR-001`; `EXP-PLY-002`, `EXP-ITM-017`, `EXP-ITM-018`, `EXP-ITM-021`,
`EXP-ITM-022`, `EXP-ITM-023`, `EXP-ITM-024`, `EXP-ITM-025`,
`EXP-ITM-008`, `EXP-ITM-009`,
`EXP-ITM-010`, `EXP-ITM-011`, `EXP-BLK-008`, `EXP-BLK-009`, `EXP-BLK-010`, `EXP-BLK-011`,
`EXP-BLK-012`, `EXP-BLK-013`, `EXP-BLK-014`, `EXP-BLK-017`, `EXP-BLK-019`, `EXP-BLK-021`,
`EXP-BLK-022`, `EXP-BLK-024`, `EXP-BLK-025`, `EXP-BLK-026`, `EXP-BLK-027`, `EXP-BLK-064`,
`EXP-BLK-065`,
`EXP-BLK-066`,
`EXP-BLK-067`,
`EXP-BLK-068`,
`EXP-BLK-069`,
`EXP-BLK-072`,
`EXP-BLK-073`,
`EXP-BLK-074`, `EXP-BLK-075`, `EXP-BLK-077`, `EXP-BLK-079`,
`EXP-ITM-012`, `EXP-ITM-016`

Concrete leaves fix their success/fallback transactions, including shelf's main-hand/front-face and
pot's client-success/server-failure fallback. Extract the remaining full decision table for every
`InteractionResult` variant and both hands into tests.
The brushable leaf separately owns a consumed start attempt, continuous reraycast, predicted
ten-tick pulse effects and the server-only shared-cooldown commit.
The skull leaf fixes standing/wall candidate order and the placement-triggered wither check while
generic reach, sequence and block-item admission remain in the interaction owners.
The Nether-wart-block leaf owns its held item's composter join: levels 0..6 return success and the
server calls `consume(1, player)` after the chance transaction, level 7 returns success without
mutation, and level 8 delegates. Generic infinite-material handling, hand ordering, prediction and
resynchronization remain with the interaction owners.
The warped-wart-block item takes those identical composter interaction and consumption branches
through its separate 0.85f entry.
The nether-sprouts item takes the same held-item composter transaction with chance 0.5; movement
and combination-step behavior remain with `PLY-002`.
Each nether-roots item takes that composter transaction at chance 0.65. Using it on an empty flower
pot instead commits the matching potted state, statistic, game event and player-aware consumption;
empty-hand removal returns the root to inventory or drops it before restoring the empty pot.
`BLK-FLOWER-POT-001` closes the pot side of that dispatcher for all contents. A mapped item on an
empty pot offers flags 3, then emits `BLOCK_CHANGE`, awards `pot_flower` and consumes one even when
the write failed; a mapped item on any filled pot returns `CONSUME` without exchange. An unmapped
held item returns `TRY_WITH_EMPTY_HAND`, so a filled pot extracts anyway. Extraction gives or drops
the content before its ignored empty-state write and event, with no statistic or explicit sound.
`BLK-COPPER-FULL-001` joins honeycomb and axe use after the main-hand/offhand blocking shortcut.
Honeycomb maps each of twelve unwaxed states to waxed identity, triggers the criterion, shrinks one,
offers flags 11, emits `BLOCK_CHANGE` and level event 3003, and returns success even when the write
fails. An axe prioritizes stripping, then previous weather age, then wax removal: these states never
strip; admitted scraping emits sound 89/event 3005, admitted unwaxing emits sound 90/event 3004,
then a server-player criterion, flags-11 write, `BLOCK_CHANGE`, player-only durability damage and
success. Null-player use skips only criterion/durability, while unaffected and blocking uses pass.
`BLK-SAPLING-001` joins bone meal after generic use-on admission. Server validity checks only the
primary small-tree base height; every valid use consumes one item and emits vibration/event 1505
even when the strict level-RNG `<0.45` success draw misses. A hit stages or invokes the exact
grower transaction, whose material writes all ignore their Boolean results.
`BLK-BAMBOO-001` joins the same generic bone-meal use after a stricter live target check. Sapling
growth is unconditional and RNG-free; stalk growth consumes one `nextInt(2)` and attempts one or
two segments without a light gate, preserving every terminal/height/air/bounds abort and ignored
write result.
`BLK-STEM-CROP-001` joins generic bone-meal use for ages zero through six. The server consumes one
inclusive 2..5 growth draw, clamps at seven and offers a flags-2 write; newly reaching seven then
invokes the ordinary random-tick routine immediately with the same RNG, so its light, crop-speed
and fruit gates still apply. Age seven is not a valid target.
`BLK-OVERWORLD-CROP-001` joins generic bone-meal use for every nonmature crop. Wheat, carrots and
potatoes add an inclusive 2..5 ages before clamping; beetroot integer-divides that increment by
three, so a successful use may offer an unchanged state. Bone meal does not apply the random-tick
brightness or crop-speed gates.
`BLK-TORCHFLOWER-CROP-001` makes crop bone meal deterministic: both stored ages add exactly one,
so age one becomes the mature flower without growth-light or RNG gates. Bone meal on that flower
instead uses the generic supported-neighbor spread search.
`BLK-PITCHER-CROP-001` makes pitcher bone meal deterministic but retains local growth predicates.
Either half resolves a lower crop state without requiring matching ages; a valid target adds one,
then writes lower before the age-three/four upper cell without RNG or rollback.
`BLK-SWEET-BERRY-BUSH-001` joins three ordered interaction paths. Bone meal on ages zero through
two passes to a deterministic one-age flags-2 write; any held item on ages two/three first runs
empty-hand harvest, producing 1..2/2..3 berries, sound, ignored age-one write and block-change
event. Eligible living contact always installs the `(0.8f,0.75,0.8f)` stuck multiplier and, above
age zero, moving at least 0.003 on either horizontal axis offers one bush-damage hit. Its
fall-reset tag separately turns the empty collider into a full ray shape for qualifying movement.
`BLK-CAVE-VINES-001` makes either unlit segment pass empty-hand use so bone meal can set its berry
bit or glow berries can place/eat. A lit segment preempts every held item: exact-one harvest loot,
uniform 0.8..1.2 pick pitch, ignored flags-2 unlit write and player-context `BLOCK_CHANGE` occur in
that order. Both identities compose into `climbable` and `can_glide_through`; their exact movement
effects stay with `PLY-MOVE-001` and `PLY-MOVE-SPECIAL-001`.
`BLK-CHORUS-001` fixes chorus fruit's post-consumption random teleport. At most sixteen attempts
draw three doubles for a candidate in the 16-block cube, clamp Y to logical build height, dismount
before the first attempt and require a loaded column, solid ground, collision-free destination and
no liquid. The first success broadcasts event 46, stops pathfinder navigation, emits `TELEPORT` at
the old position, plays the fox- or generic-teleport sound at the new one, resets fall distance and
the current impulse context. Total failure restores position after every attempt but leaves the
rider dismounted; its ordinary one-second item cooldown still applies.
`ITM-STEW-001` fixes two mob-use joins. Bowl use reaches an adult mooshroom before shears, flower
charging or inherited cow interaction; it consumes or retains the bowl by player ability and
delivers ordinary or stored-effect stew before the matching milk sound. A valid effect flower
charges only an uncharged adult brown mooshroom; a charged one emits reject smoke without consuming
or replacing its payload. Rabbit stew reaches a tamed injured wolf's food branch before its
remaining owned interactions, consumes one directly and heals 20 without creating a bowl.
`ITM-BUNDLE-001` fixes bundle interaction after ordinary menu admission. Primary clicks move the
maximum capacity-fitting prefix into a bundle and secondary clicks remove one complete stored entry;
the two override directions retain their distinct slot-modification gates, partial-transfer rules,
sounds, dirty/broadcast calls and selection clearing. Held use starts a 200-tick `BUNDLE` action and
drops one complete entry on the first use callback and then every second callback from remaining
duration 188 through 2, stopping observable output when removal finds no entry.
`ITM-BOAT-001` fixes boat/raft held use and entity interaction. Held use requires a block POV hit
and rejects when the player's eye is inside any nearby inflated pickable-entity box, then creates
the exact mapped vehicle at the hit, applies stack configuration and player yaw, checks collision
and server-spawns. A post-collision admission failure is ignored, so the server still consumes,
emits `ENTITY_PLACE` and awards the use path. Ordinary vehicle interaction mounts only below 60
out-of-control ticks and without secondary use. A chest form opens its three-row container after
the mount branch passes only when secondary use or passenger-capacity failure selects storage.
`ITM-HARNESS-001` fixes the component-selected entity interaction before Happy Ghast fallback.
Only a live adult Happy Ghast with empty body and live `can_equip_harness` membership accepts one
item; direct equip marks guaranteed drop, emits equip effects and consumes even for creative
players without awarding the generic item-used statistic. A validly harnessed adult then admits
ordinary-use mounting after the held stack passes, while secondary use delegates. Leash removal
precedes equipment shearing, and a passenger-bearing Happy Ghast cannot be sheared.
`ITM-MINECART-001` fixes six item-to-rail-vehicle mappings. Use-on requires the live `rails` tag,
derives slope height, creates with the counterintuitive `DISPENSER` spawn reason and applies stack
configuration. Legacy mode admits overlaps; minecart improvements reject an intersecting cart
after rail adjustment. A surviving server offer ignores entity-admission failure, emits
`ENTITY_PLACE`, shrinks one and awards the use stat. Subtype entity interactions remain exact:
ordinary mounting, chest/hopper menus, furnace fuel, TNT/hopper activation and command editing.
`ITM-STEERING-STICK-001` fixes two server-only held boost transactions. The player must be first
passenger/controller of the saddled exact pig/strider while holding the matching stick; an idle
mount commits one 140..980 synchronized boost before durability 7/1 and returns transformed-stack
success without `item_used`. Wrong/absent controller or an active boost instead awards `item_used`
and passes without RNG/damage. Client use always passes. Switching away pauses the process-local
boost clock; reacquisition resumes it, while entity reload cancels it.
`ITM-SPEAR-001` fixes two distinct attack inputs. Fully charged main-hand `STAB` ray-collects every
eligible entity before the first collider and suppresses the ordinary clicked-entity attack path.
Held use starts either hand for 72000 ticks; after the tier delay, look-projected attacker and target
motion selects damage, knockback and dismount. Contacts are remembered before thresholds, and
release, item change, hand swap or reload clears the process-local contact set.
`ITM-NAUTILUS-ARMOR-001` fixes nautilus interaction ordering. Every interaction first marks the mob
persistent; leash removal precedes equipment shearing, and a tamed adult's secondary use opens its
zero-column inventory before held armor can equip. Ordinary direct use accepts exactly one armor
only for a live tamed adult with empty BODY and current allowed-tag membership, consuming even for
creative without the generic item-used statistic. A passenger-bearing nautilus blocks shearing, but
its rider may open the menu and remove armor there.
`BLK-NETHER-STEM-001` owns the axe's four stem/hyphae strip results after the generic use-on gate.
The main-hand blocking-offhand shortcut returns pass first; an admitted strip preserves axis, plays
sound 88, triggers the player criterion, attempts flags-11 replacement, emits `BLOCK_CHANGE`,
damages the axe and returns success in that order.
The jigsaw leaf owns its matching-entity/game-master gate and client-local edit-screen opening;
generic hit, hand and block-use ordering remain here.
The structure-block leaf owns the same exact-entity/game-master gate and client-only local-screen
opening; admitted use returns `SUCCESS` on both sides and denied use returns `PASS`.
The test-block leaf owns the same local-screen pattern with its independent matching-entity and
game-master gate; denied use returns `PASS`, while admission returns `SUCCESS` on both sides.
The test-instance leaf owns the same matching-entity/game-master gate and client-local non-menu
screen opening; its serverbound actions independently recheck permission but impose no reach or
block-identity test at the supplied position.
The beacon leaf owns its unconditional empty-hand success, server-side matching-entity menu-open
attempt and post-attempt interaction-stat award; shared lock and menu mechanics remain downstream.
The sign leaf owns held-item versus empty-hand fallback, front/back selection, one-editor lease,
click-action-before-wax order, applicator consumption and hanging-sign chain precedence. The
honeycomb item leaf separately owns mapped copper replacement after generic use-on admission.

## `PLY-006` Continuous breaking has client progress and a server-authoritative session

- **FidelityClass:** `ExactObservableBehavior`
- **Evidence status:** `Confirmed`

### Primary evidence

`OFF-CLIENT-001`; `OFF-SERVER-001`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#startDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#continueDestroyBlock(net.minecraft.core.BlockPos,net.minecraft.core.Direction)`;
`net.minecraft.client.multiplayer.MultiPlayerGameMode#destroyBlock(net.minecraft.core.BlockPos)`;
`net.minecraft.server.level.ServerPlayerGameMode#handleBlockBreakAction(net.minecraft.core.BlockPos,net.minecraft.network.protocol.game.ServerboundPlayerActionPacket$Action,net.minecraft.core.Direction,int,int)`;
`COM-WIKI-PLY-001`

### Applies when

A survival/adventure player holds attack on a breakable target, or a creative player instant-breaks.

### Behavior and timing

The client accumulates current per-tick progress for a position plus held item/components, predicts
eligible removal before sending sequenced start/stop actions, and retains pre-write states until a
cumulative acknowledgement. The server independently follows `BLK-BREAK-001`, which recomputes
progress from current speed over elapsed ticks and owns the real transaction. Authoritative updates
are staged behind predictions and applied at acknowledgement.

### Boundaries and quirks

Target state changes do not reset the client when position and item/components still match;
mid-session speed changes can diverge because client accumulation and server whole-interval
recomputation differ. Creative five-call delay, unsequenced aborts, multi-position callback
prediction, teleport-aware collision restoration and ACK/update ordering are separate branches.

### Verification

**Owners:** `PLY-BREAK-001`; `EXP-PLY-003`

The leaf locks all client input, prediction, sequence and convergence transitions. The experiment
owns only whether the source-specified transient ACK restoration reaches a rendered frame before the
subsequent authoritative update.
