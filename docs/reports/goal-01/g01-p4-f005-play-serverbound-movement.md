# G01-P4-F005 Play Serverbound Movement Report

## Result

Ferrite implements all 15 packets in `PROTO-PLAY-SERVERBOUND-MOVEMENT-001`. Five missing codecs
were added, Play client-information now shares the Configuration body codec, and bounded
connection-local control, chunk-flow, load-gate, command, and vehicle projections complement the
existing authoritative player movement and Region transfer path.

## Verified boundaries

- All 15 locked IDs round trip; enum, UTF, float, Boolean, flag, truncation, and trailing-byte
  boundaries fail or normalize as specified.
- Four player forms, exceptional values, cadence, speed, collision, teleport/load gates, floating,
  tick-end known movement, correction, and Region transfer retain existing focused evidence.
- Input, abilities, paddles, all commands, ignored entity/data fields, hat transitions, 60-tick
  readiness, and NaN/infinite chunk feedback have explicit state tests.
- Vehicle input covers invalid-before-gate ordering, control qualification, speed correction,
  singleplayer exemption, collision residual policy, rotation wrapping, clamp, and floating state.
- Keepalive and pong retain disjoint exact echo domains.

The batch acceptance gate is `cargo ferrite task check` followed by `git diff --check`.
