# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-JUKEBOX-001` — Jukebox item, song-clock, signal and client playback state are distinct

**Parent:** `ITM-001`, `ITM-003`, `PLY-005`, `SIM-003`, `BLK-003`, `RED-001`, `RED-003`, `ENT-001`,
`CLI-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked server/client source fixes insertion/ejection, one-slot automation,
song-clock cadence, persistence, strong/comparator outputs, removal, level/game events, particles
and client playback/parrot notification. The 22 song records and 22 default playable components are
audited locked data; generic item-entity admission and sound-engine resource/budget behavior remain
under their generic owners.

**Applies when:**

A `minecraft:jukebox` is placed from item data, used, accessed as a one-slot container, ticked,
sampled for redstone, saved/loaded, replaced, or receives a playable item whose
`minecraft:jukebox_playable` component selects a song record.

**Authoritative state:**

The block has only boolean `HAS_RECORD`, default false, for two states. The block entity separately
owns one unconstrained `ItemStack`; its song player owns nullable song holder plus signed 64-bit
`ticksSinceSongStarted`; each client separately maps block position to at most one playing sound
instance. The block is dirt-map-color, bass-instrument, wood-sound, lava-ignitable, hardness 2 and
explosion resistance 6. Placement sets `HAS_RECORD=true` with flags 2 whenever typed block-entity
item data merely contains `RecordItem`, independent of its decoded item/playability; absence leaves
the placed state unchanged.

**Transition and ordering:**

With captured `HAS_RECORD=true`, item use returns `TRY_WITH_EMPTY_HAND`, and empty-hand use returns
`SUCCESS` after asking a matching block entity to eject or `PASS` for a wrong subtype. With captured
false, the helper re-reads the live state and requires a `JUKEBOX_PLAYABLE` component, the jukebox
block and live false state; rejection reaches empty-hand fallback. Admission returns `SUCCESS` on
both sides. Server admission consumes and returns one item using player abilities before looking up
the block entity. A matching subtype receives the stack, updates `HAS_RECORD` with flags 2 and emits
an unsourced `BLOCK_CHANGE`, starts the song with tick zero, emits level event 1010 carrying the
runtime song-registry ID, updates neighbors/dirty state, then the caller emits a second
player-context `BLOCK_CHANGE` using the pre-insert state. `PLAY_RECORD` is awarded even when the
subtype is wrong, so that branch loses/duplicates the consumed item by abilities, changes no jukebox
state and still succeeds/stat-awards.

**Song clock and signals:**

A ticker exists only while the block state says `HAS_RECORD=true`. `play` replaces any prior song
without a stop event, resets tick to zero, emits 1010 and updates neighbors. Each active tick first
stops when `ticks >= ceil(lengthSeconds*20)+20`; otherwise every tick value divisible by 20 emits
`JUKEBOX_PLAY`, and on a server consumes `nextInt(4)` to send one NOTE particle request at block
bottom-center plus Y 1.2 with the selected value divided by 24 as its X parameter, then increments
the counter. Stop nulls the song, resets the counter, emits `JUKEBOX_STOP_PLAY`, level event 1011,
neighbor update and dirtying. Weak/strong block-source output is 15 exactly while the song holder is
nonnull and zero otherwise. Comparator output ignores playing/state and reads the current item's
resolved song record, returning its data-defined 0..15 output or zero.

**One-slot container and ejection:**

Size/max are one. `canPlaceItem` requires an empty slot and a playable component; `canTakeItem`
requires the destination to contain any empty slot and ignores compatible merges/placement policy.
Public `setItem(0,stack)` delegates without validating or clamping: any nonempty item makes
`HAS_RECORD=true`; a playable starts/replaces a song, while an invalid item stops the old song or
remains silent. Public removal and no-update removal both call the override that ignores requested
count, returns the entire stack and sets empty, so they update state and stop playback. Ejection is
server-only and does nothing when the actual item is empty. Otherwise it removes first, then
consumes two level floats to place a copied stack at
`(x+0.5+(f-0.5)*0.7, y+1.01, z+0.5+(f-0.5)*0.7)`, creates an item entity with pickup delay 10,
attempts admission without checking the result, and invokes a second neighbor/dirty update.

**Automation boundaries:**

Inbound hopper transfer uses the empty/playable preflight then starts the song through public set.
Outbound extraction empties/stops before destination insertion. A failed normal one-item transfer is
restored through public set, so it emits stop then a fresh 1010 and restarts at tick zero. A
malformed overstack is removed whole; on failed transfer the generic one-count-only rollback does
not reinsert it. Preflight rejects a full destination even when a compatible partial stack could
merge, but admits any empty destination slot without checking its later placement policy.

**Persistence and divergence:**

Save writes `RecordItem` only when nonempty and writes `ticks_since_song_started` only while a song
holder exists. Load replaces the item without calling the ordinary state notifier. A different item
stops an existing song first; a same item can preserve it. When the tick field is present and the
item resolves, `setSongWithoutPlaying` arms that song/counter only if it has not reached the padded
finish threshold, and emits neither 1010 nor a neighbor update. Initial load of an item without the
tick field therefore leaves it unarmed. An armed song with `HAS_RECORD=false` never receives a
ticker; `HAS_RECORD=true` with an invalid/unarmed item ticks as a no-op; comparator, source signal
and occupancy may all disagree. Reloading the same currently playing item with an already-finished
tick value returns without clearing the existing song/counter.

**Removal and block loot:**

On ordinary block replacement, the old block state has already been replaced when pre-removal runs.
The override ejects the actual item but its state-notify identity guard therefore aborts; removing
the item still stops an active song, emits stop game/level events and updates neighbors, then spawns
the item. Subsequent `setRemoved` unconditionally emits another `JUKEBOX_STOP_PLAY` and 1011 even
when already stopped or empty. Flag 256 suppresses pre-removal ejection but not the unconditional
final stop pair, so the stored item is lost. Jukebox block loot independently yields one block item
when the generic explosion-survival condition passes and copies no record.

**Client presentation:**

Level event 1010 resolves the numeric song ID; a missing ID does nothing. A valid ID first stops the
mapped sound at that position without a false notification, then starts a nonlooping `RECORDS` sound
at block center with volume 4, pitch 1 and linear attenuation, updates the now-playing HUD, and
calls `setRecordPlayingNearby(pos,true)` on living entities in the block AABB inflated 3. Event 1011
stops the mapped instance and sends false to the then-nearby set. Only concrete overrides such as
parrots react; a parrot self-clears when its jukebox disappears or its center distance reaches 3.46.
Loaded/armed server song state does not replay 1010, so joining/reloading clients do not infer
playback from block/entity synchronization.

**Song and item data:**

The exact `id:lengthSeconds/comparator` records are `11:71/11`, `13:178/1`, `5:178/15`,
`blocks:345/3`, `bounce:234/8`, `cat:185/2`, `chirp:185/4`, `creator:176/12`,
`creator_music_box:73/11`, `far:174/5`, `lava_chicken:134/9`, `mall:197/6`, `mellohi:96/7`,
`otherside:195/14`, `pigstep:149/13`, `precipice:299/13`, `relic:218/14`, `stal:150/8`,
`strad:188/9`, `tears:175/10`, `wait:238/12`, and `ward:251/10`; each selects the same-suffix
`minecraft:music_disc.<id>` sound. The corresponding `music_disc_<id>` item has max stack one and a
default playable component pointing to that record; custom component-bearing stacks follow the
component, not their item ID. Tooltips append the gray song description.

**Branches and aborts:**

Captured true/actual empty produces successful ghost ejection with no repair. Captured false/actual
occupied can overwrite and lose the old item. Client admission predicts success without consuming.
Wrong subtype loses the server-consumed item but still awards the stat. Invalid direct items create
occupied/silent state. Failed state/event/entity admission is not rolled back. Replacing a playing
song directly sends a new play event but no preceding stop game/level event.

**Constants and randomness:**

State flags 2, song padding 20 ticks, playing-event interval 20, source output 15, ejection Y
1.01/XZ width 0.7, pickup delay 10, particle Y 1.2/four values divided by 24, client volume/pitch
4/1, client notification inflation 3 and parrot center distance 3.46 are fixed. Ejection consumes
two level floats plus item-entity-private construction randomness; each 20-tick server emission
consumes one bounded integer. Sound-instance private RNG/resource scheduling is presentation-owned.

**Side effects:**

Hand count/stat; item/song/counter; block state and dirty state; neighbor/comparator/source updates;
two insertion block-change events; periodic play and stop game events; level events; note particles;
item entity; client sound/HUD/parrot state; persistence and block/item loot.

**Gates:**

Captured/live occupancy, playable component, client/server, block/subtype identity, ticker-bearing
state, padded finish and modulo-20 cadence, destination emptiness, raw versus public load/set,
pre-removal flag 256, explosion condition, song-ID resolution and entity admission.

**Boundary cases and quirks:**

Occupancy is not item truth, item truth is not playing truth, and comparator truth is not
source-signal truth. Normal failed hopper rollback restarts the song. A song stays server-active for
20 padded ticks beyond its rounded-up nominal duration. Empty removal still emits a final stop pair,
while populated ordinary removal emits two stop pairs. Persistence can arm server signals without
ever starting client audio.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`, `OFF-REPORT-001`;
`net.minecraft.world.level.block.JukeboxBlock#setPlacedBy`,
`net.minecraft.world.level.block.JukeboxBlock#useItemOn`,
`net.minecraft.world.level.block.JukeboxBlock#useWithoutItem`,
`net.minecraft.world.level.block.JukeboxBlock#ownSignal`,
`net.minecraft.world.level.block.JukeboxBlock#getAnalogOutputSignal`,
`net.minecraft.world.level.block.JukeboxBlock#getTicker`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#setTheItem`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#popOutTheItem`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#loadAdditional`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#saveAdditional`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#setRemoved`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#preRemoveSideEffects`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#canPlaceItem`,
`net.minecraft.world.level.block.entity.JukeboxBlockEntity#canTakeItem`,
`net.minecraft.world.item.JukeboxPlayable#tryInsertIntoJukebox`,
`net.minecraft.world.item.JukeboxSongPlayer#setSongWithoutPlaying`,
`net.minecraft.world.item.JukeboxSongPlayer#play`,
`net.minecraft.world.item.JukeboxSongPlayer#stop`,
`net.minecraft.world.item.JukeboxSongPlayer#tick`,
`net.minecraft.world.item.JukeboxSong#hasFinished`,
`net.minecraft.client.renderer.LevelEventHandler#playJukeboxSong`,
`net.minecraft.client.renderer.LevelEventHandler#stopJukeboxSongAndNotifyNearby`,
`net.minecraft.client.resources.sounds.SimpleSoundInstance#forJukeboxSong`,
`net.minecraft.world.entity.animal.parrot.Parrot#setRecordPlayingNearby`;
`data/minecraft/jukebox_song/*.json`, `data/minecraft/loot_table/blocks/jukebox.json`, item
component reports and registry/state membership; `EXP-ITM-011`.

**Test vectors:**

Both states crossed with empty/playable/invalid/overstack actual items and matching/wrong subtype;
all 22 defaults plus custom component-bearing items; survival/infinite-material/client insertion;
same-song/replacement direct set; tick 0/19/20 and every finish boundary; comparator/source
combinations; inbound/outbound hopper success/failure/overstack; load with
absent/negative/valid/finished tick and same/different item under both states;
ordinary/flag-256/explosion removal with empty/populated/playing item and rejected entity admission;
client missing/valid 1010, replacement 1010 and repeated 1011 with parrots entering/leaving the
notification bounds.
