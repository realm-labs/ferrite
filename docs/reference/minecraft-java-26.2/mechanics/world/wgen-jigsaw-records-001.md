# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-RECORDS-001` — Ten locked records parameterize the generic jigsaw transaction

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — all ten jigsaw structure records, ten start-biome tags, six selecting structure
sets and the trial-chamber alias composition are locked data-only inputs to `WGEN-JIGSAW-CORE-001`
and shared placement. Their exact start, adaptation, override and set semantics are fixed here.
Processor-list algorithms are owned by `WGEN-JIGSAW-PROCESSORS-001`; ancient-city, bastion, outpost,
trail-ruins, trial-chambers and village template block/entity/NBT payloads are owned by
`WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-BASTION-001`, `WGEN-JIGSAW-OUTPOST-001`,
`WGEN-JIGSAW-TRAIL-RUINS-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001` and `WGEN-JIGSAW-VILLAGES-001`;
`minecraft:jigsaw` is complete at this layer.

**Applies when:**

A selecting structure set admits one of the ten records, the record samples its start height and
invokes the generic jigsaw core, the generic biome predicate tests the returned stub, terrain
adaptation consumes its piece/junction boxes, or natural spawning queries a retained override.

**Authoritative state:**

The selected record and set entry; record step, biome holder set, terrain adaptation, start
pool/name, height provider, projection, depth, range, expansion, aliases, padding, liquid and
overrides; set weights and placement record. Omitted fields use locked codec defaults rather than
family-specific inference.

**Transition and ordering:**

Shared placement selects a set position and weighted structure entry under `WGEN-PIPELINE-001`.
`JigsawStructure` then samples record height before creating positional aliases and before the core
consumes rotation/start-pool draws. Constant heights consume no random value. Trial chambers alone
consume one inclusive uniform integer for `-40..-20`; their alias choices use the independent
world-seed/start-position stream described by `WGEN-JIGSAW-CORE-001`. After the core returns a stub,
the shared generic 3-D biome predicate tests the stub position against the record tag.

The ten records are exact:

#### `ancient_city`

**Step / allowed start biomes:**

underground decoration; deep dark only

**Adaptation:**

beard box

**Start pool; height and projection:**

`ancient_city/city_center`, named `city_anchor`; absolute `-27`, no heightmap

**Depth / range / expansion:**

`7 / 116 / false`

**Other state:**

default padding/liquid; all eight creature categories replaced by empty full-box lists

#### `bastion_remnant`

**Step / allowed start biomes:**

surface structures; crimson forest, Nether wastes, soul-sand valley, warped forest

**Adaptation:**

none

**Start pool; height and projection:**

`bastion/starts`; absolute `33`, no heightmap

**Depth / range / expansion:**

`6 / 80 / false`

**Other state:**

default padding/liquid; no overrides

#### `pillager_outpost`

**Step / allowed start biomes:**

surface structures; desert, plains, savanna, snowy plains, taiga, grove plus the six-biome mountain
tag

**Adaptation:**

beard thin

**Start pool; height and projection:**

`pillager_outpost/base_plates`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`7 / 80 / true`

**Other state:**

full-box monster list replaced by pillager weight `1`, count `1..1`

#### `trail_ruins`

**Step / allowed start biomes:**

underground structures; taiga, snowy taiga, both old-growth spruce/pine taigas, old-growth birch
forest, jungle

**Adaptation:**

bury

**Start pool; height and projection:**

`trail_ruins/tower`; absolute `-15` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`7 / 80 / false`

**Other state:**

default padding/liquid; no overrides

#### `trial_chambers`

**Step / allowed start biomes:**

underground structures; exact 54-holder tag

**Adaptation:**

encapsulate

**Start pool; height and projection:**

`trial_chambers/chamber/end`; uniform absolute `-40..-20`, no heightmap

**Depth / range / expansion:**

`20 / 116 / false`

**Other state:**

symmetric padding `10`; ignore-waterlogging; all eight categories replaced by empty piece-box lists;
three alias bindings

#### `village_desert`

**Step / allowed start biomes:**

surface structures; desert

**Adaptation:**

beard thin

**Start pool; height and projection:**

`village/desert/town_centers`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`6 / 80 / true`

**Other state:**

defaults; no overrides

#### `village_plains`

**Step / allowed start biomes:**

surface structures; plains and meadow

**Adaptation:**

beard thin

**Start pool; height and projection:**

`village/plains/town_centers`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`6 / 80 / true`

**Other state:**

defaults; no overrides

#### `village_savanna`

**Step / allowed start biomes:**

surface structures; savanna

**Adaptation:**

beard thin

**Start pool; height and projection:**

`village/savanna/town_centers`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`6 / 80 / true`

**Other state:**

defaults; no overrides

#### `village_snowy`

**Step / allowed start biomes:**

surface structures; snowy plains

**Adaptation:**

