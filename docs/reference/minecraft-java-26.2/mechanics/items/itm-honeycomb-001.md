# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-HONEYCOMB-001` — Honeycomb replaces every unwaxed copper stage before emitting its wax transaction

**Parent:** `BLK-002`, `BLK-003`, `BLK-007`, `PLY-005`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-USE-001`, `ITM-ADVANCEMENT-001`, `CLI-006`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked item class and copper collections close the complete unwaxed-to-
waxed block map, state-property transfer, item mutation, advancement, block/game/level-event order
and double-copper-chest companion effects. Honeycomb's separate sign-applicator behavior belongs
to `BLK-SIGN-001`; recipes that combine honeycomb with copper items remain data-driven crafting
under `ITM-CRAFT-001`.

**Applies when:**

A honeycomb stack is used on a registered unwaxed member of any weathering stage in the copper
block, cut copper, cut slab, cut stairs, chiseled copper, door, trapdoor, bars, grate, bulb, chest,
copper-golem statue, lightning rod, lantern or chain collection.

**Authoritative state:**

The item is common, stacks to 64 and has no special data component. A memoized bidirectional map
contains 15 `WeatheringCopperCollection` pairs. Each collection contributes every unwaxed
weathering stage and its corresponding waxed block. The reverse map is exposed for wax removal but
is not consulted by honeycomb use. The separate ten-group `WAXED_RECIPES` map is recipe-generation
metadata, not use-time dispatch.

**Admission and mapping:**

Use reads the clicked state once and looks up its block in the memoized map. A missing entry returns
`PASS` with no item, world, event, criterion or statistic change. A hit creates the target block's
default state and copies every property shared with the original through `withPropertiesOf`; a
property not present on the target is not invented. Already waxed blocks are absent and therefore
pass. The map contains no tag lookup or data-reload boundary.

**Transition and ordering:**

On a mapped state, the method obtains player and held stack. If the player is a `ServerPlayer`, it
first triggers `ITEM_USED_ON_BLOCK` with the original position and pre-shrink stack view. It then
calls `ItemStack.shrink(1)` directly, without the living-entity-aware `consume` helper. Next it
calls `setBlock(position,targetState,11)` and ignores the boolean result. Regardless of write
success, it emits `BLOCK_CHANGE` at the clicked position with player plus target-state context,
then level event 3003 excluding that player, and returns `SUCCESS`.

There is no rollback: criterion and shrink precede the write, and the game/level events follow even
when the set fails. The direct shrink also has no infinite-material check inside this transaction;
client/server inventory convergence remains owned by the generic item-use pipeline.

**Double copper chest companion effects:**

When the captured original block is a chest and its captured `TYPE` is not `SINGLE`, the ordinary
write/event sequence runs first. The implementation then derives the connected position from that
original state, reads the connected block's current state, emits a second `BLOCK_CHANGE` there and
emits a second event 3003 excluding the player. It does not explicitly perform a second `setBlock`
inside this branch; any paired-state convergence follows the chest/block-update transaction. A
single chest and every non-chest copper block have only the primary event pair.

**Client projection:**

Level event 3003 is the wax-on effect: the client produces three through five wax particles on
each exposed direction and plays the honeycomb wax-on block sound at volume and pitch 1. The
authoritative call excludes the initiating player from that event; other tracking clients receive
the authoritative projection, while the initiating client has independently run its local use
path. Exact generic block-update and inventory correction packets remain under `CLI-006` and the
item-use protocol owners.

**Branches and aborts:**

All four weathering stages of all 15 collections; every shared property tuple; already waxed and
unrelated blocks; null, ordinary and server players; stack counts zero/one/many and infinite-
material ability; successful/failed writes; single/left/right copper chests with loaded, changed or
missing connected state; local-client and authoritative-server invocation.

**Constants and randomness:**

15 copper collections; one direct stack decrement; set flags 11; level event 3003; one primary
`BLOCK_CHANGE`/event pair and one additional pair for non-single copper chests. Server gameplay
mapping and commit consume no RNG. Client event rendering consumes its effect RNG for the locked
three-through-five face particles.

**Side effects:**

Item-used-on-block criterion; stack shrink; state replacement; block update work selected by flags
11; one or two `BLOCK_CHANGE` game events; one or two wax level events; client particles and sound.
There is no statistic award in this method, scheduled tick, explicit neighbor rollback, explicit
paired-chest write or persistent item-local state.

**Gates:**

Exact membership in the code-built waxable block map is the only item-specific admission gate.
Player build permission, interaction reach, feature enablement and generic use sequencing are
upstream. Difficulty, game rules, tags, recipes, tool correctness and current weathering age beyond
map membership do not add gates.

**Boundary cases and quirks:**

The direct shrink differs from sign application, which uses `consume` and honors infinite
materials. The method ignores `setBlock` failure and still emits success/effects. A double copper
chest gets two game/event positions without an explicit second write. Criterion observation occurs
before shrinking and replacement. The wax map is code-built from all collection stages rather than
from recipes or tags.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`; `OFF-DATA-001`. Anchors:
`net.minecraft.world.item.HoneycombItem#useOn`,
`net.minecraft.world.item.HoneycombItem#getWaxed`,
`net.minecraft.world.level.block.WeatheringCopperCollection#zipUnwaxedWaxed`,
`net.minecraft.world.level.block.ChestBlock#getConnectedBlockPos`,
`net.minecraft.world.item.ItemStack#shrink`; locked block/item reports and copper registrations;
`EXP-ITM-012`.

**Test vectors:**

Enumerate every map entry and all shared-property combinations; probe unmapped/already-waxed
states, zero/one/many and infinite-material stacks, forced write failure and player types. For
copper chests cross single/left/right with connected-state mutation and capture exact criterion,
shrink, write, game-event and level-event order. Run `EXP-ITM-012` as the executable matrix.
