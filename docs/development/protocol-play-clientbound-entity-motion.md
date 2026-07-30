# Play Clientbound Entity Motion

`G01-P7-F003` implements the nine packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-MOTION-001` for Minecraft Java 26.2:

| ID | Identity | Projection |
|---:|---|---|
| 35 | `minecraft:entity_position_sync` | absolute pose plus ignored handler velocity |
| 53 | `minecraft:move_entity_pos` | three signed-short deltas |
| 54 | `minecraft:move_entity_pos_rot` | deltas plus two rotation bytes |
| 55 | `minecraft:move_minecart_along_track` | unbounded frame-limited step list |
| 56 | `minecraft:move_entity_rot` | two rotation bytes |
| 83 | `minecraft:rotate_head` | head-yaw byte |
| 101 | `minecraft:set_entity_motion` | finite compact `LpVec3` velocity |
| 125 | `minecraft:teleport_entity` | pose/motion record, fixed relative mask and ground flag |
| 135 | `minecraft:projectile_power` | raw acceleration double |

Position, ordinary motion, yaw, pitch, minecart weight and projectile power retain their IEEE
domains. Rotation bytes are signed values times `360/256`; encoding floors
`angle * 256 / 360` and writes the low byte. Boolean input accepts any nonzero byte and canonical
output writes one.

The shared `LpVec3` codec uses the six-byte packed header plus optional unsigned continuation
scale. It accepts zero-scale and saturated-15-bit noncanonical forms but always decodes finite
components. Canonical output changes NaN to zero, clamps to the locked component limit, emits the
one-byte near-zero form, and preserves both low scale bits when the continuation is present.
Negative/impossible minecart counts, truncation, overlong VarInts and residual bytes fail closed.

## Relative and absolute projection

Relative position resolves each signed short against connection-local packet-position state. A
zero delta preserves the exact base bits. A nonzero delta uses Java
`(round(base * 4096) + delta) / 4096`, including saturating nonfinite conversion and wrapping
signed-long addition. Position packets replace the base before the local-authority gate. A locally
authoritative target then ignores pose, rotation and ground state.

Other targets submit position/rotation to their interpolation handler and install ground state.
The default handler schedules three ticks, uses shortest-path yaw and linear pitch/position, and
does not reset an identical active target. Specialized immediate handlers apply at once and reduce
rotation modulo 360.

ID 35 also replaces the packet base first. Local-authority targets return there. Other targets snap
when squared distance is strictly greater than 4096 or the entity is not ticking; otherwise they
use the movement hook. Its encoded velocity is never applied. A noninterpolating vehicle carrying
the local player repositions that rider after either branch.

## Teleport and specialized projections

Teleport uses an active interpolation target as its source pose when present. The low nine mask
bits independently make position, rotation and velocity relative or request old-to-new rotation of
source velocity; high bits are ignored. Final pitch clamps to `[-90,90]`, with NaN retained.

Ticking, nonlocal or position-relative targets request interpolation. At squared distance at most
4096 this submits pose and installs velocity. Otherwise it sets current and old pose directly,
using zero old velocity for the old-state calculation. A direct locally authoritative vehicle
carrying the player emits an ordinary absolute `move_vehicle` echo. A missing retained former
vehicle instead applies the record directly to the local player, ignores packet ground state,
emits `move_player_pos_rot` with both flags false, and keeps the marker until a same-ID add.

ID 101 invokes immediate velocity convergence and records the old-minecart target where applicable.
Living head rotation interpolates for three ticks; other entities apply immediately. Projectile
power requires the hurting-projectile runtime type and accepts nonfinite values. ID 55 appends only
to enabled new-behavior minecarts. Window activation replaces the current queue, sums float weights
as doubles, opens three ticks for every nonzero total, selects only positive weights, and falls
back to the last step when raw comparisons select none.

## Publication order

The tracker decision model locks the following:

- velocity and paired hurting-projectile power precede the selected pose packet;
- precise positioning, out-of-short deltas, more than 400 ordinary passes, stopped riding or
  ground change choose ID 35;
- otherwise eligible position plus rotation (or an arrow) chooses ID 54, position chooses ID 53,
  and rotation chooses ID 56;
- dirty state follows pose, head follows dirty state, and `hurtMarked` self-inclusive motion is
  last;
- passengers send only qualifying rotation, reset their base and send dirty state;
- new-behavior minecarts send recorded steps or one current snapshot and reset their base;
- controlling riding players receive the transition-relative teleport while other indirect player
  passengers receive the current absolute record with mask zero.

Velocity publication uses squared difference strictly greater than `1e-7`, plus the exact
nonzero-to-zero transition. None of the nine packets carries a sequence or acknowledgement.

## Ownership and evidence

`ferrite-protocol::java_26_2::play::clientbound::entity_motion` separates packet records, exact
codec, client projection and tracker publication decisions. The shared compact-vector codec is
owned at the version-local Play boundary for later entity spawn reuse. Entity IDs, packet bases,
relative masks, packed rotations, interpolation queues and minecart steps remain adapter/session
state; authoritative poses and motion remain normalized simulation values.

`crates/ferrite-protocol/tests/c3/play_clientbound_entity_motion.rs` owns all nine goldens and the
codec, projection, specialized runtime, fault and publication-order evidence.
