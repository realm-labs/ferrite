# Play Clientbound Merchant Protocol

`G01-P6-F003` implements the single required packet in
`PROTO-PLAY-CLIENTBOUND-MERCHANT-001` for Minecraft Java 26.2:

| ID | Identity | Adapter responsibility |
|---:|---|---|
| 52 | `minecraft:merchant_offers` | replace the matching merchant menu's offers and HUD state |

The packet carries a signed container ID, a generic VarInt-counted offer list, signed villager
level and experience VarInts, then show-progress and can-restock booleans. Counts are bounded only
by the packet body and negative lengths fault. Level, experience, and offer numeric fields retain
their signed values without codec clamping.

## Offer and item grammar

Each offer decodes base cost A, a required nonempty result stack, optional cost B, an out-of-stock
boolean, four signed big-endian integers, a raw IEEE-754 multiplier, and signed demand in locked
order. Item and component raw IDs resolve through the connection registries. Results reuse the
shared item/component-patch codec but reject empty, nonpositive, and AIR forms.

An item cost contains item identity, signed count, and an ordered exact-component predicate.
Predicate entries retain duplicates rather than normalizing to a map. Matching requires the same
item and equality for every expected component while permitting additional candidate components;
the count check remains separate.

Network decode fixes `reward_experience` to true. A true wire out-of-stock flag replaces wire uses
with maximum uses before installing the special-price difference. A false flag keeps wire uses,
which can still derive out of stock when `uses >= max_uses`. Canonical encoding always emits that
derived predicate.

## Price and satisfaction

Cost A uses Java signed wrapping and float behavior:

```text
product      = wrapping(base_count * demand)
demand_delta = max(0, java_floor((float) product * multiplier))
modified     = clamp(wrapping(base_count + demand_delta + special_price), 1, stack maximum)
```

The implementation preserves NaN, infinity, saturating float-to-int conversion, and wrapping
subtraction at the Java floor boundary. Cost B uses its unmodified signed base count. An absent
cost B requires an empty second payment input. Successful assembly returns an owned result copy.

## Client application

The bounded client projection ignores ID 52 unless its container ID exactly equals the current
menu and that menu is a merchant menu. A merchant screen is not required. Success replaces copied
offers, then experience, level, show-progress, and can-restock in source order. There is no merge,
generation token, monotonic check, or acknowledgement, so a delayed packet can affect a reused
matching merchant container ID.

## Server publication

`MerchantPublisher` composes the ordinary `ContainerPublisher`. Canonical opening emits any prior
close, then open-screen, full content/cursor, and properties before appending the owned offer
snapshot. An empty authoritative offer list omits ID 52. Later selection and trade prediction are
not acknowledged by this packet; ordinary container traffic converges payment and result slots.

Container IDs, raw registry IDs, wire offer order, presentation price caches, and GUI selection
remain connection-local adapter state rather than Region identity or persistence state.

## Evidence

`crates/ferrite-protocol/tests/c3/play_clientbound_merchant.rs` owns the locked golden, structured
and malformed codecs, duplicate exact predicates, stock normalization, Java price boundaries,
menu gates, canonical publication order, empty-offer omission, owned snapshots, and an
encode/decode-to-client convergence trace.
