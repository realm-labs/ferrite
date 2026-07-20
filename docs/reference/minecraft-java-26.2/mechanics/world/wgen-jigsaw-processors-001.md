# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-PROCESSORS-001` — Structure processors form an ordered, position-stable rewrite transaction

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked 26.2 source fixes the generic processor transaction, all 11
structure-processor types, all six rule tests, three position tests and four block-entity modifiers.
All 40 official processor-list records are audited data-only compositions here. Thirty-six are
selected by jigsaw pool elements, three are fossil-feature inputs and `empty` is an explicit unused
identity list. Ancient-city, bastion, outpost, trail-ruins, trial-chambers and village template
payloads are owned by `WGEN-JIGSAW-ANCIENT-CITY-001`, `WGEN-JIGSAW-BASTION-001`,
`WGEN-JIGSAW-OUTPOST-001`, `WGEN-JIGSAW-TRAIL-RUINS-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001` and
`WGEN-JIGSAW-VILLAGES-001`.

**Applies when:**

A structure template processes its selected palette, a single/legacy-single pool element appends its
configured list, terrain matching appends gravity, a feature swaps processor lists, or custom
locked-compatible data invokes any registered processor, predicate or NBT modifier.

**Authoritative state:**

Ordered raw palette cells; transformed current cell or null; template origin, reference position and
raw template-relative position; settings transform, clip, optional RNG and processor order; live
block/height/shape state; world seed; processor/list fields; rule predicates, output state and
optional modifier; copied cell NBT.

**Transition and ordering:**

`processBlockInfos` first scans the processor list. If no processor evaluates whole-piece state, a
transformed raw cell outside the settings box is skipped before any processor. Presence of even one
whole-piece processor—currently `capped`—disables that early clip for **every** cell and processor.
Each admitted raw cell is transformed, its NBT copied, and passed through processors in list order
until one returns null. Surviving raw and processed entries are kept at matching indices. Every
processor then receives the two lists through `finalizeProcessing`, again in list order; ordinary
processors return them unchanged. The later template write loop applies the settings box, so a
capped list can read and choose outside-box cells while only in-box results are written.

**The 11 structure processors:**

- `nop` returns the cell. `block_ignore` returns null when the current processed block identity is
  in its configured list; its state-shaped codec discards supplied properties and retains block
  identities. `protected_blocks` instead reads the live block at the current processed position and
  returns null when it belongs to the configured holder set.
- `block_rot` obtains the settings RNG at the current processed world position. A configured
  rottable set rejects nonmembers without drawing; otherwise `nextFloat() <= integrity` retains the
  cell and a greater value removes it. Integrity is codec-bounded `0..1`; equality survives. With no
  settings RNG, position hashing makes decisions position-stable; an explicitly supplied RNG makes
  them caller-stream ordered.
- `gravity` queries the configured heightmap at processed X/Z, translating `WORLD_SURFACE_WG` to
  live `WORLD_SURFACE` and `OCEAN_FLOOR_WG` to live `OCEAN_FLOOR` only for `ServerLevel`. Its output
  Y is queried height plus offset plus the **raw template-relative Y**; processed state/NBT and X/Z
  survive. Defaults are `WORLD_SURFACE_WG` and zero.
- `lava_submerged_block` reads live state at the processed position. If it is lava and the processed
  state's contextual collision shape is not a full block, the output becomes default lava with the
  current NBT; otherwise the cell survives unchanged. `jigsaw_replacement` leaves non-jigsaws and
  debug-kept jigsaws alone. Missing NBT warns and leaves the jigsaw. Otherwise it parses
  `final_state`, defaulting to air: a valid non-structure-void state replaces the cell and clears
  NBT, structure void removes it, and a parse failure logs and removes it.
- `blackstone_replace` maps cobblestone/mossy cobblestone to blackstone; stone to polished
  blackstone; stone bricks/mossy stone bricks to polished-blackstone bricks; their cobblestone,
  mossy-cobblestone, stone, stone-brick and mossy-stone-brick stair variants to the corresponding
  blackstone/polished variants; cobblestone, mossy-cobblestone, smooth-stone, stone, stone-brick and
  mossy-stone-brick slabs likewise; stone-brick/mossy-stone-brick walls to polished-blackstone-brick
  wall; cobblestone/mossy-cobblestone walls to blackstone wall; chiseled/cracked stone bricks to
  chiseled/cracked polished blackstone; and iron bars to chain. Only `facing`, `half` and slab
  `type`, when present on the input, are copied to the target default; position and NBT survive,
  other properties do not.
