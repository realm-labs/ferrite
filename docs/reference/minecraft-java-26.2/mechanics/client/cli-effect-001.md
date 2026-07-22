# Client-observable behavior mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `CLI-EFFECT-001` — Effect transport fixes audience and seed; client settings decide presentation

**Parent:** `CLI-006`

**FidelityClass:** `EquivalentPlayerVisibleBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — sound, particle, entity/damage event and level-event transport, client dispatch,
option filtering, RNG ownership and game-event separation are explicit in locked source. Concrete
gameplay leaves own whether and when their committed branch invokes one of these APIs; this leaf
owns the shared observable transaction rather than duplicating every call site.

**Applies when:**

A gameplay leaf invokes a server effect API, a corresponding clientbound packet is handled, or a
declared client-only prediction/presentation path creates an effect directly.

**Authoritative state:**

Effect registry holder/ID and payload, source position/entity, excluded player, level/dimension and
tracking set, volume/pitch/seed/source, particle flags/count/spreads/speed, level-event global bit,
entity existence, client camera/options/RNG, sound resources/device/channels, and the originating
gameplay leaf's branch order.

**Transition and ordering:**

**Server audience and packets:**

Position and entity sounds convert `except` to an excluded player only when it is a `Player`, then
broadcast in the same dimension around the source using `SoundEvent#getRange(volume)` and preserve
holder, category, float volume/pitch and long seed. Entity sound uses a tracking entity ID at the
client; a missing entity suppresses it. The server does not prefilter by the recipient's sound
settings.

Ordinary level events exclude a player source, require same dimension and use fixed radius `64`
from integer block coordinates. Global level events, when `global_sound_events` is true, send to
every server player: a same-level player inside strict squared distance `<32^2` receives the event
at the source center converted back to a block position; a farther same-level player receives it at
their position plus a normalized source direction of length `32`; a different-level player receives
it at their own position. The packet global bit is true. When the gamerule is false the event falls
back to ordinary radius-64 dispatch.

Entity and damage events go to tracking players and the entity itself. Particles iterate the
level's player list: recipients must be in the same level and their integer block position must be
strictly closer than `32` blocks to the particle vector, or `512` when server `overrideLimiter` is
true. `alwaysShow` is carried to the client but does not extend server range. The send method returns
the number of recipients. Server `gameEvent` instead posts directly to the vibration/listener
dispatcher and sends none of these presentation packets by implication.

**Client sound transaction:**

Handlers execute on the packet processor. A position sound creates a seeded simple instance; an
entity sound creates a seeded entity-bound instance only if the entity exists. Ordinary packet
sounds request no distance-of-sound delay. Local sound overloads without a supplied seed consume
one client-level `nextLong`; the optional distance delay applies only above squared camera distance
`100`, with delay ticks `floor((distance/40)*20)`.

Sound resolution selects the resource variant from the supplied seed. The engine rejects unloaded,
disallowed, unknown, intentionally empty or empty events. It clamps pitch to `[0.5,2]`; audible
gain is `clamp(volume,0,1) * clamp(final category volume,0,1) * category gain`. Zero gain normally
does not start a non-music instance unless that instance explicitly permits silent start. Linear
attenuation distance is `max(originalVolume,1) * resourceAttenuationDistance`; supplied volume is
therefore still relevant above one even though output gain clamps. Successful channels are retained
for at least `20` sound ticks. Muting affects presentation, not receipt or server gameplay.

**Client particle transaction:**

For packet count zero, the client creates one particle at the exact position with velocity
`speed*(xDist,yDist,zDist)` and consumes no Gaussian distribution draws. For positive count, each
attempt consumes six client RNG Gaussians: three position offsets multiplied by each spread then
three velocities multiplied by speed. An exception logs and stops the remaining positive-count
loop; the single-particle branch logs its failure.

The client combines packet `overrideLimiter` with the particle type's own override bit. Override
creates directly, bypassing camera distance and particle-option suppression. Otherwise camera
distance must be squared `<=1024`. `alwaysShow` does not mean unconditional: at MINIMAL it has a
one-in-ten chance to promote to DECREASED, after which DECREASED has a one-in-three chance to fall
back to MINIMAL; ordinary DECREASED likewise has that one-in-three suppression. Only a final
non-MINIMAL result creates the particle. These option draws occur before the override branch too,
although an override ignores their result.

