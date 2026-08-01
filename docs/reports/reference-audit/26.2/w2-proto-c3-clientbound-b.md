# Minecraft Java 26.2 Reference Audit — Wave 2, Worker 2: C3 Clientbound Projections B

## Result

The source-backed audit completed for the scope below. Its findings update reference documentation
only and do not change Ferrite implementation dispositions.

## Scope and evidence

- Baseline: `c5675bd7945981cbbfb120146c716abb130edaf8`
- Version lock: Minecraft Java `26.2`, server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`,
  client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`
- Runtime: Azul Java and `javap` 25
- Scope: protocol reference correction only. No Ferrite runtime, implementation disposition, packet
  catalog or shared conformance document was changed, and no implementation state was marked
  `Verified`.

## Audited families

- `PROTO-PLAY-CLIENTBOUND-SCOREBOARD-001`
- `PROTO-PLAY-CLIENTBOUND-SOUND-001`
- `PROTO-PLAY-CLIENTBOUND-TITLE-TAB-001`
- `PROTO-PLAY-CLIENTBOUND-WORLD-BORDER-001`
- `PROTO-PLAY-CLIENTBOUND-WORLD-EFFECT-001`

The audit followed each packet from its official codec through the client main-thread handler,
retained presentation state and authoritative server publisher. It separately tested malformed
decode boundaries, handler-time missing/invalid branches, prefix mutations, reload/level/connection
lifetime, cross-family ordering and non-round-trip reproduction vectors.

## Findings

1. The existing global level-event projection overstated direction placement. The client computes
   `camera + 2 * normalize(blockCenter - camera)`, but `Vec3#normalize` returns `Vec3.ZERO` below
   length `1.0e-5`; coincident and near-coincident targets therefore place the sound at the camera,
   not exactly two blocks away. This was corrected in the primary document and completion record.
2. A cross-level recipient of a server global level event receives
   `BlockPos.containing(player.position())`. This self position does not preserve the original event
   direction. The client aims from its camera toward that carried block's center. Only the far
   same-level projection approximates the event direction, and block flooring can perturb it. The
   primary document and completion record now distinguish the branches.
3. HUD expiry has asymmetric retained state. A positive title timer clears both title and subtitle
   on the decrement-to-zero tick; a title whose phase sum is initially nonpositive remains stored
   but invisible because the tick path never enters. Action-bar ticking reaches zero without
   clearing its stored overlay component. Clear/reset and later replacement remain the only packet
   transitions for those retained values.
4. IDs 116/117 seed the instance used for weighted sound resolution, but neither the packet nor the
   selected resource survives a sound-engine reload. `SoundEngine#reload` destroys the engine and
   clears current, queued, ticking, category-index and delayed-delete collections before rebuilding;
   a later stop packet can affect only post-reload instances. Missing entity/resource and silent
   entity paths still consume no durable retry or acknowledgement state.
5. Client scoreboard state belongs to `ClientPacketListener`, not `ClientLevel`. Respawn and
   dimension replacement within one play connection retain objective/team/score maps and make
   delayed packet names resolve against them. Reconnect constructs a new listener/empty scoreboard,
   after which the join snapshot reconstructs teams and displayed objectives. Client world-border
   state has the opposite lifetime: it belongs to the current level, and ID 43 initializes a
   replacement level rather than replaying IDs 88 through 92.
6. The remaining assigned wire grammars, strict/fallback mappings, publication audiences, seeded
   sound forms, team/objective partial-failure prefixes, border dirty/listener order, level-event
   local dispatch/RNG order and tokenless ordering matched the locked official jars and generated
   reports. No source-undetermined constant was promoted to fact.

## Official entry points inspected

Codec and mapping anchors included all assigned `Clientbound*Packet` classes plus
`SoundEvent#STREAM_CODEC`, `SoundSource`, `DisplaySlot`, `ObjectiveCriteria.RenderType`,
`NumberFormatTypes`, `Team.Visibility`, `Team.CollisionRule` and `TeamColor`.

Client application anchors included:

- `ClientPacketListener#handleAddObjective/#handleSetScore/#handleResetScore`,
  `#handleSetDisplayObjective/#handleSetPlayerTeamPacket`;
- `ClientPacketListener#handleSoundEvent/#handleSoundEntityEvent/#handleStopSoundEvent`,
  `ClientLevel#playSeededSound`, `SoundEngine#play/#stop/#stopAll/#reload`, and
  `EntityBoundSoundInstance#canPlaySound/#tick`;
- `ClientPacketListener#handleTitlesClear/#handleSetActionBarText/#handleSetTitleText`,
  `#handleSetSubtitleText/#handleSetTitlesAnimation/#handleTabListCustomisation`,
  `Hud#setOverlayMessage/#setTitle/#setSubtitle/#setTimes/#clearTitles/#resetTitleTimes/#tick`, and
  `ClientAdvancements#setSelectedTab`;
- the five border handlers, `WorldBorder`, `LevelEventHandler#levelEvent/#globalLevelEvent`, and
  `Vec3#normalize`.

