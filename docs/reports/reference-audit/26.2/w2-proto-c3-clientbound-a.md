# Minecraft Java 26.2 reference audit — C3 clientbound projections A

## Scope and provenance

- Worktree: `/Users/mikai/CLionProjects/ferrite-worktrees/w1-jigsaw-engine`
- Branch: `codex/ref-proto-c3-clientbound-a`
- Baseline: `c5675bd7945981cbbfb120146c716abb130edaf8`
- Protocol: `776`; world version: `4903`
- Locked client SHA-1: `2dc72797acbc1b63fc16a11c4ac393605f453754`
- Locked server SHA-1: `86765a5899bd9c96461036a628796b4245715058`
- Families: `PROTO-PLAY-CLIENTBOUND-BOSS-WAYPOINT-001`,
  `PROTO-PLAY-CLIENTBOUND-CHAT-PRESENTATION-001`,
  `PROTO-PLAY-CLIENTBOUND-COMPLETIONS-001`, `PROTO-PLAY-CLIENTBOUND-PARTICLE-001`,
  `PROTO-PLAY-CLIENTBOUND-PLAYER-INFO-REMOVE-001` and
  `PROTO-PLAY-CLIENTBOUND-PLAYER-PROJECTION-001`

Evidence was restricted to the locked official client/server jars, generated reports, repository
documentation and `mc-ref`. No Ferrite runtime code, implementation disposition, runtime packet
catalog or shared conformance document was changed.

## Falsification coverage

- Re-read every assigned packet codec and nested codec, including enum strategy, identifiers,
  trusted components, list/map counts, signed values, registry dispatch and residual-byte faults.
- Followed main-thread client handlers through keyed collection mutation, chat cache/validator and
  delay state, suggestion correlation, particle sampling/admission, local-player state replacement
  and social/listed-player teardown.
- Followed canonical server publishers through boss and waypoint deltas, asynchronous suggestions,
  player disconnect, per-connection chat packing, particle audiences, cooldown lifecycle, vitals,
  experience and dirty-stat drains.
- Checked cross-family handoffs among player info, chat validation, online-name completion and
  entity removal without assigning a generation or acknowledgement where none exists.
- Reconfirmed the generated play-clientbound packet IDs and the configured 125-entry particle
  registry against the locked reports.

## Material findings and corrections

1. `MessageSignatureCache#push` does not de-duplicate the newly queued body-last-seen signatures
   and packet signature. It installs the queue from tail to head exactly, so repeated new
   signatures can occupy repeated slots. Its temporary set only prevents an old cache entry from
   surviving when the same signature occurs in the new queue. The prior document incorrectly
   described new signatures as de-duplicated.
2. A canonical zero-duration cooldown start inserts a zero-width server entry and immediately
   publishes duration zero, which removes the client projection. The next server cooldown tick
   expires that entry and publishes a second zero. Negative starts similarly publish before their
   next-tick expiry. This mutation/output prefix was previously implicit.
3. A negative particle count makes no `ClientLevel#addParticle` call and therefore consumes neither
   Gaussian nor particle-setting limiter RNG. Every positive attempt consumes six Gaussians before
   client admission, so distance/settings rejection preserves that six-draw prefix.
4. Player-info removal immediately reduces later online-name completion queries, but it neither
   cancels the current command-suggestion future nor resets its signed transaction ID or custom
   completion set. A matching response already in flight can still complete with stale names.
5. Waypoint operation decoding uses signed floor-modulo three. The negative boundary is explicitly
   `-3=track`, `-2=untrack`, `-1=update`; the nested waypoint representation enum remains strict.

## Confirmed behavior

- Packet IDs remain boss `9`, suggestions `15`, cooldown `22`, custom completions `23`, delete chat
  `31`, disguised chat `33`, particles `47`, player chat `65`, player-info remove `69`, experience
  `103`, health `104`, system chat `121` and waypoint `138`.
- Boss operations/styles are strict and keyed by UUID; waypoint operations wrap but location types
  are strict, track replaces complete state, update mutates same-type contents only and untrack keys
  only by identifier.
- Command suggestions convert before latest-ID correlation; the official server preserves the
  original range and transaction while truncating only lists above 1,000 entries.
- Player-info removal retains independently owned entity, chat history, social preference,
  scoreboard/team and waypoint state. Canonical leave publication follows authoritative removal.
- Chat global-index mismatch precedes body unpack/cache mutation; successful unpack precedes cache
  push, sender lookup and validation. Delete-chat retains its cache, pending, delay and HUD order.
- Health/food saturation-zero markers, experience duplication around respawn, typed stat drain and
  cooldown signed wrapping matched the documented official paths.
- Particle count-zero float multiplication, positive six-Gaussian sampling, 32 override types,
  limiter ordering and strict 32/512-block server audiences matched the locked sources.

## Independent reproduction vectors

- Seed a 128-slot signature cache, decode a player-chat body with repeated `[A, A, B]` last-seen
  entries and packet signature `A`, then assert tail-first repeated slots and suppression only of
  pre-existing cache entries matching `{A, B}`. Repeat with unknown sender and validator failure to
  retain the cache-mutation prefix.
- Start group `g` at durations `0`, `-1`, `1`, and values wrapping `tickCount + duration`; record
  server map state and exact ID-22 sequence across the next tick, then replay delayed zero against a
  newer client cooldown.
- Replay particle counts `-1`, `0`, and `1` under ALL/DECREASED/MINIMAL, always-show and override
  combinations with a counting random source. Assert zero draws for `-1`, limiter-only draws for
  `0`, and the six-Gaussian prefix before limiter draws for `1`, including rejected attempts.
- Issue suggestion transaction `n`, remove the supplying player info, then deliver matching and
  stale results while querying custom/player unions. Assert future correlation is unchanged and
  only later online-name queries lose the removed name.
- Decode waypoint operations `INT_MIN`, `-3..=3`, and `INT_MAX`, then cross track/update/untrack
  against missing, same-type and mismatched targets and icon changes.
- Retain existing official-codec goldens for all thirteen assigned packet identities and add
  malformed counts, strict registry/enum failures, nonfinite numeric values and illegal ordering as
  independent negative vectors rather than relying on round trips.

## Unresolved gates and integration notes

No new source-inconclusive gate was found. Cryptographic chat acceptance still requires fixtures
with valid and invalid signed profile/session material for executable conformance, but its branch
behavior is source-determined. No shared-file correction is required for integration.

## Verification

The exact checks were:

```sh
export MC_REF_JAVA=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/java
export MC_REF_JAVAP=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home/bin/javap
./target/debug/mc-ref protocol inventory
./target/debug/mc-ref protocol coverage
./target/debug/mc-ref protocol readiness
./target/debug/mc-ref protocol verify
./target/debug/mc-ref verify --offline
git diff --check
```

All passed. Inventory verified 256 packets, including 141 play-clientbound packets, with digest
`f34b0956b6399c749d4638cd6d3c9226685f41fa`. Coverage verified 58 families with 44 `Specified`
and 14 `GatedOptional`; protocol readiness and offline protocol verification completed. The full
offline verifier checked 417 documentation IDs, 2,798 symbol locators across 952 classes, 9,078
locked registry IDs with zero gaps, 307 experiment definitions, protocol, command roots,
cross-system joins, behavior surfaces and existing implementation-manifest consistency.

Rust formatting, Clippy and crate tests were not run because this audit changes protocol reference
documentation and completion metadata only.
