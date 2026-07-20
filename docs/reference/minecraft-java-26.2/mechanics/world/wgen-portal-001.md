# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-PORTAL-001` — Portal travel is cooldown, destination transform, search, creation, and safe placement

**Parent:** `WGEN-005`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client control flow fixes contact accumulation, entity
eligibility, all three portal-family destination paths, the End-portal physical and rendered
surface, Nether POI ranking and creation, exit geometry, gateway generation, passenger transfer and
failure side effects; `EXP-WGEN-003` is conformance-only.

**Applies when:**

A living or projectile entity intersects a Nether portal, End portal or End gateway, an existing
portal is located or a destination structure is created, or a prepared transition moves an
entity/passenger graph. Portal block identity, source/destination dimension keys and dimension types
remain distinct inputs.

**Authoritative state:**

Each entity owns an optional `PortalProcessor`
`(portal block object, immutable entry block, accumulated time, inside-this-tick)`, a cooldown,
pose/dimensions, movement/rotation, passenger graph and type-specific eligibility. The source level
supplies gamerules and key; destinations supply level availability, key/type scale, border, POI
sections, block states/build limits and respawn data. A gateway block entity persists age, optional
spawn-bounds-validated exit position and exact-teleport flag; its 40-tick cooldown is transient.

**Transition and ordering:**

Contact while off cooldown creates a processor with inside true, replaces it immediately when the
portal block object differs, or updates its entry position on the first same-portal contact after
the prior processing tick. Contact while on cooldown instead resets the full entity cooldown and
does not mark a processor. Each server entity tick first decrements a positive cooldown, then
processes the portal: a marked processor clears its inside flag and, if currently eligible, compares
the **pre-increment** accumulated time with the portal wait before returning ready; an unmarked
processor loses `4` accumulated ticks and is discarded at zero. A ready attempt sets the full entity
cooldown **before** resolving the destination; null destinations, disabled entry and rejected
transfers therefore still consume cooldown.

Default portal wait is `0`. A Nether portal gives nonplayers wait `0`; a player uses
`players_nether_portal_creative_delay` when `abilities.invulnerable` is true and
`players_nether_portal_default_delay` otherwise, clamped to at least zero. Locked defaults/minima
are `0/0` and `80/0`. Consequently delay zero attempts on the first processing tick after contact,
while delay `80` attempts after 81 marked processing ticks because the old counter is compared. A
player entity cooldown is `10`; another root uses `10` only when its first passenger is a server
player, otherwise `300`. Remaining inside during cooldown refreshes it on every later contact.
Nether's local transition is `CONFUSION`: the client closes a disallowed screen (and an open
container), plays the trigger sound once at intensity zero with pitch `0.8+0.4*nextFloat` and volume
`0.25`, raises intensity by `0.0125` per marked client tick, otherwise lowers it by `0.05`, clamped
to `[0,1]`. End portal and gateway local transitions are `NONE`.

**Eligibility:**

Base admission is alive and, unless the caller explicitly permits it, not a passenger; living
entities additionally reject while sleeping. Fishing hooks, Withers and Ender Dragons always reject;
a heart-bound Creaking rejects; throwable projectiles always admit regardless of the base predicate.
Thus passengers do not initiate ordinary block contact, but a vehicle root can transfer its graph.
Before transfer, the source level rejects a destination whose key is Nether when source
`allow_entering_nether_using_portals` is false. Cross-key transfer then calls the entity's
`canTeleport`; a root moving literal End→Overworld rejects if any direct passenger is an
unseen-credits server player, and an Ender pearl on that route additionally requires a server-player
owner who has seen credits. Same-key gateway moves skip `canTeleport`.

**Nether destination, lookup and creation:**

A source keyed Nether targets Overworld; every other key targets Nether. Missing destination level
returns null. X/Z are multiplied by `sourceType.coordinate_scale/destinationType.coordinate_scale`,
Y is unchanged, and the destination border clamps/floors the scaled triple to the search block. A
Nether destination searches radius `16`; an Overworld destination searches `128`. The POI manager
loads/validates that area, streams Nether-portal POIs inside the inclusive X/Z square, rejects
positions outside the border or whose current state lacks horizontal axis, then chooses minimum 3D
squared block distance and minimum Y. A residual equal distance/Y tie has no coordinate comparator:
Java `Stream.min` keeps the first encounter from chunk-range, ascending section and the `PoiSection`
hash-set stream, so POI reconstruction/insertion state is observable. The selected state identity
defines the largest matching rectangle, capped `21×21`.