**Entity, damage and level-event dispatch:**

Entity event lookup failure is a no-op. IDs `21`, `35` and `63` are client-special: guardian attack
sound; 30-tick totem tracking emitter plus totem sound and local-player activation display; and
sniffer sound. Other IDs call the entity's locked `handleEntityEvent`. Damage events call
`handleDamageEvent` only on a present entity. A level event selects global or ordinary
`LevelEventHandler` solely from the packet bit; that handler owns the fixed ID/data-to-sound/
particle algorithm and client RNG. Client-level `gameEvent` is empty: server vibration semantics
are not recreated from visual packets.

**Prediction and call-site ownership:**

Client-only hit/break, GUI, pickup and use feedback is emitted exactly at its local call site and
does not gain deduplication merely because a later server effect resembles it. A server gameplay
leaf must preserve its own state-mutation/effect call order, excluded initiator and seed. Ferrite may
produce equivalent presentation through the same semantic packets, but cannot replace an entity or
level event with a bare sound when its handler also animates state or creates particles. The
complete call-site catalog is the union of the 65 parent/116 leaf owners and their explicit Side
effects sections; this aggregate owns their shared transports.

**Branches and aborts:**

Excluded player; other dimension; range equality/outside; gamerule global/fallback; missing tracked
entity; missing/empty sound resource or zero category gain; unavailable channel; particle range,
MINIMAL/DECREASED draws, type override and exception; unknown event ID behavior in its handler;
client-only call site not reached.

**Constants and randomness:**

Level-event radii `64` ordinary and projected `32` global; particle send radii strict `<32` or
`<512`; client particle squared range `1024`; distance-delay threshold squared `100` and propagation
speed `40 blocks/s`; sound pitch `[0.5,2]`, gain `[0,1]`, channel minimum `20` ticks; particle
promotion `1/10`, decreased suppression `1/3`, and six Gaussians per positive-count attempt. Sound
seed is server/call-site supplied; packet particle distribution and option filtering use client RNG.

**Side effects:**

Packets, sound channels/subtitles/listeners, delayed sound queue, particles/tracking emitters,
entity animation/state, item-activation display and level-event presentation. Server game-event
listeners/vibrations are a separate authoritative side effect.

**Gates:**

Originating gameplay branch, dimension/audience/tracking/range, global-event gamerule, client packet
thread and entity lookup, registry/resource/device/channel, category settings, camera distance,
particle flags/options/RNG and event-specific handler.

**Boundary cases and quirks:**

`alwaysShow` changes MINIMAL sampling but not server range and is not absolute visibility.
`overrideLimiter` changes both server range and client filtering. Count zero is a one-particle
velocity encoding, not zero particles. Sound volume above one expands server/resource attenuation
range while final output gain still clamps. Sound, particles, entity event, damage cue, level event
and game event are independent effects unless the concrete leaf invokes more than one.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `ServerLevel#playSeededSound`, `#levelEvent`,
`#globalLevelEvent`, `#sendParticles`, `#broadcastEntityEvent`, `#broadcastDamageEvent`,
`#gameEvent`; `ClientPacketListener#handleSoundEvent`, `#handleSoundEntityEvent`,
`#handleParticleEvent`, `#handleEntityEvent`, `#handleDamageEvent`, `#handleLevelEvent`;
`ClientLevel#playSeededSound`, `#playLocalSound`, `#doAddParticle`, `#calculateParticleLevel`;
`SoundEngine#play`; concrete leaf Side effects; `EXP-CLI-003`.

**Test vectors:**

Excluded/source/observer players at one below/equal/above every radius and across dimensions;
global gamerule and near/far/zero-direction projection; entity removed before packet; fixed sound
seed under volume/category/resource/channel variants; delayed distance `10` boundary; particles at
count `0/1`, distance squared `1024`, both flags and every option draw branch; special/default entity
events, missing entity, damage and global/ordinary level event; assert no implicit server game event
and exact concrete-leaf ordering.
