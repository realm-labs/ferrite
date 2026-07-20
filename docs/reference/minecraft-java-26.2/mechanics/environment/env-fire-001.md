# Environment mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENV-FIRE-001` — Fire aging, extinguishing, spread, and fuel destruction are ordered scheduled-callback branches

**Parent:** `ENV-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server class files and bundled 26.2 data fix ordinary/soul placement
and survival, portal dispatch, the near-player rule, schedule and age transitions, rain and
infiniburn branches, every direct and spatial spread roll, the complete hardcoded flammability
table, positional increased-burnout values, target writes/removals, TNT dispatch, base-fire contact
and all failure behavior. Client-only ambient presentation is owned by `CLI-EFFECT-001`, entity
fire-duration/damage processing after base-fire contact by `ENT-EFFECT-001`, and portal construction
after frame recognition by `WGEN-PORTAL-001`; `EXP-ENV-003` is a conformance trace, not an
implementation blocker.

**Applies when:**

A base fire is placed or receives a shape/contact callback, or an ordinary `fire` scheduled callback
is admitted for the current block at a loaded position. Soul fire never runs the ordinary age/spread
callback.

**Authoritative state:**

The captured ordinary-fire state and age, current surrounding states, waterlogging, face sturdiness,
the dimension type's infiniburn holder set, local rain predicates from `ENV-WEATHER-001`, difficulty
ID, the positional environment-attribute system, `fire_spread_radius_around_player`, player
positions/spectator flags, the level RNG, block-write results, and the TNT rule/entity-admission
path.

**Transition and ordering:**

Placement selects ordinary/soul state and portal dispatch before scheduling ordinary fire. A due
ordinary-fire callback reschedules first, applies the near-player gate, then performs survival
removal, infiniburn/rain, age/self-removal, positional-attribute resolution, six direct fuel
transactions and the ordered empty-space scan. Base-fire entity contact orders freeze clearing,
ignition and queued contact damage separately from that scheduled state machine. The following
sections fix each subtransaction and every branch-local draw.

**Placement, survival, and soul dispatch:**

`BaseFireBlock.getState` chooses default `soul_fire` exactly when the block below is in
`#minecraft:soul_fire_base_blocks` (`soul_sand`, `soul_soil`); otherwise it asks ordinary fire for
its placement state. Ordinary fire returns its all-false directional default when the below block
has positive ignite odds or a sturdy upper face. Otherwise it sets each `up`, `north`, `south`,
`west`, and `east` property to whether that adjacent state has positive ignite odds; `down` has no
property. It survives when the below upper face is sturdy or any of all six adjacent states has
positive ignite odds. A surviving ordinary-fire shape callback reselects through
`BaseFireBlock.getState`: it can therefore become default soul fire over a newly selected soul base,
while an ordinary result recomputes directional properties and preserves age; a failed survival
check returns air. Soul fire survives only over that two-member tag, returns default soul fire after
a surviving shape update and air otherwise; its `canBurn` override is always true for inherited
presentation layout, but it defines no scheduled tick or age/spread table.

On a base-fire placement whose old block is the same block, base processing returns. Otherwise
overworld/nether placement first searches an empty portal shape preferring axis X and then Z and, on
success, creates portal blocks and returns; absent a portal, an unsurvivable state is removed with
`drop = false`. Ordinary fire then schedules itself even if base processing just replaced/removed
it; the scheduler's current-block validation makes that entry stale. The separate `canBePlacedAt`
predicate requires air and either a survivable selected state or a portal candidate. Portal
candidacy is overworld/nether only, first requires any of six neighbors to be obsidian, then uses
the clicked horizontal direction's counterclockwise axis as the preferred axis; a vertical click
consumes the level RNG to choose that preferred horizontal axis. The portal search falls back to the
other axis. Frame recognition/construction beyond this dispatch belongs to `WGEN-PORTAL-001`.

**Near-player and schedule gate:**

