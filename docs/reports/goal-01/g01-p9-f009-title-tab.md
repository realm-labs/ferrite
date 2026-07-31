# G01-P9-F009 Title and Tab Report

## Result

Ferrite implements and verifies clientbound IDs 14, 85, 87, 112, 114, 115, and 122 in
`PROTO-PLAY-CLIENTBOUND-TITLE-TAB-001`. Normalized title, action-bar, player-list decoration, and
advancement-selection intent form the authoritative boundary; client timers, holder identities,
flattened strings, and HUD widgets remain adapter-local.

## Verified boundaries

- All seven packet bodies and IDs are locked. The clear and nullable-selection booleans normalize
  nonzero bytes; advancement identifiers retain the default bound; trusted component NBT uses the
  shared frame and depth limits; animation fields retain every signed integer value. Invalid NBT,
  identifiers, truncation, and residual bytes fail before projection.
- Action-bar text replaces even with an empty component, restarts 60 client ticks, and disables
  animated color without touching title state. Title/subtitle replacement is independent, and
  title activation uses the wrapping signed sum of the current 10/70/20-derived durations.
- Animation fields replace independently only when nonnegative and restart only an already-active
  title. Clear always removes title, subtitle, and remaining time; its flag alone decides whether
  durations reset, while action-bar state remains untouched. Timer expiry has no response.
- Advancement correction resolves against the handler-time tree, selects null for missing IDs,
  compares exact holder-object identity for notification, accepts known child holders on the
  client, and emits no opened-tab echo. Server publication retains only displayed roots and sends
  only selection identity changes.
- Header and footer flatten independently. Empty rendered text clears that field while nonempty
  rendered text retains the original styled component; neither field changes player entries or
  list state.
- Clear/timing publication preserves selected-player order and reuses one immutable packet.
  Text commands resolve per target and send immediately, so a resolution failure retains the
  already-sent prefix and prevents later targets. No family packet has a sequence, generation,
  retry, acknowledgement, dimension, or distance gate.

## Evidence

- `crates/ferrite-protocol/src/java_26_2/play/clientbound/title_tab/`
- `crates/ferrite-protocol/tests/c3/play_clientbound_title_tab.rs`

Focused validation:

```text
cargo test -p ferrite-protocol --test c3 play_clientbound_title_tab --all-features
12 passed; 0 failed
cargo test -p ferrite-protocol --test c3 --all-features
265 passed; 0 failed
cargo test -p ferrite-protocol --test c1 --all-features
68 passed; 0 failed
cargo clippy -p ferrite-protocol --all-targets --all-features -- -D warnings
```

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