If lookup is empty, creation keeps the source entry axis (default X when absent). It visits the
radius-`16` `BlockPos.spiralAround(target,EAST,SOUTH)` columns, rejects target and one forward block
outside the border, starts at `min(MOTION_BLOCKING height, min(maxY,minY+logicalHeight-1))`,
descends through dry replaceable blocks, requires the base plus four vertical blocks, and measures
target squared distance. A preferred site has solid support and a replaceable `3×4` volume at the
portal plane and both perpendicular offsets; the nearest preferred site wins. Until one exists, the
nearest center-only site is retained. Both comparisons replace only on strictly smaller distance, so
scan order wins a tie. If neither exists, fallback Y is
`clamp(targetY,max(minY-1,70),logicalTop-9)`; an inverted range returns empty. The fallback position
is one block opposite the positive portal-axis direction, then border-clamped; it builds a dry
`3×2×4` support/clearance box, using obsidian at relative Y `-1` and air above. Every success writes
a `4×5` obsidian border and `2×3` axis-valued portal interior; creation failure returns null.

**Nether exit pose:**

The source entry block's exact state identity defines its largest `21×21` rectangle and normalized
coordinates: horizontal fraction is clamped after subtracting the source minimum plus half entity
width (or `0.5` when the entity cannot fit), vertical fraction is similarly clamped (or `0`), and
perpendicular offset is measured from the portal center plane. A missing source axis falls back to
axis X and relative `(0.5,0,0)`. At the destination, the along-plane offset is
`width/2 + (portalWidth-entityWidth)*horizontalFraction`, vertical offset is
`(portalHeight-entityHeight)*verticalFraction`, and plane offset is `0.5+sourcePerpendicular`.
Destination/source axis mismatch adds relative yaw `90`; matching axes add zero. Movement and
rotation are relative, so existing velocity and pitch survive and yaw gains that offset. Entities at
most `4×4` are offered a collision-free adjustment in the surrounding width/height/width search
volume (including one block upward); larger entities and a failed search keep the computed position.
An existing portal tickets the selected POI block; a new portal tickets
`BlockPos.containing(final entity position)`. Both use chunk radius `3`. Their shared sound callback
sends level event `1032` only to a transferred server player; it is a no-op for another entity.

**End portal route:**

Contact by an unseen-credits server player in literal End immediately dismounts/removes the player
instead of creating a processor; when `wonGame` was false it also sets won/seen state and sends the
win-game event. Otherwise a literal-End source targets the saved world-respawn dimension/position;
every other source targets literal End at `(100,50,0)`. Missing target level returns null. Entering
End replaces a `5×5×4` volume centered below the spawn: its bottom `5×5` layer is obsidian and the
next three layers air, destroying mismatches with drops. Nonplayers target bottom center
`(100.5,50,0.5)`; players target `(100.5,49,0.5)`. Yaw is absolute west-facing, pitch remains
relative, velocity survives, then the player-only sound callback and final-position radius-3 ticket
run. Leaving End sends a server player through its normal bed/anchor/world-spawn resolver with no
portal post-effect; a nonplayer uses its adjusted world-spawn block, preserves velocity, applies
saved respawn yaw/pitch as relative rotation, then the no-op sound callback and final-position
ticket. The generic direct-passenger unseen-credits guard applies only when that destination key is
literal Overworld.

**End portal block and subtype surface:**

The sole block state has no collision. Its selection and entity-inside contact shape spans the full
X/Z cell over Y interval `[6/16,12/16]`; the explicit inside shape therefore admits portal contact
across that slab despite collision being empty. It emits light `15`, has hardness `-1`, explosion
resistance `3,600,000`, no loot table, piston reaction `BLOCK`, rejects replacement by every fluid,
returns an empty clone stack regardless of the include-data argument and suppresses ordinary
block-model rendering. Each admitted client `animateTick` consumes exactly two doubles and emits one
zero-velocity smoke particle at `(x+first, y+0.8, z+second)`.