Both placement and every executing ordinary-fire callback sample `30 + nextInt(10)`, hence delay
`30..39`; a due callback schedules its successor before any other work. It then reads
`fire_spread_radius_around_player` (ordinary default `128`, minimum `-1`). Value `-1` admits
unconditionally. Any other value searches the chunk map's full player set and admits iff a
nonspectator has strict Euclidean distance `< radius` from their exact position to `Vec3(x,y,z)` at
the fire block's integer corner. Equality fails; radius `0` therefore denies every player. Denial
returns after scheduling and performs no survival, age, rain, environment-attribute, fuel or spread
work. Java 26.2 registers no `doFireTick` rule.

**Ordered ordinary-fire callback:**

After admission, execute exactly these stages with the captured state and callback RNG.

1. If the captured state cannot survive, call `removeBlock(pos,false)` and ignore the result, but do
   **not** return; later stages still use its captured age. Read the current below state and test it
   against the dimension type's infiniburn set. Locked `overworld` and `overworld_caves` use
   `#infiniburn_overworld`; the nether tag includes that tag, so all three resolve to `netherrack`
   and `magma_block`. The end tag additionally contains `bedrock`.
2. Unless below is infiniburn, active level rain plus `isNearRain(pos)` consumes `nextFloat()` and
   removes/returns on strict `< 0.2F + age*0.03F`. `isNearRain` short-circuits `isRainingAt` in
   exact order current, west, east, north, south; it never probes above or below.
3. Sample `newAge = min(15, age + nextInt(3)/2)` with integer division, so only result `2` adds one.
   If changed, write the updated captured state with flags `260`, ignoring failure.
4. Unless infiniburn, if no adjacent state has positive ignite odds, remove and return when the
   below upper face is not sturdy or captured age is greater than `3`; otherwise return with no
   burn/spread work. If there is adjacent fuel and captured age is `15`, consume `nextInt(4)`;
   result `0` removes/returns only when below itself is not fuel.
5. Resolve `minecraft:gameplay/increased_fire_burnout` at the fire position. It is a positional
   boolean with default `false`; exactly `bamboo_jungle`, `frozen_peaks`, `jagged_peaks`, `jungle`,
   `mangrove_swamp`, `mushroom_fields`, `snowy_slopes`, and `swamp` set it to `true` in locked biome
   data.
6. Invoke direct burnout in exact order east, west, below, above, north, south. Horizontal
   denominators are `300`, vertical denominators `250`; increased burnout subtracts `50` from each.
   Every invocation consumes `nextInt(denominator)` even for a waterlogged or unregistered target
   whose burn odds are zero.
7. Traverse candidate offsets with X outer `-1..1`, Z middle `-1..1`, Y inner `-1..4`, skipping
   `(0,0,0)`. A nonempty candidate or one whose six neighbors all have zero ignite odds consumes no
   draw. Otherwise let `encouragement` be the maximum neighboring ignite odds and compute integer
   `threshold = (encouragement + 40 + difficultyId*7)/(age+30)`, then integer-divide it by `2` under
   increased burnout. A positive threshold consumes `nextInt(denominator)`, where the denominator is
   `100` through Y `1`, then `200`, `300`, `400` at Y `2`, `3`, `4`; the candidate passes on result
   `<= threshold` (inclusive). After that draw, active rain plus `isNearRain(candidate)` rejects it.
   Otherwise consume `nextInt(5)`, add `result/4` to captured age, clamp to `15`, choose
   ordinary/soul state from the candidate's current support, and write with flags `3`, ignoring
   failure.

**Direct target transaction:**

A direct target's effective burn odds are zero when it has `waterlogged=true`; otherwise use the
table below, defaulting to zero. A first draw below burn odds captures the current target and
consumes `nextInt(age+10)`. If that result is below `5` and `isRainingAt(target)` is false, consume
`nextInt(5)`, add `result/4` to captured age, clamp to `15`, select ordinary/soul fire from the
target's support and write it with flags `3`. Otherwise remove the target with `drop = false`. Both
write/removal results are ignored. If the captured target block was any `TntBlock`, call
`TntBlock.prime` **after** that mutation. A server with `tnt_explodes=false` rejects priming, but
the TNT remains already removed/replaced. Otherwise priming constructs a centered primed-TNT entity
at integer Y, ignores admission failure, and still plays the primed sound and emits `PRIME_FUSE`;
explosion/fuse behavior then belongs to `RED-EXPLOSION-001`. Fire destruction never invokes ordinary
loot or item drops.

