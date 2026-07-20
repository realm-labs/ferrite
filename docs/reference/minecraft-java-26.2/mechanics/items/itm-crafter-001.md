# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-CRAFTER-001` — A crafter rising edge schedules one cached craft and pushes or dispenses every output

**Parent:** `ITM-004`, `ITM-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — pulse scheduling, disabled-slot input, cache lookup, output/remainder delivery,
input shrink, comparator and insertion balancing are explicit in locked source.

**Applies when:**

A crafter gains neighbor power, its scheduled block tick runs, its animation timer ticks, or
automation inserts items.

**Authoritative state:**

Nine stacks and nine disabled flags, block `TRIGGERED`/`CRAFTING`/orientation, recipe cache,
six-tick animation counter, front container and nearby players.

**Transition and ordering:**

On unpowered-to-powered transition, schedule a block tick after `4`, set block/entity triggered
true; losing power clears triggered/crafting flags but does not cancel an already scheduled tick. At
the scheduled tick, build a cropped crafting input in which disabled empty slots remain explicit
constraints, then use `ITM-RECIPE-001`. No match or empty assembly emits failure event `1050` and
changes no inventory. Success sets animation counter `6`, sets `CRAFTING`, invokes system-crafted on
the result, and delivers the result followed by each nonempty recipe remainder. Delivery targets the
container immediately in front: a front crafter or a stack larger than the destination maximum is
offered one item at a time; otherwise repeated whole-stack insertion continues while count
decreases. Any residue is dispensed `0.7` blocks forward with speed `6`; only this residue branch
triggers nearby-player crafted criteria and level events `1049`/`2010`. After all deliveries, shrink
every nonempty backing slot by one and mark changed. The block-entity ticker decrements the
animation counter each tick and clears `CRAFTING` when it reaches zero.

**Branches and aborts:**

Pulse width shorter than four ticks still crafts because the scheduled tick remains. A destination
may accept all, some or none of each output independently; delivery occurs before input shrink and
therefore destination callbacks precede consumption. Remainders are delivered separately rather than
returned to their source cells. Failed recipe/result leaves inputs intact.

**Constants and randomness:**

Trigger delay `4`; animation `6`; nine slots; crafted-criterion AABB size `17`; ejection offset
`0.7` and speed parameter `6`. Recipe/cache/insert branches consume no RNG; residue entity motion
uses dispenser spawning RNG.

**Side effects:**

Trigger/crafting block state, output insertion or item entities, remainder delivery, input counts,
dirty state, failure/success events and conditional nearby-player criteria. Comparator output equals
the count of slots that are nonempty or disabled, from `0..9`.

**Gates:**

Rising neighbor signal, scheduled tick, recipe match and enabled nonempty assembly, destination
sided/capacity rules, slot enabled state and insertion balancing.

**Boundary cases and quirks:**

Automation rejects a disabled or full slot. For a nonempty candidate slot, if any later enabled slot
is empty or holds the same stack at a smaller count, insertion is rejected there; this makes
ordinary ascending-slot automation fill/balance later slots first. Full insertion into the front
container bypasses the residue branch and therefore emits neither the two dispense events nor the
nearby-player crafted criterion. Input consumption occurs even when output was only partially
inserted and the rest dispensed.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.world.level.block.CrafterBlock#neighborChanged(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.Block,net.minecraft.world.level.redstone.Orientation,boolean)`,
`net.minecraft.world.level.block.CrafterBlock#dispenseFrom(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos)`,
`net.minecraft.world.level.block.CrafterBlock#dispenseItem(net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.level.block.entity.CrafterBlockEntity,net.minecraft.world.item.ItemStack,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.item.crafting.RecipeHolder)`,
`net.minecraft.world.level.block.entity.CrafterBlockEntity#serverTick(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.CrafterBlockEntity)`,
`net.minecraft.world.level.block.entity.CrafterBlockEntity#canPlaceItem(int,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.entity.CrafterBlockEntity#getRedstoneSignal()`; `EXP-ITM-003`.

**Test vectors:**

One-tick pulse; second edge before/after scheduled tick; no match/empty result; disabled cells in
shaped pattern; all/full/partial/no front insertion for ordinary and crafter destinations;
multi-count result and each remainder; verify delivery-before-shrink callback; animation countdown;
comparator `0..9`; balancing across equal stacks.