The registered `end_portal` block entity adds no subtype state, persistence, update packet or ticker
behavior; its sole method admits a render face iff the direction axis is Y. Each renderer extraction
clears its reusable face set and inserts only `DOWN` and `UP`, without neighbor culling. Submission
translates a unit cube by `(0,0.375,0)`, scales it by `(1,0.375,1)` and emits four position-only
vertices for each admitted face, so world block-entity geometry is exactly the two horizontal quads
at Y `0.375` and `0.75`. The `end_portal` render type uses the End-sky texture as sampler 0 and
End-portal texture as sampler 1, the position-only quad pipeline, default depth state and shader
define `PORTAL_LAYERS=15`; the locked shader applies its fixed projected base color plus 15
time-scrolled/rotated/scaled portal samples and then environment fog. The built-in special
block-model path reuses the same transform and render type but submits all six unit-cube faces; its
renderer does not branch on supplied light, overlay, foil or outline values.

**End gateway route:**

Server contact requires a gateway block entity not currently cooling; it marks the entity and
immediately sets/broadcasts gateway cooldown `40`, even if the later transition fails. Gateways also
trigger the same cooldown whenever age is divisible by `2400`; age below `200` is the spawning
visual state. A configured exit teleports to its bottom center when exact; otherwise it finds the
highest non-bedrock full-collision block in a radius-`5` square around `exit.above(2)`, excluding
the center, and returns one above it (or `exit.above(3)` when none exists). No exit outside literal
End yields null.

The first unconfigured gateway in literal End normalizes its X/Z direction and starts `1024` blocks
outward. It steps backward by `16` through at most 16 nonempty chunks, then forward by `16` through
at most 16 empty chunks. In the chosen chunk it scans from Y `30` through the highest section for
End stone with two non-full-collision blocks above, retaining strictly smaller squared distance to
world origin and therefore the first encounter on a tie. If absent, it uses
`(floor(x+0.5),75,floor(z+0.5))` and places the locked End-island configured feature when available.
It then chooses the highest full block in the radius-`16` square (first `dx,dz` scan position wins
an equal-height tie), adds `10`, creates a reciprocal gateway whose known exit is the source block
and nonexact, and stores that new gateway as this exit while retaining the source gateway's current
exact flag (default false). An ordinary entity keeps velocity/rotation through relative flags; an
Ender pearl instead exits with zero velocity and zero rotation. Both stay in the same level and
place a radius-3 ticket at the final entity position—no portal sound.

**Passenger and entity transfer:**

A transition first detaches a non-passenger root. Same-level transfer recursively moves passengers
before the root, deriving passenger positions from their offsets relative to the vehicle and their
rotation differences, marks their transitions `asPassenger`, applies the position/movement/rotation
relatives, then runs each post-transition callback. Cross-level transfer snapshots and ejects direct
passengers, recursively transfers them, creates a destination instance of each ordinary entity with
`DIMENSION_TRAVEL`, restores state, removes the old instance as `CHANGED_DIMENSION`, positions/adds
the new one, and remounts successfully transferred passengers. A server player overrides this by
moving the same player instance and synchronizing
respawn/difficulty/permissions/abilities/level/player/effect state; dimension-change criteria and
Nether travel tracking run. If ordinary root type creation fails, already transferred passengers
remain transferred while the old ejected root remains. Spectator players whose camera was the
transferred entity follow it and reset their camera.

**Branches and aborts:**

Cooldown/no cooldown; same/different portal object; marked/decaying/expired processor;
player/nonplayer and invulnerable ability; sleeping/passenger/dead/override eligibility;
missing/disabled destination; same/cross key; unseen credits; POI found/tied/stale;
preferred/fallback/no creation site; source axis present/absent; small/large/colliding exit;
entering/leaving End; gateway exact/nonexact/unconfigured/non-End/cooling; pearl/ordinary entity;
same-level/cross-level/player/type-creation failure.

**Constants and randomness:**

Wait defaults `0/80`, decay `4`, entity cooldown `10/300`, search `16/128`, creation radius `16`,
portal interior `2×3`, maximum rectangle `21×21`, safe-size limit `4`, portal-ticket radius `3`, End
spawn `(100,50,0)`, End-portal slab/render bounds `6/16..12/16`, smoke Y `0.8`, render layers `15`,
gateway spawn/cooldown/attention `200/40/2400`, radial gateway start/step/limit `1024/16/16`,
gateway surface offsets `10`, `5` and `16`. Lookup, ranking, creation-site selection, exit geometry
and gateway terrain search consume no RNG. Each End-portal display call consumes two doubles;
gateway feature placement receives a fresh random source; the client confusion trigger consumes one
float and other presentation particles own their separate draws.