**Locked `(ignite odds / burn odds)` table:**

Every block not listed has `0/0`; a listed state with `waterlogged=true` also has effective `0/0`.
Ignite odds control support, directional shape and empty-position encouragement; burn odds control
direct consumption.

- `5/5`: `oak_log`, `spruce_log`, `birch_log`, `jungle_log`, `acacia_log`, `cherry_log`,
  `pale_oak_log`, `dark_oak_log`, `mangrove_log`, `bamboo_block`; all corresponding stripped logs;
  all stripped and unstripped `oak`, `spruce`, `birch`, `jungle`, `acacia`, `cherry`, `pale_oak`,
  `dark_oak`, and `mangrove` wood; `stripped_bamboo_block`; `coal_block`.
- `5/20`: the ten overworld/bamboo planks (`oak`, `spruce`, `birch`, `jungle`, `acacia`, `cherry`,
  `dark_oak`, `pale_oak`, `mangrove`, `bamboo`), `bamboo_mosaic`; their eleven slabs and eleven
  stairs including bamboo mosaic, plus the ten fences and ten fence gates without a mosaic variant;
  `mangrove_roots`, `composter`, `beehive`.
- `5/100`: `pale_moss_block`, `pale_moss_carpet`, `pale_hanging_moss`.
- `15/20`: `target`.
- `15/60`: `cave_vines`, `cave_vines_plant`.
- `15/100`: `tnt`, `vine`, `glow_lichen`.
- `30/20`: `bookshelf`, `lectern`, `bee_nest`; `acacia_shelf`, `bamboo_shelf`, `birch_shelf`,
  `cherry_shelf`, `dark_oak_shelf`, `jungle_shelf`, `mangrove_shelf`, `oak_shelf`, `pale_oak_shelf`,
  `spruce_shelf`.
- `30/60`: the nine overworld leaves from `oak` through `mangrove`; `azalea_leaves`,
  `flowering_azalea_leaves`, `azalea`, `flowering_azalea`, `hanging_roots`, `dried_kelp_block`;
  every one of the 16 registered color wool blocks (`white`, `orange`, `magenta`, `light_blue`,
  `yellow`, `lime`, `pink`, `gray`, `light_gray`, `cyan`, `purple`, `blue`, `brown`, `green`, `red`,
  `black`).
- `60/20`: `hay_block` and every one of the same 16 color carpets.
- `60/60`: `bamboo`, `scaffolding`.
- `60/100`: `short_grass`, `fern`, `dead_bush`, `short_dry_grass`, `tall_dry_grass`, `sunflower`,
  `lilac`, `rose_bush`, `peony`, `tall_grass`, `large_fern`, `dandelion`, `golden_dandelion`,
  `poppy`, `open_eyeblossom`, `closed_eyeblossom`, `blue_orchid`, `allium`, `azure_bluet`,
  `red_tulip`, `orange_tulip`, `white_tulip`, `pink_tulip`, `oxeye_daisy`, `cornflower`,
  `lily_of_the_valley`, `torchflower`, `pitcher_plant`, `wither_rose`, `pink_petals`, `wildflowers`,
  `leaf_litter`, `cactus_flower`, `sweet_berry_bush`, `spore_blossom`, `big_dripleaf`,
  `big_dripleaf_stem`, `small_dripleaf`, `firefly_bush`, `bush`.

**Base-fire contact boundary:**

