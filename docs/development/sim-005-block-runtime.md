# SIM-005 Block Runtime

`G01-P5-S008` implements the final two `SourceSpecified` block slices, both primarily owned by
`SIM-005`. The protocol-neutral kernels live in `ferrite-gameplay`; Region integration supplies
ordered world/entity observations, owns the block-event queue and applies the returned semantic
effects.

## Module ownership

| Module | Audited responsibility |
|---|---|
| `block::bell` | All 32 state identities, placement/support transforms, exact collision/outline boxes, hit admission, ring ingress, cached hearing, shake/resonance clocks, glow/particle plans, render rotation and loot boundary |
| `block::enchanting_table` | Static identity/profile, menu and custom-name boundary, 32 ordered bookshelf probes, client particle draw order, shared-stream book animation and renderer inputs |

Bell ingress remains deliberately split. A successful server attempt mutates transient
block-entity state and yields the exact queued block event before the immediate ordinary sound,
game event and optional player stat. The later event owns cache refresh, hearing and synchronized
shake/resonance reset. Identical event deduplication remains the Region scheduler's responsibility;
the bell kernel still reports every immediate ingress effect.

The enchanting table constructs only the menu-provider boundary. Offer generation, enchantment
selection, lapis/experience payment and commit remain owned by `ITM-ENCHANT-001`.

## Determinism and state boundaries

- Bell caches preserve query order and refresh only when the current game time is strictly greater
  than the previous timestamp plus 60. Current entity observations are resolved through cached
  identities so moved, dead and removed entities retain source behavior.
- Bell resonance retries on each shaking tick until a qualifying raider is present. Its 40-tick
  completion emits either a server glow plan or the exact client particle sequence; neither path
  consumes RNG.
- Enchanting-table bookshelf scanning always consumes one bounded draw per offset before invoking
  the provider/transmitter read. Only admitted probes consume three floats.
- `BookAnimation::tick` accepts a caller-owned random stream. Client integration must share that
  stream across tables and worlds in ticker order; it must not derive a per-Region or per-table
  seed.
- Bell animation and enchanting-table book fields are client transient. Only the table's nullable
  custom name crosses persistence/component/loot boundaries.

## Runtime integration

Region-owned block state remains the only world authority. Runtime adapters must apply returned
writes, queue records, memories, effects, sounds, game events, statistics and particles in source
order. Protocol IDs in these modules are dense Minecraft Java 26.2 projection identities, while
durable content continues to use stable resource identities.

Generic interaction precedence, block-event deduplication/broadcast, entity query ordering, effect
merging, packet projection and particle admission remain shared-owner boundaries. The kernels
expose their exact subtype inputs and outcomes without creating alternate world, entity, content
or protocol authorities.

## Verification

The committed test owner is
`crates/ferrite-gameplay/tests/slices/blocks/sim_005.rs`. Its 28 tests exhaust Bell state IDs,
placement/support/shape/hit branches, side-specific ingress, cache boundaries, resonance reruns,
glow/particle filtering and renderer axes, plus enchanting-table menu/name/loot behavior, all 32
probe positions and draw order, player selection, page clocks, rotation/clamp boundaries, renderer
formulas and caller-shared randomness.