**Side effects:**

Processor/cooldown state, client confusion/screen/sound state, End-portal smoke and submitted render
geometry, destination chunk/POI loading, obsidian/air/portal/platform/island/gateway blocks, block
drops from End platform replacement, block-entity save/event state, radius-3 tickets, entity
removal/creation/remounting, player network resynchronization, velocity/rotation/position, credits
and advancement criteria.

**Gates:**

The three portal block implementations; entity overrides; source/destination keys and levels; the
three owned gamerules; world border, logical/build height and collision; portal POI/current block
state; compatible End-portal block-entity renderer admission; gateway block-entity cooldown/exit
fields and configured features; respawn state; passenger/credits state. Portal ignition/frame
completion, random piglin spawning, End-frame activation and configured-feature geometry retain
their block/worldgen owners; this leaf owns transfer contact through completed transition and the
End-portal subtype surface.

**Boundary cases and quirks:**

The wait comparison is post-increment storage but pre-increment comparison. All blocks of one portal
block class share the same portal object, so moving between separate same-type structures can retain
the processor/timer. Attempt failure still starts entity cooldown; gateway contact failure also
starts gateway cooldown. Nether search distance includes Y even though eligibility uses an X/Z
square. A final POI tie is hash encounter order, not lexicographic position. Creation uses logical
top but Nether build storage extends above logical height. Player entry into End is one block below
the nonplayer target. A gateway's nonexact exit deliberately excludes the stored gateway center.
Cross-level ordinary transfer can partially move a passenger graph when root construction fails.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`. Anchors: `PortalProcessor`,
`Portal`, `Entity#setAsInsidePortal`, `Entity#handlePortal`, `Entity#teleport`,
`LivingEntity#canUsePortal`, `ThrowableProjectile#canUsePortal`, `NetherPortalBlock`,
`PortalForcer`, `PortalShape`, `PoiManager#getInSquare`, `PoiSection#getRecords`,
`Blocks#END_PORTAL`, `EndPortalBlock`, `TheEndPortalBlockEntity#shouldRenderFace`,
`BlockEntityRenderers`, `TheEndPortalRenderer`, `AbstractEndPortalRenderer`, `BuiltInBlockModels`,
`EndCubeSpecialRenderer`, `RenderTypes#endPortal`, `RenderPipelines#END_PORTAL`,
`EndPlatformFeature`, `EndGatewayBlock`, `TheEndGatewayBlockEntity`, `TeleportTransition`,
`ServerLevel#isAllowedToEnterPortal`, `ServerPlayer#showEndCredits`, and `ServerPlayer#teleport`.

**Test vectors:**

(1) Sweep wait `0/1/80`, leave/reenter decay, same/different structures and cooldown refresh for
players, vehicles and projectiles. (2) Toggle all three gamerules and every eligibility override,
including sleeping, heart-bound Creaking, bosses, hooks, pearls and unseen-credit direct/nested
passengers. (3) Search at `15/16/17` and `127/128/129`, border edges, Y-distance ties, equal
distance/Y POIs under different insertion/reload histories and stale POIs. (4) Exhaust preferred,
center-only, fallback and inverted-height creation paths; assert every block/flag and logical-height
boundary. (5) Sweep portal axes, rectangle sizes, oversized entities, edge fractions, perpendicular
offsets and blocked collision adjustment. (6) Enter/leave End with player/nonplayer, custom
source/respawn keys, missing destination, platform replacements and credits. (7) Exercise gateway
cooldown/attention, exact/nonexact/missing exits, zero/radial directions, empty/nonempty chunk runs,
candidate/island fallback and pearl motion. (8) Transfer nested passenger graphs same/cross level,
spectator cameras and a destination type-construction failure. (9) Probe End-portal contact at Y
`6/16` and `12/16`, clone/fluid/display calls and RNG counts; extract every direction and assert two
world quads at `0.375/0.75`, all-six special-model faces, bound textures, 15 layers and fogged time
evolution. Run `EXP-WGEN-003` only as an executable conformance matrix.
