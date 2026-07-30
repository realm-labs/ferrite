# Redstone Piston Runtime

`G01-P5-S015` implements the `SourceSpecified` `RED-PISTON-001` slice. The
`ferrite-gameplay::redstone::piston` module splits authoritative decisions into power geometry,
atomic structure resolution, ordered world execution, and moving-block/entity behavior. Region
adapters own block storage, block-event queues, mutation commits, entity queries, neighbor
delivery, deterministic RNG, and presentation.

## Power and block-event admission

Power checks `Direction::ALL` around the piston while skipping its facing side, then queries the
piston position toward `Down`, then checks `Direction::ALL` around the block above while skipping
`Down`. The first signal exits. Above power therefore affects the next check but never manufactures
a missing neighbor callback.

An unextended powered piston resolves a complete extension structure before event 0 is queued.
Failure queues nothing. An extended unpowered piston normally queues event 1. A compatible
extending moving piston two blocks ahead selects event 2 when progress is below 0.5, it ticked in
the current game time, or the server is handling a tick.

Server event execution rechecks current power. Restored power before event 1/2 offers an extended
flag-2 write and cancels; lost power before event 0 cancels without a write. Client event handling
does not perform this server revalidation.

## Pushability and atomic resolution

`ResolverWorld` is an immutable observation used to build a finite plan before mutation.
Pushability rejects positions outside world Y/border bounds, the four exact immovable identities,
outward min/max-Y movement, extended piston bases, destroy speed `-1.0`, `BLOCK`, and every block
entity. `DESTROY` requires explicit caller permission. `PUSH_ONLY` compares movement with the
caller's connection direction. Unextended ordinary/sticky piston bases remain admitted.

Extension starts one block ahead and moves along piston facing; retraction starts two blocks ahead
and moves opposite facing. Air succeeds. An initially unpushable `DESTROY` state succeeds only on
extension and becomes the sole destroy entry.

Line construction walks backward through mutually sticky states, stopping at Air, the piston, a
non-sticky boundary, or an unpushable state. Honey and slime never adhere to one another; either
adheres to ordinary states. The backward run is appended farthest-first, then the resolver walks
forward. It admits a single destroy terminator, rejects a piston/unpushable target, and enforces the
12-block limit before every append. Collision reordering moves the newest line ahead of the
collided suffix and rechecks branches. Sticky branches scan every non-push-axis direction in
`Direction::ALL` order. Any failed line or branch rejects the entire plan without mutation.

## Ordered world execution

Execution snapshots all pushed states, then:

1. visits `toDestroy` in reverse, drops resources, writes Air with flag 18, and emits block destroy;
2. visits `toPush` in reverse, writes each destination as moving piston with flag 324 and installs
   a moving entity carrying the snapshot;
3. installs the analogous moving head on extension;
4. writes flag-82 Air to original sources not overwritten by a destination;
5. applies source-indirect, Air-neighbor, and Air-indirect shapes for those clears;
6. visits destroyed positions in reverse for removal hook, indirect shapes, and oriented updates;
7. notifies original pushed positions in reverse;
8. notifies the extension head.

The clear-source collection is intentionally a set; reverse destroy, reverse move, and reverse
notification lists remain distinct ordered sequences. Experimental execution consumes one
bounded-48 orientation draw fixed to resolver push direction.

Successful extension writes the base extended with flag 67, then consumes the sound float for
`float*0.25+0.6` at volume 0.5, then emits block activate. Failed movement performs none of those
steps.

Retraction first finalizes a moving entity directly ahead, writes the base as a retracting source
with flag 276, installs its source moving entity, and immediately updates base neighbors/shapes.
A sticky piston finalizes a compatible extending piece two blocks ahead. Without one, event 1
starts a fresh pull only for non-Air pull-pushable normal reaction or piston bases; event 2 never
starts a fresh pull. Other paths remove the head. Nonsticky retraction always removes it. Sound
then consumes `float*0.15+0.6` and block deactivate follows.

## Moving piston entities

Each admitted moving tick records game time and advances progress by exactly 0.5. Collision
displacement is the largest required separation, capped to progress delta, plus 0.01. `IGNORE`
entities are skipped. Moving slime assigns the movement-axis velocity to ±1 for non-server-player
entities before displacement. Horizontal honey carries on-ground normal-reaction entities on its
top by the exact progress delta; the top AABB ends at `1.5000010000000001`. Retracting source motion
performs the separate piston-base ejection calculation.

Progress reaches 0.5 then 1.0, but normal completion occurs on the following tick. Server completion
removes the block entity. If neighbor-shape adjustment becomes Air it restores the carried state
with flag 340 then runs update-or-destroy with flag 3. Otherwise it clears waterlogged, writes with
flag 67, and emits an oriented neighbor callback. A client retains a completed moving entity for
five additional ticks.

Forced finalization is admitted while previous progress is below 1 or on the client. It removes the
block entity, installs Air for a source piston or the adjusted carried state otherwise with flag 3,
and sends the callback when the live block is still moving piston. Breaking a moving block removes
an extended base behind it. A server use on a moving block with no block entity consumes the action
and removes it; drops come only from a present moving entity's carried state. Moving states are
invisible, have empty outline/clone item, and are not pathfindable.

## Region determinism

Resolver observations are immutable and all-or-nothing. The accepted plan is committed within one
Region transaction; boundary-owned destinations must be preflighted before any destroy/write.
Block events and moving ticks use logical game time. Experimental orientation and successful sound
draws use named Region RNG streams at their explicit post-write points. Cross-Region effects carry
the ordered plan rather than reconstructing it from mailbox arrival.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/redstone/red_004.rs`. Its 15 tests cover every power group,
callback admission, event reversal, pushability class, 12/13 limits, sticky branches, execution
orders/flags, event 1/2 pull behavior, sound arithmetic, progress/finalization, collision/slime/
honey/base-ejection boundaries, and moving-block hooks.
