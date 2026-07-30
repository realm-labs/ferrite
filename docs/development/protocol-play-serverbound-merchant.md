# Play Serverbound Merchant Selection

`G01-P6-F009` implements ID 51, `minecraft:select_trade`, from
`PROTO-PLAY-SERVERBOUND-MERCHANT-001`. Its body is one unrestricted signed VarInt selection hint.
It has no container ID, state ID, payment stack, offer digest, boolean, sequence or acknowledgement.
Malformed/truncated VarInts and residual packet data fault decoding.

## Local prediction and admission

The client adds the visible offer-button index to its scroll offset, stores that hint, recomputes
the result and runs payment return/auto-fill locally before sending ID 51. This is prediction only.

The server accepts the packet only when the handler-time current menu is a still-valid merchant
menu. It does not reset idle or gate on loaded, spectator, death or a packet container ID. An
admitted request stores every signed hint and recomputes the result before auto-fill checks whether
the hint is a real list index.

## Selection lookup

Offer lookup deliberately treats hints asymmetrically:

- a hint strictly greater than zero and below offer count tests only that offer;
- zero, every negative hint and every hint at or beyond offer count scan from offer zero and choose
  the first payment match.

A missing or out-of-stock match retries with the two payments swapped. A usable offer installs a
copied result and its signed future XP. With a nonempty offer list, no match clears result and XP
and still notifies the merchant. Entirely empty input clears without notification. Nonempty input
against an empty offer list clears only the active offer, retains stale result/XP and notifies with
that retained result.

## Payment return and auto-fill

Only an in-range hint, including zero, enters payment movement:

1. return payment zero into player-menu slots `3..39` in reverse merge order;
2. stop if that nonempty payment moved nothing;
3. return payment one the same way and stop if it moved nothing;
4. continue only after both payments are empty;
5. scan player inventory ascending and fill exact selected-offer costs.

Returns are intentionally non-atomic and may be partial. A second-slot failure can follow a partial
or complete first return. Fill requires the item and every cost predicate component, permits extra
candidate components, and requires full item/component equality when merging into a nonempty
payment. It moves up to the source item's maximum stack size, not merely the required cost count.
Each payment write recomputes the result.

The selection operation never consumes payment or increments offer uses. Actual result clicks stay
inside ordinary container click/convergence behavior.

## Convergence and ownership

The handler emits no direct response and does not explicitly broadcast. Ordinary container
slot/cursor/data detection publishes only authoritative differences. A matching prediction may
produce no packet; inventory races, changed offers and partial capacity converge through ordinary
container deltas.

The signed hint, offer-list order, player/payment/result slots and prediction trace are
connection/menu-local. Normalized offer semantics and authoritative inventory remain with gameplay
and Region owners. ID 51 and clientbound offer ID 52 do not acknowledge each other.

## Evidence

`crates/ferrite-protocol/tests/c3/play_serverbound_merchant.rs` owns the golden and malformed forms,
current-menu admission, hint-zero/forced/invalid lookup, swapped and out-of-stock results,
empty-input/empty-offer quirks, exact component fill, source-maximum movement, non-atomic returns,
local-first ordering and end-to-end tokenless convergence.
