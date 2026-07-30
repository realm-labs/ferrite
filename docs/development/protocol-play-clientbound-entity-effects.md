# Play Clientbound Explosions and Mob Effects

`G01-P7-F002` implements all three packets in
`PROTO-PLAY-CLIENTBOUND-ENTITY-EFFECTS-001` for Minecraft Java 26.2:

| ID | Identity | Body |
|---:|---|---|
| 36 | `minecraft:explode` | center, radius, block count, optional knockback, particle, sound holder, weighted particle recipes |
| 78 | `minecraft:remove_mob_effect` | entity VarInt and mob-effect holder |
| 132 | `minecraft:update_mob_effect` | entity/effect holders, amplifier, duration and flags |

Explosion vectors retain IEEE-754 values and block count is a fixed big-endian signed integer.
Particles use the locked 125-entry static type table and type-specific options. Block-state options
are limited to `0..=32365`; dust scales follow the vanilla `0.01..=4.0` clamp. Vibration sources
strictly distinguish block and entity forms. Item particles delegate to the shared item-template
and component-patch codec.

A sound holder value of zero carries an inline identifier and optional fixed-range float. Other
values subtract one and resolve through the connection-local sound registry. The final recipe
count is nonnegative and bounded by the frame. Individual weights must be nonnegative and their
signed total must not overflow. Unknown holders/types, mismatched options, invalid delegated data,
truncation and residual bytes fail before semantic use.

## Explosion presentation

Authoritative explosion calculation and world mutation complete before this adapter publishes a
packet. The small primary particle is selected when radius is below two or block interaction is
disabled; otherwise the large particle is selected. Only players at squared distance strictly less
than 4096 are recipients, and knockback is selected independently for each recipient.

The client projection plays sound at volume four with pitch
`(1 + (random1 - random2) * 0.2) * 0.7`, emits the primary particle with velocity `(1,0,0)`, queues
the tracker only when recipe weight is positive, then adds optional knockback to existing player
motion. On the next tracker tick, non-`ALL` settings discard the queue. The `ALL` path sums signed
block counts with overflow checking and attempts at most 512 samples before clearing the queue.
Recipe selection and the sampled-air test remain presentation concerns; no packet acknowledges the
explosion.

## Effect replacement and publication

Update handling resolves the current entity and requires a living target that can accept the
effect. Amplifier clamps to `0..=255`; exactly `-1` is infinite and other nonpositive durations have
no remaining tick. Only the low four flag bits control ambient, visible, icon and blend behavior.
A same-holder update replaces rather than merges, has no hidden effect and copies the prior blend
state. Removal requires a living target and silently ignores an absent holder.

Publication plans preserve the audited order:

- add/update marks particle metadata dirty, performs the applicable attribute operation, sends
  blend-clear updates to direct player passengers, then sends the self-player update;
- only a newly added self effect sets blend; replacement and the finite 600-tick refresh clear it,
  and the periodic refresh performs no attribute refresh;
- removal removes modifiers, sends direct-passenger removals, refreshes affected attributes, then
  sends the self removal;
- initial self replay retains the active-effect map's supplied iteration order and clears blend;
- mount positions and challenges the rider, replays vehicle effects with blend clear, then sends
  the passenger list; dismount sends every removal before the passenger list.

Indirect passengers and ordinary tracking viewers are excluded from active-effect packets. Dirty
metadata and syncable attributes converge through their separately owned packet families.

## Ownership and evidence

`ferrite-protocol::java_26_2::play::clientbound::entity_effects` separates packet values, the strict
codec and particle table, client projection, and publication ordering. Namespaced effect and sound
identities cross the adapter boundary; raw holders, packet flags, particle IDs, recipe counts and
client tracker/blend state remain version-local.

`crates/ferrite-protocol/tests/c3/play_clientbound_entity_effects.rs` owns three exact goldens, all
125 particle option shapes, signed/IEEE/registry/malformed boundaries, effect replacement and
removal, explosion ordering and tracker overflow, audience selection, and lifecycle/mount ordering.
