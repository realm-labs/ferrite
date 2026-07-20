# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-ENCHANTING-TABLE-001` — Enchanting tables own menu ingress, custom name, bookshelf particles, and a client-only shared-RNG book clock

**Parent:** `SIM-005`, `BLK-001`, `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `ITM-001`, `ITM-002`,
`CLI-005`, `CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source, block/item reports, both bookshelf tags and block
loot fix the table's state/shape/light, menu-provider boundary, custom-name component/persistence,
all 32 particle probes, client animation recurrence and renderer transform. Offer generation,
enchantment selection, lapis/XP commit and `enchant_item` remain explicitly owned by unfinished
`ITM-ENCHANT-001`; no part of that downstream algorithm is claimed here.

**Applies when:**

An enchanting table is placed, picked/broken, queried for a menu, used through the main-hand
try-empty-hand fallback, receives a client display tick, or its client block-entity book
ticker/renderer runs.

**Authoritative state:**

The block has no properties and exactly one state. It is red-map-colored, base-drum instrumented,
correct-tool-required, light level 7, strength `5/1200`, nonpathfindable, and a full-X/Z column from
Y 0 through 12 whose shape is also used for light occlusion. The block entity persistently owns
nullable custom name; all ten animation fields and the class-static `RandomSource` are
client-transient.

**Transition and ordering:**

Generic placement may apply the item custom-name component before later menu/loot use. Main-hand
block interaction constructs/opens only the provider described below. Independently, every client
display tick probes all bookshelf offsets with the level RNG, and every admitted client block-entity
tick advances proximity, opening, rotation and page state in the exact order below before render
interpolation.

**Menu and name boundary:**

The base item-on-block result requests empty-hand fallback; only main-hand fallback calls
`useWithoutItem`, while generic secondary-use bypass with either hand nonempty skips it. The client
always returns success without opening. The server asks the current state for a provider and calls
`openMenu`, then also returns success; missing/wrong block entity yields null and no menu without
changing the result. A matching subtype supplies its custom display name or translatable default
`container.enchant`, and creates
`EnchantmentMenu(containerId,inventory,ContainerLevelAccess.create(level,pos))`. Spectator provider
opening remains the generic interaction path. All slot/offers/cost/commit behavior after
construction is `ITM-ENCHANT-001`.

Custom name saves as nullable `CustomName`, loads through safe component parsing, supplies
`Nameable`, is applied/collected as implicit `minecraft:custom_name`, and removes the legacy tag
field after item-component extraction. The default item has max stack 64 and no custom name. Pick
block uses the generic default item and does not copy the name. The locked loot returns one table
through `survives_explosion` and copies only custom name from the block entity; no animation field
is saved, synchronized or copied.

**Bookshelf particle scan:**

`BOOKSHELF_OFFSETS` is the immutable `betweenClosedStream(-2,0,-2,2,1,2)` order filtered to
`abs(x)==2 || abs(z)==2`, exactly 32 offsets. For every offset on every client animate tick, consume
`nextInt(16)` first. Only zero then reads validity: the target must be the sole
`enchantment_power_provider` member `bookshelf`, and the midpoint `(x/2,y,z/2)` using Java integer
division must be in `enchantment_power_transmitter`, which expands `#replaceable`. Invalid/roll-miss
probes consume no floats. Each valid hit consumes three floats and emits one `ENCHANT` particle at
table `(x+0.5,y+2,z+0.5)` with parameters `(offsetX+f-0.5, offsetY-f-1, offsetZ+f-0.5)`.

**Client book tick:**

There is no server ticker. Each client tick first copies open/rotation to their previous fields,
then selects the first nearest nonspectator player in client player-list order at strict squared
distance below 9 from block center; creative is admitted. With a player, target rotation is
`atan2(playerZ-centerZ,playerX-centerX)` and open increases 0.1. If the new open is below 0.5,
short-circuit directly to page selection; otherwise first consume shared-static `nextInt(40)` and
select a page only on zero. Page selection repeatedly consumes two `nextInt(4)` and adds their
difference to `flipT` until it differs from its pre-loop value. Without a player, target rotation
adds 0.02 and open subtracts 0.1 with no page RNG.