beard thin

**Start pool; height and projection:**

`village/snowy/town_centers`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`6 / 80 / true`

**Other state:**

defaults; no overrides

#### `village_taiga`

**Step / allowed start biomes:**

surface structures; taiga

**Adaptation:**

beard thin

**Start pool; height and projection:**

`village/taiga/town_centers`; absolute `0` plus `WORLD_SURFACE_WG`

**Depth / range / expansion:**

`6 / 80 / true`

**Other state:**

defaults; no overrides

The outpost mountain tag is exactly meadow, frozen peaks, jagged peaks, stony peaks, snowy slopes
and cherry grove, making 12 resolved holders. The trial tag's 54 explicit holders span every locked
ocean climate, ordinary surface family and the dripstone/lush/sulfur cave biomes but deliberately
exclude deep dark, Nether and End biomes. Tag list order does not weight the generic membership
test. Ancient-city full-box empties suppress ambient, axolotl, creature, misc, monster,
underground-water-creature, water-ambient and water-creature lists throughout the structure box.
Trial empties use piece boxes instead. Unmentioned categories retain ordinary biome lists.

**Trial aliases:**

Binding order is ranged group, melee random, small-melee random. The first equal-weight three-way
choice atomically maps both `contents/ranged` and `contents/slow_ranged` to skeleton, stray or
poison-skeleton paired pools. The next equal three-way choice maps `contents/melee` to zombie, husk
or spider. The final equal four-way choice maps `contents/small_melee` to slime, cave spider,
silverfish or baby zombie. These three positional choices are independent of the structure stream
and remain fixed for that world-seed/start-position pair.

**Structure sets:**

All use random spread and default linear spread/frequency unless stated. `ancient_cities` is sole
ancient city weight `1`, spacing/separation `24/8`, salt `20083232`. `nether_complexes` is fortress
weight `2` then bastion weight `3`, `27/4`, salt `30084232`. `pillager_outposts` is sole outpost
weight `1`, `32/8`, salt `165745296`, legacy-type-1 frequency `0.2`, and rejects candidates within
`10` chunks of set `villages`. `trail_ruins` is sole trail ruins weight `1`, `34/8`, salt
`83469867`. `trial_chambers` is sole chambers weight `1`, `34/12`, salt `94251327`. `villages` lists
plains, desert, savanna, snowy and taiga in that order at equal weight `1`, `34/8`, salt `10387312`.
Placement arithmetic, frequency reduction, exclusion queries and set-entry choice remain with shared
placement.

**Branches and aborts:**

Six set admissions and every weighted entry; outpost frequency/exclusion; ten height/biome outcomes;
trial 21 heights and 3×3×4 alias outcomes; heightmap present/absent; five adaptations; expansion
true/false; padding/liquid defaults versus trial overrides; empty/full/piece override consumers;
generic core success/empty start/padding/range; later processor/payload outcomes.

**Constants and randomness:**

Only trial's uniform height advances the structure stream before core rotation. Constants, tags,
steps, adaptations, pool IDs, range/depth/flags and overrides consume none. Trial's three alias
choices use the independent positional stream. Shared set placement/entry randomness stays with
`WGEN-PIPELINE-001`.

**Side effects:**

Record-selected piece graphs and terrain-adaptation inputs; trial liquid behavior and alias-stable
spawner pool choices; ancient/trial empty spawn lists or outpost pillager list; set
locate/exclusion/distribution behavior through the shared owner.

**Gates:**

Shared set placement, frequency and exclusion; weighted entry; record height and biome; core
pool/padding/range; processing chunks; adaptation/noise consumers; natural-spawn category and
bounding-box policy.

**Boundary cases and quirks:**

Ancient city is the only named start connector. Trial height is absolute, not relative to surface.
Trail ruins alone subtracts `15` from surface projection. Only trial changes padding/liquid and only
trial uses aliases. Outpost's monster override is full-box while trial empties are piece-box and
ancient empties full-box. The shared Nether set preserves fortress-before-bastion order and `2:3`
weights.

**Evidence:**

`Confirmed`; `OFF-DATA-001`; `OFF-SERVER-001`. Anchors: all ten `worldgen/structure` records; six
`structure_set` records; ten `has_structure` tags and `is_mountain`;
`JigsawStructure#findGenerationPoint`; generic structure biome validation; alias creation;
constant/uniform height providers.

**Test vectors:**

Query/decode all records, tags and sets. Assert every omitted/default versus explicit field, exact
resolved holder memberships, trial height endpoints and alias cross-product, generic core
parameters, terrain adaptation and override box/category behavior. Replay every set
weight/position/frequency/exclusion edge through its shared owner. `WGEN-JIGSAW-PROCESSORS-001`
binds all referenced processors; the ancient-city, bastion, outpost, trail-ruins, trial-chambers and
village leaf owners bind all payload families without changing these records.