- `block_age` uses the settings RNG at the processed position. Stone bricks, stone and chiseled
  stone bricks first pass a `<0.5` gate; eager construction consumes facing/half draws for both
  stone-brick and mossy-stone-brick stair candidates before a mossiness draw chooses mossy versus
  nonmossy and a two-way draw chooses brick/stair. Any stair first passes `<0.5`, then chooses mossy
  stair-or-slab versus stone/stone-brick slab; any slab or wall changes to its mossy-stone-brick
  counterpart on `<mossiness`, retaining compatible properties. Obsidian becomes crying obsidian on
  `<0.15`. Other states and failed gates survive. The full-block replacement can therefore discard
  orientation from the source; stair/slab/wall paths copy compatible properties.
- `rule` creates a fresh RNG from the processed world-position seed for each cell and tests rules in
  data order. A rule short-circuits input, then live-location, then position predicates. The first
  match returns its exact output state and modifier-produced NBT; no match preserves the current
  cell. Separate rule processors at the same world position restart the same seed, while tests
  within one processor share and conditionally advance it.
- `capped` is the sole whole-piece processor. A maximum limit of zero, empty processed list, or
  sampled limit below one is identity. A raw/processed size mismatch logs and skips. Otherwise it
  seeds a thread-local RNG from world seed, forks positional randomness at the template origin,
  samples the positive limit, caps it by list size, shuffles all indices, and invokes its delegate
  on each candidate until that many **non-null, unequal** outputs occur or candidates end. Null and
  equal delegate results neither count nor alter the list. It replaces matching indices in place and
  never removes cells. The delegate receives the raw cell position but current processed state/world
  position; later capped processors see earlier changes and independently restart the same origin
  seed unless their limit sampling differs.

**Rule registries:**

`always_true` admits without drawing. `block_match` compares block identity; `blockstate_match`
compares the canonical full state by identity. Their random variants first require the same
comparison, then draw and admit only on `nextFloat() < probability`; `tag_match` tests holder
membership. Location tests apply the same rule to the live world state. Position `always_true` draws
nothing. `linear_pos` uses Manhattan distance from world position to reference;
`axis_aligned_linear_pos` uses absolute displacement on its configured axis, default Y. Both draw
once and admit on
`draw <= clampedLerp(inverseLerp(distance,min_dist,max_dist),min_chance,max_chance)`; construction
rejects `min_dist >= max_dist`. Chance defaults are zero, distances zero and axis Y, so valid data
must override the distance interval.

The modifier default is `passthrough`, including nullable NBT. `clear` returns a new empty compound.
`append_static` returns a configured-data copy when input is null, otherwise merges configured keys
into the current copied compound. `append_loot` copies or creates a compound, writes the typed
`LootTable` key and consumes one rule RNG `nextLong` for `LootTableSeed`. Output NBT is not
validated against the output block type until the later write/load transaction.

**Locked list census:**

The 40 records contain 52 top-level processors: 35 rule, seven protected-block, six block-rot and
four capped. Their rule and capped delegates contain 164 ordered rules: input predicates are one
always-true, 23 block-match, eight blockstate-match, 123 random-block-match and nine tag-match;
locations are 154 always-true and ten block-match; positions are 163 default always-true and one
axis-aligned-linear; modifiers are 160 passthrough and four append-loot. No locked list uses
random-blockstate, linear-pos, append-static, clear, block-age, ignore, gravity, jigsaw-replacement,
lava-submerged, nop or blackstone-replace, but their registered behavior is source-specified for
custom compatible data and direct source callers.

Exact records, shown as `top-level processors / rules / jigsaw pool-element references`, are:

- Ancient city: `ancient_city_generic_degradation` `3/3/36`, `ancient_city_start_degradation`
  `2/3/3`, `ancient_city_walls_degradation` `3/4/24`. Generic/walls begin with tag-limited integrity
  `0.95`; all crack deepslate bricks/tiles at `0.3`, remove soul lantern at `0.05`, walls
  additionally remove deepslate-tile slabs at `0.3`, and all end with the exact seven-block
  `features_cannot_replace` set.
- Bastion: `bastion_generic_degradation` `1/5/9`, `bottom_rampart` `1/4/1`, `bridge` `1/2/1`,
  `entrance_replacement` `1/4/1`, `high_rampart` `1/3/1`, `high_wall` `1/4/10`, `housing` `1/4/31`,
  `rampart_degradation` `1/7/5`, `roof` `1/3/3`, `side_wall_degradation` `1/4/2`,
  `stable_degradation` `1/4/51`, `treasure_rooms` `1/4/50`. These are ordered rule lists over
  polished/chiseled/cracked blackstone, blackstone, magma, gold and gilded blackstone. The sole
  position rule is high-rampart always-true to air with Y-axis chance rising from `0` at distance
  `0` to `0.05` at `100`; first-match ordering prevents later degradation after a match.
