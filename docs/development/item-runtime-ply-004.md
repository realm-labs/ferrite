# PLY-004-Owned Boat and Harness Runtime

`G01-P6-S009` implements the audited boat/raft and Happy Ghast harness item slices primarily
owned by `PLY-004`.

## Runtime boundary

`item::runtime::transport` separates three responsibilities:

- `catalog` closes the ten ordinary/chest boat families, twenty item/entity mappings, sixteen
  harness mappings, fuel and fisherman trade records;
- `boat` owns held and dispenser placement decisions, mount-versus-container interaction, chest
  loot/storage persistence, destructive removal, ride-height joins, recipe form, and goat
  advancement admission;
- `harness` owns direct/dispenser equip admission, live-tag validity, leash/shearing order,
  temptation, passenger/controller gates, ridden input and rotation, still timeout, and recipe
  profiles.

Vehicle physics and damage, generic menus, loot evaluation/fill allocation, item-entity velocity,
equipment death drops, passenger serialization, protocol codecs, recipes, advancements, trade-set
selection, and client rendering remain with their dedicated owners.

## Exact transactions

Held boat use preserves the source order: POV hit, eye containment against every inflated
pickable candidate, exact factory creation, server-only default stack configuration, yaw,
collision, spawn attempt, placement event, consumption, and statistic. Once collision succeeds,
a rejected server entity admission deliberately does not roll back the event or consumption.

Ordinary interaction mounts only below 60 out-of-control ticks. Chest interaction delegates that
branch first, then opens storage only for secondary use or when another passenger cannot be
admitted. A failed mount while capacity remains is a pass, not a container open.

Chest storage has 27 slots. Pending loot and materialized items are mutually exclusive on save,
spectators cannot open pending loot, and materialization clears the pending key before invoking
the fill owner. Destructive server removal scatters contents before matching vehicle itemization
and does so even when `entity_drops` is disabled.

Harness equip requires a live adult Happy Ghast, an empty body slot, and current allowed-entity
tag admission. Server equip always splits one item, including creative use. Leash cutting wins
over shearing for that interaction. A valid harness enables four-passenger mounting and first
player control only when the persisted still timeout is not positive.

## Determinism and Region ownership

The authoritative Region owns the complete vehicle, storage, equipment, passenger, and timeout
state. Placement and equipment results are transaction descriptions for the entity owner; they
do not consult process identity. Entity admission remains generation-fenced by the Region
runtime.

Chest content splitting and loot fill receive caller-owned randomness and callbacks. Encounter
order selects dispenser candidates. Catalog and trade arrays preserve locked registry/data order,
so topology changes cannot alter identity resolution.

## Validation

`crates/ferrite-gameplay/tests/slices/items/ply_004.rs` verifies all 36 item identities, all boat
entity IDs, fisherman coverage, held placement aborts and admission quirk, mount/open asymmetry,
loot persistence and materialization, removal ordering, dispenser offsets, ride heights, harness
equip/candidate/shearing order, live-tag validity, passenger/controller gates, flight vectors,
rotation, timeout, and recipes.
