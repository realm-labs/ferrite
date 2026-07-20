# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-VAULT-001` — Vaults maintain a rewarded-player exclusion set and eject a resolved reward in reverse list order

**Parent:** `BLK-003`, `BLK-007`, `PLY-006`, `ITM-001`, `ITM-003`, `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client classes and the two locked trial-chamber vault payloads
determine activation hysteresis, key interaction, loot inputs, rewarded-player retention, four-state
timing, display synchronization, reverse ejection, persistence quirks and client effects. Loot
evaluation and spawned-item motion dispatch to their generic owners, but the vault fixes their
contexts and call order.

**Applies when:**

A loaded `vault` block entity ticks, a player uses a nonempty stack on an active vault, its
`inactive`, `active`, `unlocking` or `ejecting` state changes, config/server/shared data load or
save, or the client ticks synchronized display data. The normal/ominous trial-chamber templates
initially provide inactive state, trial/ominous key and reward table; their exact NBT is owned by
`WGEN-JIGSAW-TRIAL-CHAMBERS-001`.

**Authoritative state:**

Block state owns horizontal facing, ominous and vault state. Config owns reward table, strict
activation/deactivation radii, exact key stack, optional display-table override and the
inclusive-creative/nonspectator detector. Server data owns insertion-ordered rewarded UUIDs,
absolute state-resume time, ordered pending stacks, transient last-failure time and total ejection
count. Shared saved/synchronized data owns display stack, connected UUID set and particle range;
client data owns current/previous spin only. Locked normal/ominous configs use radii `4/4.5`, one
trial/ominous key, their matching reward table, no display override and default particle range
`4.5`.

**Transition and ordering:**

Each server block-entity tick first cycles display when `gameTime mod 20 == 0` and the captured
state is active, even if the same tick will deactivate. Only when `gameTime>=stateUpdatingResumesAt`
does the captured state compute one next state. A changed canonical state is offered with flags `3`;
its Boolean is ignored, then old-state exit and requested-state enter callbacks run regardless.
Finally any dirty server/shared data marks the block entity/chunk changed; shared dirtiness
additionally sends a flags-`2` block update using captured old state and computed next state, then
both dirty flags clear.

`inactive` and `active` scan at their due tick using strict block-position distance below activation
`4` or deactivation `4.5`, respectively. Creative is admitted, spectator is not, sight is
irrelevant, and already rewarded UUIDs are removed. The resulting set replaces connected players,
the next state becomes active iff nonempty, and the next scan is paused until `now+20`. Entering
active fills an empty display from the display/reward table then emits event `3015` with ominous
data; entering inactive clears display and emits `3016`. Thus activation/deactivation has
strict-radius hysteresis. `connected_particles_range` is never changed by server scans: it stays
saved/default `4.5`, even for a custom config radius.

Entering unlocking plays insert sound. At its resume tick it sets the next pause to `now+20` and
enters ejecting, whose enter callback plays open-shutter. On a due ejecting tick, a nonempty list
pops its **last** stack, dispenses it upward at speed `2` from `bottom-center+1.2Y`, emits event
`3017`, plays eject sound at pitch `0.8+0.4*progress`, replaces display with the new last stack,
pauses to `now+20`, and remains ejecting. Before that pop, `progress=1` when total is one, otherwise
`1-inverseLerp(currentSize,1,total)`, producing `0..1` across an uninterrupted list. A due empty
list zeroes total, rescans at deactivation radius and pauses 20; leaving ejecting plays
close-shutter before active/inactive enter effects. The shutter therefore opens 14 ticks after
accepted insertion, first ejects 20 ticks later, repeats every 20, and closes/transitions 20 ticks
after the last item.

**Use/key transaction:**

Empty hand or any nonactive state returns `TRY_WITH_EMPTY_HAND`. An active nonempty use returns
`SUCCESS_SERVER` on both sides; the client performs no key validation or mutation. The server
additionally requires a current `VaultBlockEntity`. It silently returns when config key is
empty/state inactive, then tests exact item **and components** plus sufficient count. Invalid stack
takes precedence over already-rewarded status and requests insert-fail sound; a valid rewarded
player requests reject sound. Failure sound occurs only at `gameTime>=lastFailure+15`, then stores
that time; because the transient default is zero, failures at times `0..14` are silent.

An admitted key evaluates the reward table first with vault context
`(origin=center, luck=player luck, this_entity=player, tool=the live inserted stack)`. Empty results
do nothing—no sound/stat/key/rewarded mark. A nonempty result awards that inserted item's used stat,
consumes the configured count unless the player has infinite materials, copies the ordered result
list into pending storage, records total, displays its last element, pauses state work to `now+14`,
offers unlocking and its sound, appends the UUID to rewarded players, then recomputes connections at
deactivation radius. Rewarded storage retains insertion order and at size `129` removes exactly its
oldest UUID, capping at `128`.

**Display and client:**

Every active server time divisible by 20 evaluates the override or reward table with vault
`(origin=center)` and the level RNG; from a nonempty result it then uses the level RNG once more to
select one stack uniformly, otherwise empty. Display equality suppresses dirtiness. Client spin
advances `10°` every client tick even for empty display. Each client tick has an inclusive
`nextFloat<=0.5` cage-position smoke attempt and adds normal/soul flame only when display is
nonempty; a displayed item separately has inclusive `nextFloat<=0.02` ambient sound with independent
volume/pitch floats. Every 20 client ticks, each synchronized connected UUID that resolves to a
player at squared block distance `<=particleRange²` emits `2..5` connection particles from facing
keyhole `(bottomCenter + facing*0.5 + 1.75Y)` toward player midheight with random directional
offset.

Event `3015` requires a current client vault: it emits those connection particles, then 20
smoke-plus-normal/soul-flame pairs at independent cage positions, then activation sound using two
pitch floats. Event `3016` emits 20 normal/soul flames from center-cage positions with Gaussian
velocities, then uses two pitch floats for deactivation sound. Event `3017` emits 20
small-flame/smoke pairs using shared positions/Gaussian vectors; the server's separate eject-sound
packet carries computed progress pitch. Unlock/open/close sounds are likewise server sound packets.
Light is `6` inactive and `12` in all other states.

**Persistence:**

Save writes config, shared and server data. Update packets contain only shared data. Server codec
includes rewarded players, resume time, pending items and total. On load, however,
`VaultServerData#set` copies only resume time, items and rewarded players: it leaves the receiving
object's previous total unchanged (normally zero on a fresh chunk load). Mid-ejection reload can
therefore compute progress with total zero, making ordinary remaining size `n` yield progress `n`
and pitch `0.8+0.4n`; loading into a reused object can retain another stale total. Last failure
time, dirty flags and client spin are transient. Shared load restores display/set/range; config load
restores serialized fields and default detector/selector.

