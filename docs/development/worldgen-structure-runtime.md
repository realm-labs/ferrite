# Minecraft 26.2 structure runtime

Ferrite's `WGEN-003` owner lives in `ferrite-world::generation::structure`. It separates four
responsibilities that vanilla often composes inside one structure class:

1. structure admission and graph construction;
2. persistent piece geometry and ordered expansion;
3. saved-template decoding, transformation, processors, and marker transactions;
4. locked structure, structure-set, pool, processor, biome, spawn, and loot records.

This boundary keeps graph RNG independent from position-derived template processor RNG and from
caller-owned loot RNG. A piece carries stable geometry and saved-template identity; placement is
always clipped by the processing box and writes through `PieceWorld`/`TemplateWorld`, so a Region
owner can reject or schedule work without granting a structure direct access to another Region.

## Shared template and jigsaw path

`template`, `template_manager`, `template_place`, and `nbt` decode the official gzip-NBT structure
format, cache resources by namespace, apply mirror-before-rotation transforms, retain explicit air
when the selected processor permits it, and perform block/NBT/entity updates in source order.
`processor` implements the locked processor transactions and their private random streams.

`jigsaw`, `pool_catalog`, and `pool_place` own connector compatibility, weighted pool expansion,
collision/free-space gates, projection, feature/list/single elements, and final block placement.
The locked Ancient City, Bastion, Pillager Outpost, Trail Ruins, Trial Chambers, and Village
families are data-driven through the same graph and placement path. Their registry records remain
content-bundle inputs rather than copied generated Rust data.

## Procedural structure path

Dedicated modules own source-specific graphs and placement for buried treasure, desert pyramids,
End cities, Nether fortresses, igloos, jungle temples, mineshafts, Nether fossils, ocean monuments,
ocean ruins, ruined portals, shipwrecks, strongholds, swamp huts, and woodland mansions. Shared
piece primitives provide orientation, clipping, fluid scheduling, shape postprocessing, chests,
and downward support columns without hiding structure-specific draw or mutation order.

The woodland-mansion implementation is split further into graph, perimeter/room scheduling, roof,
catalog, and runtime modules. Its 73 official templates remain external locked inputs. Generation
produces an ordered list of depth-zero template pieces; later chunk processing reloads each named
template, recomputes its transformed box, places explicit air while ignoring only structure
blocks, and handles DATA markers. The structure-level foundation pass runs only after pieces and
fills qualifying live columns through air or liquid down to the first solid nonliquid block.

## Determinism and ownership

- Generation consumes the caller's structure stream in source order.
- Processor settings may derive private streams from stable positions.
- Loot seeds are drawn only after a live typed container is available.
- Mansion allay counts use world RNG, matching their entity-lifecycle boundary.
- Piece and template iteration is ordered; no hash iteration controls generation.
- Structure placement never implies cross-Region mutation authority. The caller supplies the
  processing box and an owner-scoped world implementation.

Official template assertions are conditional on the locked resource cache being present, matching
the repository's other content-backed conformance tests. Pure graph, geometry, catalog, and branch
tests remain runnable without that cache.
