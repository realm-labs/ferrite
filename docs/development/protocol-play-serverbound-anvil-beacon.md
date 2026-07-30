# Play Serverbound Anvil and Beacon Protocol

`G01-P6-F006` implements both packets in
`PROTO-PLAY-SERVERBOUND-ANVIL-BEACON-001` for Minecraft Java 26.2:

| ID | Identity | Fields |
|---:|---|---|
| 48 | `minecraft:rename_item` | one default-bounded UTF string |
| 52 | `minecraft:set_beacon` | independent optional primary and secondary mob-effect holders |

Rename uses the common 32,767-UTF-16-unit and 98,301-byte bound. Beacon optionals each use a
presence boolean followed by a strict raw VarInt in the configured 40-entry `minecraft:mob_effect`
registry. Nonzero presence bytes decode true and canonical encoding writes zero or one. Unknown
raw IDs, missing present values, malformed VarInts, excess UTF and residual bytes fault.

The established no-context codec entry remains available to C1/C2 callers. The strict
`decode_packet_with_registries` and `encode_packet_with_registries` entries are required for
beacon traffic. `ServerConnection` uses the immutable `play_registries` supplied in its settings,
so real framed ingress follows the strict path rather than accepting a process-local raw ID.

## Anvil prediction and admission

`AnvilClientProjection` models the 50-UTF-16-unit edit box and current slot-zero stack. Missing input
does nothing. A default hover name without `CUSTOM_NAME` normalizes to the empty proposal. Every
effective local `setItemName` filters and predicts result/custom-name presentation before producing
ID 48; edits collapsing to the current filtered name produce no packet. Escape remains ordinary
container close and has no final rename submission.

Server admission resolves the handler-time current menu and silently ignores a wrong or invalid
menu. It removes U+00A7, every UTF-16 unit below U+0020 and U+007F before applying the 50-unit
semantic bound and equality test. A changed name updates menu-local state, maps Java blank strings
to absent custom name, reruns the owned anvil computation and records ordinary broadcast
convergence. Filtering therefore allows an over-50 wire value to shrink into range and can make
distinct requests equivalent. Result persistence remains with the authoritative result-take
transaction.

## Beacon selection and admission

The client projection admits tier buttons from synchronized level state. Selecting a different
primary clears secondary; the upgrade button copies primary. Done is enabled only with payment and
primary, then emits ID 52 followed by ordinary close ID 19. It does not predict payment or
block-entity mutation. Cancel and Escape emit close only.

Server admission first resolves a current, still-valid beacon menu, then requires only a nonempty
payment. The exact tier mapping is speed/haste 1, resistance/jump boost 2, strength 3,
regeneration 4 and `i32::MAX` for every other codec-valid effect. Secondary requires level four;
primary must be below tier four; tier-one-through-three secondary must equal primary.

This preserves forged boundaries: both choices absent can succeed, absent primary plus
regeneration secondary succeeds at level four, while absent primary plus a tier-one-through-three
secondary reaches the source null-equality fault. A controlled validation refusal or missing
payment requests the generic disconnect without first publishing a correction.

Success writes primary then secondary as built-in effect raw ID plus one, with zero reserved for
absence. This menu-data domain is kept separate from the packet optional grammar. Block-entity
state retains only the six beacon effects, the primary write plays the selection sound when beam
sections exist, one payment is consumed, and the chunk becomes unsaved. Canonical close then
returns only remaining payment; no handler-local broadcast or acknowledgement is introduced.

## Ownership

Raw packet IDs, optional booleans, configured registry indices, built-in-plus-one values and local
GUI choices stay in the 26.2 adapter. Current-menu admission produces normalized anvil/beacon
transactions; ordinary container convergence, anvil result computation, Region-owned block-entity
state, persistence and disconnect publication retain their existing owners.

## Evidence

`crates/ferrite-protocol/tests/c3/play_serverbound_anvil_beacon.rs` owns both goldens, UTF and
registry codec faults, client rename prediction, server filtering and broadcast, effect/data-domain
mapping, every payment/tier/null-equality boundary, send-before-close order and end-to-end decoded
menu transactions.