**Branches and aborts:**

State/resume-time; connected set and strict hysteresis; config key empty; active/nonactive and
empty/nonempty hand; BE type; item/component/count; rewarded membership; fail-sound buffer; reward
empty; infinite materials; pending-list size; display override/result/equality; state write failure;
client UUID/range/display and every presentation draw.

**Constants and randomness:**

Scan/display client rates `20`; unlock delay `14`; ejection and after-last delays `20`; failure
buffer `15`; rewarded cap `128`; spin `10°`; light `6/12`; idle thresholds inclusive `0.5/0.02`;
activation/deactivation particle count `20`; connection count inclusive `2..5`. Loot/RNG consumption
and reverse list traversal are at the exact positions above.

**Side effects:**

Key stack/stat; rewarded/connected/pending/display/config data; dirty/save/update packets; block
state/light; loot evaluation; item entities; sounds, level events and particles; client spin.

**Gates:**

Loaded compatible block entity; normal gameplay and resume time; state and strict player
distance/mode; rewarded exclusion; exact key stack/components/count; nonempty loot; player
infinite-material ability; state write; shared-data synchronization and client player/range/display.
Difficulty and mob-spawning gamerules are not read.

**Boundary cases and quirks:**

Active uses are client-successful even when server validation silently/fail-sound rejects. The first
failure before game time 15 is silent. Empty reward loot does not consume a key or mark the player.
Ejection reverses the generated list. State transition effects run after a rejected/replaced state
write. Display can cycle immediately before deactivation. Particle cutoff is inclusive default `4.5`
and is decoupled from custom scan radii. Reload drops the decoded total through the setter bug and
changes later pitch. Reward eviction makes the oldest of 128 eligible again.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`net.minecraft.world.level.block.VaultBlock#useItemOn`, `#getTicker`,
`net.minecraft.world.level.block.entity.vault.VaultBlockEntity$Server#tick`, `#tryInsertKey`,
`#resolveItemsToEject`, `net.minecraft.world.level.block.entity.vault.VaultState#tickAndGetNext`,
`#onTransition`, `net.minecraft.world.level.block.entity.vault.VaultServerData#set`,
`#ejectionProgress`,
`net.minecraft.world.level.block.entity.vault.VaultSharedData#updateConnectedPlayersWithinRange`,
and `net.minecraft.client.renderer.LevelEventHandler#levelEvent`.

**Test vectors:**

Cross all four states at resume `-1/0/+1`; player radii just below/equal/above `4` and `4.5`,
creative/spectator/rewarded; empty/wrong/component-different/short/exact keys; world times
`0/14/15/29/30`; empty/one/many reward lists with survival/creative; rejected state writes; rewarded
sizes `128/129`; display table empty/one/many and active-deactivate same tick; exact
unlock/ejection/close times; save/reload fresh and reused mid-list; all client event/tick draw
counts and range equality. `EXP-BLK-007` owns the executable matrix.
