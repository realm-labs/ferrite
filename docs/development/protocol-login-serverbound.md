# Required Login Serverbound Protocol

Ferrite's required offline-mode login implementation covers the two serverbound packets owned by
the locked C1 family:

| Identity | Locked ID | Payload |
|---|---:|---|
| `minecraft:hello` | `0` | player name `UTF(16)`, then supplied profile UUID |
| `minecraft:login_acknowledged` | `3` | fieldless terminal acknowledgement |

Catalog-recognized IDs `1`, `2`, and `4` are optional encryption, custom-query, and cookie paths.
The required decoder rejects them until their explicit C4 gates are implemented. IDs outside the
five-entry login lane also fail closed.

## Offline identity and admission

The packet codec admits zero through 16 UTF-16 name units. The listener separately accepts only
units strictly above ASCII space and below DEL; consequently the empty name is valid at this
boundary, while whitespace, control characters, DEL, and non-ASCII units are refused.

Offline mode ignores the supplied UUID. Ferrite computes the authoritative UUID by taking MD5 over
the UTF-8 bytes of `OfflinePlayer:` plus the exact case-sensitive name, then applies RFC 4122
version-3 and variant bits. The normalized profile starts with no properties. An optional intended
connection UUID must match that normalized UUID.

Admission receives an explicit connection-local snapshot after normalization. A policy component is
sent as login disconnect before closure. If a matching player is already active, Ferrite requests
that connection's disconnection once and keeps this login waiting until the duplicate disappears.
No packet field or supplied UUID crosses into gameplay persistence or ECS state.

## Compression and terminal transition

The dedicated default threshold is 256. A negative threshold or memory connection skips
negotiation. Otherwise Ferrite emits login compression with the old uncompressed envelope and
enters a callback-pending stage. Only the send-completion callback installs compression in both
directions and produces login finished. Threshold zero is valid.

The listener enters `ProtocolSwitching` before login finished is sent. Only one acknowledgement is
legal there. Its transition plan is ordered:

1. install configuration clientbound;
2. build the connection cookie from the normalized profile;
3. install configuration serverbound;
4. start configuration tasks and mark login accepted.

Early or duplicate hello/acknowledgement packets are terminal protocol faults. The timeout uses the
official post-increment boundary: the tick whose prior counter is 600 sends slow-login disconnect.
A separate pool retains one injected random server-session UUID while any acquired connection is
active and clears it only after the pool drains.

## Conformance evidence

`crates/ferrite-protocol/tests/c1/login_serverbound_required.rs` locks the hello and raw/compressed
acknowledgement goldens, every name boundary, Java-compatible UUIDv3 derivation, ignored supplied
UUIDs, admission and duplicate-player behavior, compression callback order, acknowledgement order,
the exact timeout comparison, optional-ID refusal, and shared server-session UUID lifetime.