- Villages/outpost: five farms are desert `1/2/3`, plains `1/3/2`, savanna `1/1/3`, snowy `1/2/2`,
  taiga `1/2/2`; three streets are plains `1/4/36`, savanna `1/4/48`, snowy-or-taiga `1/5/72`;
  mossify `10/20/70` percent are each `1/1` with `53/2/2` references; `outpost_rot` is `1/0/1` at
  integrity `0.05`; zombie desert/plains/savanna/snowy/taiga are `1/10/32`, `1/17/40`, `1/14/36`,
  `1/13/33`, `1/12/28`. Street rules inspect live water/ice before random path-to-grass; zombie
  lists remove doors/lights, apply ordered cobweb/glass/crop/campfire changes, and their exact
  states/probabilities remain the queried data contract.
- Trail ruins: houses `3/5/72`, roads `2/4/7`, tower-top `1/1/5`. Houses/roads first age gravel/mud.
  Their capped common suspicious-gravel limits are `6/2/2`; houses additionally applies rare limit
  `3`. The replaceable tag is exactly gravel. Each successful archaeology rule writes dusted-zero
  suspicious gravel and the common or rare loot record plus a position-seeded loot seed.
- Trial chambers: copper-bulb degradation is `2/3/50`; ordered `0.1`, `0.33333334`, `0.5` tests
  change waxed copper bulbs to lit/unpowered oxidized, weathered or exposed bulbs, followed by the
  protected set.
- Fossil records are `fossil_coal` `2/0`, `fossil_diamonds` `3/1`, `fossil_rot` `2/0`: integrity
  `0.1`, `0.1`, `0.9`, respectively; diamonds has the middle coal-ore to deepslate-diamond-ore rule;
  all end protected. They have no pool references and remain feature inputs. `empty` is `0/0/0`. The
  other 36 names account for exactly 757 single/list element references before weight expansion.

The ancient-city rottable tag is exactly 12 deepslate/gray-wool blocks; `features_cannot_replace` is
bedrock, spawner, chest, End-portal frame, reinforced deepslate, trial spawner and vault; trail
replaceable is gravel. The zombie door predicate expands the locked doors tag, including wooden-door
tag members, all nine copper oxidation/wax variants and iron door.

**Branches and aborts:**

Early clip enabled/disabled; every processor order and null; supplied/derived settings RNG; rottable
member/nonmember and threshold equality; live protected/lava/height states; every replacement and
property compatibility; valid/missing/invalid/void jigsaw state; first/no rule match and predicate
short-circuit; position interpolation endpoints; nullable NBT and four modifiers; capped
zero/empty/mismatch/sample/shuffle/equal/null/replacement paths; all list selections.

**Constants and randomness:**

Position-derived settings RNG uses the processed world-position seed; rule RNG independently
restarts from that seed. Capped uses world seed plus template origin, samples limit before
shuffling, and append-loot consumes the rule stream only after all predicates match. Data list
order, first-match behavior and processor order are observable.

**Side effects:**

A rewritten or removed cell list; live height/state reads; capped whole-piece selection; parsed
jigsaw final states; copied/cleared/merged/loot-bearing NBT; warnings for missing/malformed jigsaws
or mismatched capped lists. Actual block/entity writes remain the generic template transaction.

**Gates:**

Codec validity and registry lookup; caller processor assembly; palette/transform/clip; settings RNG;
block/tag/state/property identities; live world queries; world seed/origin; rule/modifier and
resulting block-entity compatibility.

**Boundary cases and quirks:**

One capped processor moves clipping after processing for the entire list. Block-rot equality
retains, random rule equality rejects, and position-rule equality admits. Rule processors restart
their seed, so splitting one rule list changes draws. Capped null delegates do not remove or count.
Gravity uses raw template-relative Y after prior processors may have moved the cell. Block-age
eagerly consumes four direction/half draws on a successful full-stone gate before selecting an
output. High-rampart removal probability increases with vertical distance. Missing jigsaw NBT
preserves the jigsaw, malformed final state removes it.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`; `OFF-REPORT-001`. Anchors:
`StructureTemplate#processBlockInfos`; `StructureProcessor`; all 11 processor implementations;
`ProcessorRule`, all rule/position tests and all four block-entity modifiers; all 40
`data/minecraft/worldgen/processor_list/*.json` records; all 188 pool records; four referenced block
tags and both trail-ruins archaeology loot records.

**Test vectors:**

Query all 40 records and 24 registry IDs. Assert list order/counts, exact
predicates/states/probabilities/tags/limits/modifiers and pool reference census. Replay supplied
versus positional RNG, clip edges, every processor branch, rule short-circuit/draw trace, position
endpoints, NBT modes and capped whole-piece/mismatch/shuffle behavior. Cross all 36 jigsaw lists
through `WGEN-JIGSAW-CORE-001`; keep ancient-city, bastion, outpost, trail-ruins, trial-chambers and
village payload semantics in their named owners.