Normalize current and target rotation independently into `[-PI,PI)`, wrap their difference likewise,
add 40% of it to current rotation, clamp open to `[0,1]`, increment signed `time`, copy previous
flip, clamp `0.4*(flipT-flip)` to `[-0.2,0.2]`, move flip acceleration 90% toward that value, then
add acceleration to flip. The static page RNG is shared by every client table/world and is neither
world-seeded nor persisted; table ticker order and prior client history therefore select its stream.

**Renderer:**

Interpolate flip/open linearly and use `time+partialTicks`. Wrap `rot-oRot` into `[-PI,PI)` and
interpolate yaw from `oRot`. Translate to `(0.5,0.75,0.5)`, add vertical `0.1+sin(time*0.1)*0.01`,
rotate Y by negative yaw then Z by 80 degrees. Page inputs are `clamp(frac(flip+0.25)*1.6-0.3,0,1)`
and the same with `+0.75`; the book model receives those, time and open. Chunk unload/reload resets
animation and changes future shared-RNG consumption while persistent custom name remains.

**Branches and aborts:**

Main/offhand/secondary-use, client/server, missing subtype, custom/default/malformed name, explosion
condition, all 32 offsets, roll miss, provider/transmitter fail, nonspectator/creative/ties/range
equality, opening threshold, 1/40 page branch, repeated zero delta, rotation wraps,
integer/time/float evolution, unload and render partial ticks.

**Constants and randomness:**

Shape/light/strength above; 32 probes, bound 16 and three floats per admitted particle; player
radius 3; open/idle rotation `0.1/0.02`; page bound/chance `4/40`; rotation factor 0.4; page clamp
0.2 and acceleration factor 0.9; renderer constants above. Particle RNG is client level RNG; page
RNG is one static client stream.

**Side effects:**

Menu open packet/provider and title, custom-name persistence/components/loot, client enchant
particles, transient book state and rendered transforms. No table-owned stat, game event, sound,
server animation, or RNG occurs; downstream enchant commits are separately owned.

**Gates:**

Generic interaction priority, logical side, current block-entity subtype, explosion survival, client
display/ticker activity, provider/transmitter tags, per-offset roll, nearest nonspectator strict
range, open threshold/page chance and renderer availability/settings.

**Boundary cases and quirks:**

A proper server/client use reports success even when no provider exists. Every offset consumes a
bounded draw before world reads. Opening ticks below 0.5 force page-target changes and skip the 1/40
draw. Creative players animate the book; spectators do not. Static page randomness couples otherwise
unrelated tables and is lost with process history, while custom name alone survives.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.EnchantingTableBlock`,
`net.minecraft.world.level.block.entity.EnchantingTableBlockEntity#bookAnimationTick`,
name/component/save methods, `net.minecraft.client.renderer.blockentity.EnchantTableRenderer`;
locked reports, `data/minecraft/tags/block/enchantment_power_provider.json`,
`data/minecraft/tags/block/enchantment_power_transmitter.json`,
`data/minecraft/loot_table/blocks/enchanting_table.json`; `EXP-BLK-010`.

**Test vectors:**

One-state properties/shape/light/pathfinding; main/offhand empty/item/secondary/spectator use with
matching/wrong/missing subtype; default/custom/malformed name through placement/save/load/menu/loot
plus generic pick-name loss; all 32 offset orders with every roll/tag/midpoint and exact float
sequence; player list ties, creative/spectator and squared distance below/equal/above 9; open
0.4/0.5, chance and repeated equal page draws; rotation at every ±PI wrap, clamps, long time and
partial-tick renderer formulas; multiple tables sharing RNG and unload/reload. Run `EXP-BLK-010` as
the executable matrix.
