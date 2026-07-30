# G01-P6-F005 Play Clientbound Special-Screens Report

## Result

Ferrite implements and verifies IDs 41, 58, and 60 in
`PROTO-PLAY-CLIENTBOUND-SPECIAL-SCREENS-001`. Screen state remains an ephemeral client projection;
mount inventory, book components, and sign authority continue through their existing owners.

## Verified boundaries

- Three goldens lock signed VarInts, fixed entity int, strict hand ordinal, packed position, and
  boolean order; invalid hands, truncation, residual bytes, and noncanonical booleans are covered.
- Mount handling performs wrapping columns-times-three allocation admission before entity lookup,
  then distinguishes horse cargo, nautilus no-cargo, wrong, and missing tracked entities.
- Specialized mount publication shares the ordinary container counter and synchronization path,
  replacing only the generic open-screen packet.
- Book handling reads the current selected hand at execution, prefers written content, supports
  writable forged activation, and applies filtered/raw page selection without snapshot tokens.
- Canonical book publication emits menu convergence before activation when resolution changes the
  stack and sends nothing without written content.
- Sign handling resolves current ordinary/hanging projection and selected side; server admission
  stores editor authority and orders block correction before editor activation.
- A three-packet end-to-end trace decodes into the prepared mount, book, and sign client state with
  no acknowledgement.

## Evidence

- `crates/ferrite-protocol/tests/c3/play_clientbound_special_screens.rs`
- `docs/development/protocol-play-clientbound-special-screens.md`

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
