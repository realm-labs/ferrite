# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-STATE-001` — Registered block states are closed, canonical property tuples

**Parent:** `BLK-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked block report supplies every block's property names, allowed
serialized values, complete legal tuples and default tuple; the runtime source specifies canonical
transition and invalid-input behavior.

**Applies when:**

Code creates, stores, compares, serializes, derives or patches a block state.

**Authoritative state:**

A block owns one immutable `StateDefinition`; each state is a canonical
`(registered block, one value per declared property)` object from that definition.
`reports/blocks.json` is authoritative for the 1,196 definitions and their default/legal tuples.
Block-entity data, fluid ticks, scheduled ticks and item components are not members of this tuple.

**Transition and ordering:**

A property transition first requires that the exact property object belongs to the state's
definition, then requires a value from that property's allowed set. A valid different value returns
the definition's already-created canonical state; setting the current value returns the same state.
Missing properties or out-of-domain values throw rather than constructing an extra state. The item
`minecraft:block_state` component instead iterates its string map: an unknown property name or
unparsable value is ignored, while each valid entry replaces that property in the accumulating
canonical state. `BlockItem` applies this component only **after** the initial placement write; if
the result is a different canonical state it writes it with flag `2` and ignores that write's
boolean result. It does not rerun survival, collision or placement-state derivation after this
patch.

**Branches and aborts:**

Direct runtime mutation throws for a foreign property/value. Component application skips only the
invalid entry and continues. An empty component returns its input reference without a world write. A
valid component can select any reported tuple, including one that ordinary placement derivation
would not have selected.

**Constants and randomness:**

Property values are finite strings/booleans/integers/enums exactly as reported. State transitions
consume no RNG and perform no numeric rounding. Reported numeric state IDs are lookup data, not
gameplay identity.

**Side effects:**

Pure state transitions have none. The placement component's optional flag-`2` write has exactly the
publication and storage effects specified by `BLK-UPDATE-001`; it has no ordinary neighbor
notification and does not itself consume an item.

**Gates:**

Registry/version lock, owning state definition, property membership and value parser. Gameplay mode,
difficulty and chunk activity do not alter the tuple set.

**Boundary cases and quirks:**

Canonical reference equality is observable inside mutation control flow: writing the same canonical
state can be rejected as no change. The lenient component parser and strict direct API intentionally
differ. A post-placement component can create a state that immediately becomes invalid only when
later neighbor/shape work evaluates it.

**Evidence:**

`Confirmed`; `OFF-REPORT-001`; `OFF-SERVER-001`;
`net.minecraft.world.level.block.state.StateHolder#setValue(net.minecraft.world.level.block.state.properties.Property,java.lang.Comparable)`;
`net.minecraft.world.item.component.BlockItemStateProperties#apply(net.minecraft.world.level.block.state.BlockState)`;
`net.minecraft.world.item.BlockItem#updateBlockStateFromTag(net.minecraft.core.BlockPos,net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.level.block.state.BlockState)`.

**Test vectors:**

(1) For all reported states, round-trip every property tuple and default; assert no unreported tuple
is constructible. (2) Set a property to its current value, another legal value, a foreign property
and an invalid value; assert identity, canonical transition and both exceptions. (3) Apply a
component containing one valid, one unknown and one invalid entry; assert only the valid property
changes. (4) Place a block with a valid but normally unselected state component and assert the
initial flag-`11` write precedes the flag-`2` patch without a second placement validation.