Server publication anchors included `ServerScoreboard`, `PlayerList#updateEntireScoreboard`,
`TitleCommand`, `PlayerAdvancements#setSelectedTab`, `ServerLevel#playSeededSound/#levelEvent`,
`#globalLevelEvent`, `PlayerList#broadcast/#addWorldborderListener/#sendLevelInfo`, and the five
authoritative `WorldBorder` setters.

## Reproduction

Use only the locked jars and ignored report cache:

```sh
export MC_REF_JAVA="$JAVA_HOME/bin/java"
export MC_REF_JAVAP="$JAVA_HOME/bin/javap"

$MC_REF_JAVAP -classpath target/mc-reference/26.2/client.jar -p -c \
  net.minecraft.client.renderer.LevelEventHandler net.minecraft.world.phys.Vec3 \
  net.minecraft.client.gui.Hud net.minecraft.client.sounds.SoundEngine \
  net.minecraft.client.multiplayer.ClientPacketListener
$MC_REF_JAVAP -classpath target/mc-reference/26.2/server-26.2.jar -p -c \
  net.minecraft.server.level.ServerLevel net.minecraft.server.players.PlayerList \
  net.minecraft.server.ServerScoreboard net.minecraft.world.level.border.WorldBorder
```

Executable edge assertions, independent of encode/decode round trips:

- place the initialized camera exactly at, and less than `1.0e-5` from, the carried block center for
  global event 1023/1028/1038; the submitted sound position equals the camera rather than a
  two-block projection;
- publish a global event with a recipient in another level; its carried position is the floor of
  that recipient's position, independent of the event position, and the client direction is toward
  that carried block center;
- set title times to `0/0/0`, set subtitle/title, and tick; `titleTime` remains zero while both
  components remain stored; separately start a positive title and observe both components clear on
  its decrement-to-zero tick;
- set an action bar and tick 60 times; the timer reaches zero while the overlay component remains;
- play registered and direct positional/entity sounds with fixed seeds, reload resources, then apply
  each stop filter; pre-reload instances are absent and are not reconstructed;
- apply scoreboard packets, replace only `ClientLevel`, and observe retained listener scoreboard;
  construct a new listener and observe an empty scoreboard before join publication;
- start a border lerp, replace the level, and observe that only ID 43 initializes the replacement
  border; delayed IDs 88 through 92 then mutate only that replacement in receive order.

The aggregate official packet goldens remain the locked conformance rows
`C3-GOLD-CLIENTBOUND-SCOREBOARD`, `C3-GOLD-CLIENTBOUND-SOUND`, `C3-GOLD-CLIENTBOUND-TITLE-TAB`,
`C3-GOLD-CLIENTBOUND-WORLD-BORDER` and `C3-GOLD-CLIENTBOUND-LEVEL-EVENT`; the vectors above falsify
semantic claims without relying on round-trip symmetry.

## Integration notes

`protocol/conformance.md` was explicitly out of scope and was not edited. Its
`C3-LEVEL-EVENT-DISPATCH` expected result currently says every recognized initialized-camera global
sound shifts exactly two blocks. Integration should qualify that result with the `Vec3#normalize`
sub-`1.0e-5` zero-vector branch and should retain the cross-level self-position distinction. The
separate `PROTO-PLAY-CLIENTBOUND-BOSS-WAYPOINT-001` family owns boss-listener linked-map and
aggregate screen/music/fog state; the assigned scoreboard family has no protocol acknowledgement or
shared client collection with it, so no boss-family record was changed here.

## Unresolved items

None. All five assigned records retain `status = "Specified"` and `unknowns = []`. The source and
locked data determined the audited corrections; no implementation disposition is asserted.

## Evidence and verification

- `shasum target/mc-reference/26.2/server.jar target/mc-reference/26.2/client.jar` — passed; both
  SHA-1 values matched the repository lock shown above.
- `MC_REF_JAVA=... MC_REF_JAVAP=... cargo run -q -p mc-reference --bin mc-ref -- protocol inventory`
  — passed; 256 packets and digest `f34b0956b6399c749d4638cd6d3c9226685f41fa`.
- `MC_REF_JAVA=... MC_REF_JAVAP=... cargo run -q -p mc-reference --bin mc-ref -- protocol coverage`
  — passed; all 256 packets in 58 families, with 44 `Specified` and 14 `GatedOptional`.
- `MC_REF_JAVA=... MC_REF_JAVAP=... cargo run -q -p mc-reference --bin mc-ref -- protocol readiness`
  — passed; inventory and coverage repeated successfully and protocol readiness completed.
- `MC_REF_JAVA=... MC_REF_JAVAP=... cargo run -q -p mc-reference --bin mc-ref -- protocol verify` —
  passed in offline mode; the inventory, coverage and existing runtime packet catalog were verified.
- `MC_REF_JAVA=... MC_REF_JAVAP=... cargo run -q -p mc-reference --bin mc-ref -- verify --offline` —
  passed; 417 documentation IDs, 331 completion slices, 2,798 source locators, 9,078 locked catalog
  IDs, 307 experiment definitions and all protocol, command-root, join, behavior-surface and
  implementation-manifest consistency checks completed.
- `git diff --check` — passed.

These are documentation-only changes, so the `AGENTS.md` exception applies and Rust formatting,
Clippy and runtime crate tests are not required. The successful reference-specific commands compiled
and exercised `mc-reference` itself.