Fire and soul fire first request `CLEAR_FREEZE`, then `FIRE_IGNITE`, then queue their damage after
that effect. Fire-immune entities abort ignition. A negative remaining-fire counter increments by
one; otherwise only a `ServerPlayer` consumes level RNG `nextInt(1,3)` and adds `1` or `2`. If the
resulting counter is nonnegative, ignition is set to eight seconds. The queued contact damage is
`in_fire` for `1.0F` from ordinary fire and `2.0F` from soul fire. The inside-effect applier's
deduplication/order and subsequent `fire_damage`, immunity, armor/effect and recurring-burn handling
belong to `ENT-EFFECT-001`; this leaf fixes only base-fire dispatch and its RNG.

**Branches and aborts:**

Stale scheduled entry/current-block mismatch; radius denial; survival removal without early return;
infiniburn bypasses rain and self-removal only; rain chance; no adjacent fuel; old-age removal; zero
encouragement/threshold; inclusive spread miss; post-draw rain rejection; waterlogging; direct burn
miss; replacement versus removal; failed state write; disabled TNT or entity-admission failure;
portal/soul dispatch.

**Constants and randomness:**

Age is `0..15`. Schedule, rain, age, old-age, six direct transactions and 53 spatial candidates
consume the single callback RNG only at the branch sites above; portal vertical-axis selection and
player contact use the level RNG at their own sites. Increased burnout makes direct destruction more
likely by lowering denominators but makes empty-space spread less likely by halving its threshold.

**Side effects:**

Scheduled ordinary-fire ticks; fire age/directional state writes; no-drop removals; ordinary or
soul-fire placement; neighbor/client updates implied by flags `260`/`3`; TNT entity/sound/game event
after an already-committed fuel mutation; base-fire freeze clearing, ignition and contact damage;
portal dispatch. The ordinary scheduled callback itself emits no ambient sound, particle or game
event.

**Gates:**

Current scheduled block, loaded tick admission, strict nearby nonspectator radius or `-1`,
support/neighbor flammability, infiniburn holder set, active/local rain, age, positional
increased-burnout attribute, difficulty, waterlogging, target emptiness/state writes,
`tnt_explodes`, entity admission, portal dimension/frame, and fire immunity/contact deduplication.

**Boundary cases and quirks:**

A radius-denied fire persists and keeps rescheduling without even rechecking survival. A failed
survival removal still consumes later callback work and can affect neighbors. Infiniburn does not
prevent aging or outward/direct spread. Rain adjacency is horizontal only and short-circuited.
Direct burnout draws six times even around nonflammable blocks; empty-space scans do not. The
spatial comparison is inclusive. Fire can replace a target with soul fire based on support. TNT
priming is nontransactional with its preceding removal, and failed entity admission does not
suppress sound/event.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`;
`net.minecraft.world.level.block.FireBlock#tick(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.util.RandomSource)`,
`FireBlock#checkBurnOut`, `FireBlock#bootStrap`,
`net.minecraft.world.level.block.BaseFireBlock#getState`, `BaseFireBlock#onPlace`,
`BaseFireBlock#canBePlacedAt`, `BaseFireBlock#fireIgnite`,
`net.minecraft.world.level.block.SoulFireBlock#canSurvive`,
`net.minecraft.server.level.ServerLevel#canSpreadFireAround`,
`net.minecraft.server.level.ChunkMap#anyPlayerCloseEnoughTo`,
`net.minecraft.world.level.block.TntBlock#prime`,
`net.minecraft.world.attribute.EnvironmentAttributes`, locked dimension types/block tags and the
eight biome JSON records above. `EXP-ENV-003` replays these specified branches.

**Test vectors:**

Radius `-1/0/128` with spectator, integer-corner/equality boundaries and no player; delay endpoints;
unsupported fire whose removal succeeds/fails; each infiniburn set; every rain-neighbor direction
and float equality; age draw `0/1/2`, age `3/4/15`; both increased-burnout values; six direct
directions with waterlogged/default/every odds pair; every spatial Y denominator, difficulty,
threshold zero/equality and post-draw rain; soul support; TNT with rule/admission failure; portal
horizontal/vertical click; ordinary/soul entity contact with negative/nonnegative counters and
player/nonplayer.
