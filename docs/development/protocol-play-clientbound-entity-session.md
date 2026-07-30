# Play Clientbound Entity Session

`G01-P7-F004` implements the six packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-SESSION-001` for Minecraft Java 26.2:

| ID | Identity | Projection |
|---:|---|---|
| 2 | `minecraft:animate` | entity and unsigned action |
| 25 | `minecraft:damage_event` | damage holder, biased cause/direct IDs and optional position |
| 42 | `minecraft:hurt_animation` | entity and raw yaw float |
| 82 | `minecraft:respawn` | common spawn record and signed keep byte |
| 93 | `minecraft:set_camera` | camera entity |
| 124 | `minecraft:take_item_entity` | source, collector and signed amount |

All entity IDs remain current-level lookup keys. Damage type and respawn dimension type resolve
strictly through the frozen connection registries. Cause/direct values add one on encode and
subtract one on decode with wrapping signed-int arithmetic. Optional damage position and hurt yaw
retain all IEEE values. Boolean input accepts nonzero bytes and canonical output writes one.
Unknown configured holders, malformed/truncated primitives and residual bytes fail closed.

## Feedback and damage

Animation actions zero and three require a living target and swing the main/off hand. Action two
requires a player and wakes it. Actions four and five accept any entity and create critical or
enchanted particles. Missing targets and unknown actions are ignored; the explicit wrong-type
casts fault.

Hurt animation accepts any present entity and raw yaw. It is independently published directly to
the damaged player after unblocked indication. Damage event first resolves the damaged target.
Missing targets ignore the event and nonliving base handling is inert. For a living target,
position-present creates a positional source and deliberately skips cause/direct lookup; otherwise
the two entity IDs resolve independently and may stay absent. The handler installs walk speed
`1.5`, invulnerability 20, hurt time/duration 10, the selected damage presentation and current
client game time.

Canonical damage-event publication requires the full unblocked damage branch and targets trackers
plus self. It carries neither amount nor health, absorption, knockback, death, wear or criteria.
Those remain separate authoritative projections.

## Camera and pickup

Camera changes only for a currently present entity. Canonical publication changes authoritative
camera ownership, relocates the player, updates chunk tracking, sends ID 93, then resets known
position. A missing camera target leaves existing state unchanged and produces no response.

Pickup resolves and type-checks the collector before inspecting the source. A missing collector
falls back to the local player; a present nonliving collector faults even when source is absent.
A present source creates sound/particle projection. Item count uses wrapping subtraction by the
signed amount and removes at `count <= 0`; a negative amount can grow the stack. Experience orbs
remain. Every other source is removed regardless of amount. Publication reaches players tracking
the source and does not automatically include a player source itself. No inventory grant or
transaction acknowledgement is inferred.

## Respawn replacement

Respawn reuses the strict common-spawn codec. Keep bit `0x01` retains complete attribute
values/modifiers; without it, only bases survive. Bit `0x02` retains player position/input,
sprinting, nondefault entity data, velocity and yaw/pitch; without it position/motion reset and yaw
becomes `-180`. Higher bits are ignored and both tests are independent.

A dimension-key change creates a new level generation and drops level-scoped debug subscriptions.
Every valid packet clears the camera and open container, replaces the local player, marks
client-loaded false, starts the appropriate wait reason, preserves entity ID/stats/recipe book,
adds the replacement and makes it the camera. Duplicate packets replace again without a sequence.

Death respawn publication begins with respawn, then position challenge, default spawn, difficulty,
experience, active effects, level info and permission. Cross-dimension publication begins with
respawn, then difficulty, permission, transfer, position challenge, abilities and new-level
projection. The later `player_loaded` packet closes the independently owned readiness interval.

## Ownership and evidence

`ferrite-protocol::java_26_2::play::clientbound::entity_session` separates packet records, strict
codec, client feedback/session projection and publication order. The existing shared respawn
record remains in `clientbound::session`. Entity numbers, raw holders, masks, UI/camera state and
pickup cache deltas remain version-local; authoritative damage, inventory, relocation and player
lifecycle state remain in their gameplay owners.

`crates/ferrite-protocol/tests/c3/play_clientbound_entity_session.rs` owns all six goldens,
wrapping/IEEE/registry/fault boundaries, runtime type gates, feedback/damage/camera/pickup behavior,
respawn keep intersections and publication ordering.
