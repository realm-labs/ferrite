# G01-P9-F008 Sound Report

## Result

Ferrite implements and verifies clientbound IDs 116, 117, and 119 in
`PROTO-PLAY-CLIENTBOUND-SOUND-001`. Stable sound identities and authored events form the
authoritative boundary; connection-local registry IDs, wire coordinate quantization, selected
audiences, and active client sound instances remain adapter-local.

## Verified boundaries

- Registered sound holders use positive `raw ID + 1`; zero selects a direct identity with an
  optional fixed range. All eleven source IDs are strict, while signed entity IDs, signed fixed
  coordinates, IEEE volume/pitch/range values, and signed seeds retain their complete wire domains.
- Positional publication reproduces Java's `double * 8` saturating integer conversion. Client
  projection reproduces the subsequent integer-to-float division, including low-bit loss at large
  coordinates; duplicate packets create duplicate live instances.
- Entity sounds resolve the entity once at handler time. Missing entities are lost without retry,
  silent entities do not start playback, and accepted instances bind to the exact object identity,
  follow float-rounded positions, and stop on removal or same-ID object replacement.
- `stop_sound` reads only flag bits 0 and 1, ignores higher bits, and canonicalizes on encode. Its
  absent/source/sound/source-and-sound forms filter only current instances; later matching sounds
  are not suppressed.
- Server publication preserves viewer list order, excludes the exact source player, requires the
  same dimension, and applies a strict squared-distance comparison. Fixed ranges remain unchanged;
  otherwise range is 16 except that volume above one scales it by 16. Negative, NaN, and infinite
  values retain the locked arithmetic behavior.
- Positional and entity broadcasts use their respective authored/current positions for audience
  selection. Stop publication targets its selected player list directly without dimension or
  distance filtering. The family has no acknowledgement, sequence, or generation token.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/sound/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_sound.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_sound --all-features
12 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
253 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
