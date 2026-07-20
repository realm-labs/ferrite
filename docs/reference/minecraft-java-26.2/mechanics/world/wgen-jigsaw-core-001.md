# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-CORE-001` — Jigsaw placement expands prioritized pool connectors through a bounded collision volume

**Parent:** `WGEN-003`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the generic jigsaw start transaction, all three
pool-alias families, weighted primary/fallback pools, all five pool-element families, connector
ordering/attachment, rigid and terrain-matching alignment, expansion-hack and collision behavior,
priority traversal, junctions, the sole saved piece and operator-triggered generation. All 188
locked template-pool records and the topology metadata of all 994 jigsaw-family templates are
audited data-only here. The ten concrete structure records and 40 processor lists are owned by
`WGEN-JIGSAW-RECORDS-001` and `WGEN-JIGSAW-PROCESSORS-001`; ancient-city, bastion, outpost,
trail-ruins, trial-chambers and village template payloads are owned by
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-BASTION-001`, `WGEN-JIGSAW-OUTPOST-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001` and `WGEN-JIGSAW-VILLAGES-001`; all
six locked payload families are explicit and complete at this layer.

**Applies when:**

A jigsaw structure has sampled its record-owned start height and asks for a generation stub, a
placed connector expands a template pool, a saved `jigsaw` piece places/reloads, or an authorized
jigsaw-block request invokes immediate generation.

**Authoritative state:**

The start pool and optional start connector name; depth, heightmap projection, expansion flag,
horizontal/vertical range, aliases, padding and liquid mode; template-pool registry, template
manager and processor inputs; structure RNG and independent alias RNG; piece
boxes/positions/rotations/ground deltas/junctions; pending placement-priority queues and the two
mutable free-space shapes.

**Transition and ordering:**

Depth is `0..20`. Horizontal range is `1..128`; scalar form uses the same vertical range, while
record form defaults vertical to the dimension Y size and admits `1..Y_SIZE`. Padding is a
nonnegative scalar or bottom/top record, default shared zero. Liquid defaults to apply waterlogging.
Terrain adaptation other than none reserves another `12` horizontal blocks, so decode rejects
horizontal range plus `12` above `128`; none uses no reserve.

The concrete structure first samples its height provider with the structure RNG at source-chunk
minimum X/Z. Alias resolution then uses a separate stream made from world seed, forked positionally
at that unadjusted start position. Bindings execute list order: direct writes one mapping; random
makes one weighted target draw; random-group makes one weighted group draw and executes that group's
nested bindings in order. Duplicate aliases fail immutable-map construction rather than overwrite.
Unmapped keys remain identity mappings. This alias stream never advances the structure RNG.

The core next consumes a uniform rotation, resolves an aliased keyed start pool when possible, and
draws one element from its weight-expanded list; an empty list returns the Empty element. Empty
aborts. With a named start connector, the selected element shuffles all connector records, stably
sorts selection priority descending, and takes the first matching `name`; no match logs and aborts.
Before the later ground-level move, the chosen connector is translated exactly onto the unadjusted
start point. Without a name, element origin stays there.

A center piece is constructed before projection. Its center X/Z is Java integer division of each box
min+max by two. Without a projection heightmap, target ground Y is adjusted-origin Y. With one, it
is unadjusted start Y plus the first-free height at center X/Z. The piece moves by target ground Y
minus `(box.minY + groundLevelDelta)`. Padding that is not the identity-equal shared ZERO rejects
when the moved box minimum is below `dimension.minY+bottom` or maximum is above
`dimension.maxY-top`; shared ZERO deliberately skips this check. The stub position is center X,
projected ground Y plus the pretranslation connector-local Y, center Z.

The deferred builder adds the center to a private discovery list. Depth zero returns before copying
even that center into the builder, yielding an empty start. Positive depth creates an allowed AABB
centered on the stub: horizontal endpoints are center ± range with exclusive upper `+1`; vertical
endpoints are range-clamped to padded dimension bounds. The initial external free shape is this AABB
minus the moved center box. Discovered pieces are appended to the builder only after graph expansion
finishes, preserving discovery rather than processing order.

**Connector traversal and pools:**

A source piece shuffles its connectors, then stably sorts selection priority descending. For each
connector it computes the outward neighbor from the rotated front, aliases the named pool, and skips
a missing pool, an illegally empty non-`empty` pool, or an illegally empty non-`empty` fallback. An
outward neighbor inside the source box uses a lazily initialized source-local free shape equal to
that source box; all other connectors share the external/context free shape. Thus internal children
can occupy disjoint subsets of their parent independently of external siblings.

When current depth differs from maximum, candidate order is a fresh shuffle of every weight-expanded
primary entry followed by a fresh shuffle of every expanded fallback entry. At maximum depth,
primary is omitted but fallback remains. Encountering Empty terminates the entire combined iterator;
it does not merely reject that entry, so later primary entries and the fallback can be suppressed.
Every nonempty candidate consumes all four rotations in shuffled order. For each rotation its
connector list is independently shuffled and stable-sorted by selection priority.

Attachment requires source front opposite target front, source `target` exactly equal target `name`,
and either source joint rollable or equal rotated top directions; target joint is ignored. Candidate
origin initially puts the target connector at the source outward neighbor. Let
`deltaY = sourceLocalY - targetLocalY + sourceFront.stepY`. If both projections are rigid, candidate
box Y is source box minimum plus delta. Otherwise the source connector's `WORLD_SURFACE_WG`
first-free height is cached and candidate box Y is that height minus target connector local Y.

The optional expansion hack is computed once per candidate rotation only for original box height at
most `16`. For every candidate connector whose own outward neighbor remains inside that candidate
box, it finds the larger maximum Y span of its aliased pool and fallback. The greatest value expands
the candidate box upward to include `minY + max(childSpan+1, oldMaxY-oldMinY)`; placement origin and
template payload do not expand. Collision admits only when the candidate box deflated `0.25` on
every side is wholly inside the applicable free shape. Admission subtracts the full, un-deflated
box; rejection tries the next connector/rotation/element.

An admitted element becomes a depth-zero `jigsaw` piece. Rigid targets inherit source ground delta
minus deltaY; nonrigid targets use the element default. The paired junction Y is source box minimum
plus source-local Y for rigid source, target box Y plus target-local Y for
nonrigid-source/rigid-target, or cached surface plus Java-truncated `deltaY/2` when both are
nonrigid. Each side saves the opposite projection and signed delta. The piece is appended
immediately. Depth `max+1` is retained as a terminal piece but not queued; otherwise it enters a
queue keyed by the source connector's placement priority. Exactly one candidate can attach to a
source connector.

Pending work always removes the highest numeric priority; equal priority is FIFO. A newly added
higher-priority queue can preempt lower pending work. When a queue empties, the iterator selects the
current greatest nonempty key. Each queued piece repeats the connector transaction with its own
internal free shape but the shared external shape.

**Element and placement behavior:**

A pool expands each weight `1..150` into repeated element references. Random choice is one bounded
index; shuffling is over the expanded list. Its cached max size is the greatest nonempty unrotated Y
span. Rigid projection adds no processor. Terrain-matching appends
`GravityProcessor(WORLD_SURFACE_WG,-1)`.

- Empty has terrain-matching projection, zero size, no connectors, no box, and a successful no-op
  placement. Its special iterator position is what terminates candidate search.
- Feature has zero size and a point box. It exposes one synthetic downward/rollable connector named
  `bottom`, targeting `empty`, with empty pool and final air. Placement invokes its placed feature
  at element origin without consulting the piece clip and returns that feature result.
- List must be nonempty, forces its projection onto every child, uses componentwise maximum size and
  the union of nonempty child boxes, but exposes connectors only from child zero. It places children
  in record order, short-circuits on false and never rolls earlier writes back.
- Single exposes its template's shuffled/priority-sorted connectors and ordinary rotated box.
  Settings are known-shape, entity-inclusive/finalizing, chunk-clipped, flags `18`, with processors
  in order: ignore structure blocks, optional jigsaw replacement, configured processor list, then
  projection processors. Its liquid override wins over the structure setting. Successful template
  placement transforms DATA structure blocks and calls the inherited no-op marker handler.
- Legacy single starts from Single settings, removes ignore-structure and appends
  ignore-structure-and-air after all other processors. It therefore preserves live cells where
  template air would otherwise be offered.

Unless an operator requests keeping jigsaws, replacement parses each jigsaw's `final_state`. Missing
NBT leaves the jigsaw unchanged; parse failure drops that block; structure void also drops it; any
other parsed state replaces it without block NBT. The compile-time debug keep flag bypasses
replacement. Connector records themselves were extracted before placement, so replacement does not
alter the already-built graph.

**Persistence and immediate generation:**

Every generated piece uses registry ID `jigsaw`, protocol `55`, depth `0`, and saves generic box
plus position, ground delta, pool-element codec, rotation, junction list and nondefault liquid
setting. Load defaults missing position/delta to zero, requires a valid element/rotation, recomputes
its box, restores junctions, and defaults liquid. Moving mutates box and element position together.
Junction equality omits source-ground-Y even though its hash includes it; ordinary storage is an
ordered list.

Ordinary chunk placement delegates the element with the processing box, shared reference point and
`keepJigsaws=false`; its Boolean is ignored. Immediate operator generation instead uses the level
RNG, a permissive biome predicate, named connector, max horizontal/vertical `128`, no expansion,
projection, aliases or padding, and default liquid. A present stub places every pool piece
immediately in builder order against an infinite box, passing the request's keep-jigsaws flag,
ignoring each result and returning true; an absent stub returns false.

**Locked corpus boundary:**

There are exactly 188 pool records: empty `1`; ancient-city `7` pools/65 weighted entries/expanded
weight `107`; bastion `60/176/189`; outpost `4/11/16`; trail ruins `7/84/84`; trial chambers
`47/213/1534`; villages `62/649/2950`. Across 1,198 weighted entries the element counts are legacy
single `601`, single `527`, feature `36`, empty `31`, list `3`; projections are rigid `984`,
terrain-matching `183`, and the 31 Empty records omit projection. There are nine distinct fallback
IDs.

Those pools name 989 templates. Exactly 988 exist;
`ancient_city/walls/intact_horizontal_wall_stairs_5`, weight one in `ancient_city/walls/no_corners`,
is missing from the locked jar. The six present jigsaw-family templates not named by any pool are
ancient-city `city_center/walls/bottom_right_corner`, three village decay grass templates, and snowy
normal/zombie `streets/crossroad_01`. The complete six-family corpus is 994 single-palette templates
with 869,846 block entries, 393,131 explicit air, no structure void, 3,754 jigsaws, 426 non-jigsaw
block-NBT entries—one a non-DATA structure block—and 62 template entities. Connector metadata has
1,840 aligned and 1,914 rollable entries, selection priorities `0:3717,1:34,2:3`, placement
priorities `0:3704,1:44,2:5,3:1`, 160 distinct pool IDs and 70 final-state strings. Exact payload
interpretation is owned by the six named family leaves and processor interpretation by
`WGEN-JIGSAW-PROCESSORS-001`.

**Branches and aborts:**

Codec range/adaptation; scalar/record range/padding; every alias draw/map conflict; empty/missing
start pool/element/name; four rotations; projected/unprojected and padding pass/fail; depth
zero/positive/max/max+1; inside/outside connector; missing/empty primary/fallback; every expanded
shuffle and Empty sentinel position; four target rotations; attachment front/top/name outcomes; four
rigidity pairs; surface cache miss/hit; expansion disabled/tall/zero/positive; collision
reject/admit; priority/FIFO/preemption; all five element types and child failures; replacement
missing/invalid/void/state/debug; clip/template/feature/entity outcomes; save defaults/errors;
immediate absent/present/keep/result outcomes.

**Constants and randomness:**

Record height sampling precedes this core's rotation and weighted start draw. Named connector,
source connectors, primary/fallback lists, rotations and target connectors consume the structure
stream exactly in traversal order. Expansion max-size inspection consumes none. Alias resolution
uses its separate positional stream. Placement continues caller RNG through element
processors/entities/features; exact record payload draws remain with follow-up owners.

**Side effects:**

An ordered saved piece graph and junction list; chunk-clipped template blocks, block entities and
finalized template entities; optional placed features; processor/liquid/shape effects; jigsaw
final-state replacement; warnings/errors for invalid resources; immediate operator writes.

**Gates:**

Record-owned start height/biome/set admission; pool/template/processor registry availability; alias
mapping; connector metadata; depth/range/padding/heightmap; free-shape collision; element placement
and live world state; processing box; operator authorization and request fields.

**Boundary cases and quirks:**

Depth zero returns an empty builder. Fallback still runs at maximum depth and can add a max+1
terminal piece. Empty terminates the combined candidate stream. Source joint alone controls top
alignment. Expansion changes collision only, not payload. Collision deflates for admission but
subtracts the full box. Internal and external children use different free volumes. Discovery order
differs from priority processing order. Legacy air suppression runs after configured/projection
processors. Feature elements ignore the piece clip. One locked pool locator is missing and six
payloads are unreachable from locked pools.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors: `JigsawStructure`;
`JigsawPlacement` and `Placer`; `SequencedPriorityIterator`; `JigsawBlock#canAttach`;
`StructureTemplatePool`; all five `StructurePoolElement` implementations; all three alias
implementations and `PoolAliasLookup`; `PoolElementStructurePiece`; `JigsawJunction`;
`JigsawReplacementProcessor`; all 188 `worldgen/template_pool` records and topology fields from all
994 jigsaw-family NBT inputs; three alias, five element and `jigsaw` piece registry entries.

**Test vectors:**

Replay exact streams across all codec, alias, start, named-anchor, projection, padding and depth
edges; adversarial expanded pools with Empty at every index; all
attachment/rigidity/surface/expansion/collision/priority cases; every element and replacement path;
save-load/move and immediate generation. Assert exact pool census, connector
priorities/joints/pools/final states, missing/unreachable inputs and graph
discovery/processing/placement order. `WGEN-JIGSAW-PROCESSORS-001` asserts every processor/list
composition; the ancient-city, bastion, outpost, trail-ruins, trial-chambers and village leaf owners
assert every locked payload family.
