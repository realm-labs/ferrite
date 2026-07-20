# World mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `WGEN-JIGSAW-TRIAL-CHAMBERS-001` — Trial-chamber pools bind one structure-wide mob roster to destructive copper rooms

**Parent:** `WGEN-003`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the 47 locked trial-chamber pools, record-owned alias bindings, one copper-bulb
processor list, 191 present NBT templates, 28 trial-spawner configurations, 24 transitively linked
loot records and the standalone water-dispenser table fix the complete payload supplied to generic
jigsaw/template placement. Every template exists and is reachable; four connector pool IDs are
intentional aliases rather than missing records. Ordinary-single air/water, protected live cells,
fixed container/spawner/vault NBT and caller-stream container seeds are specified here. Village
payloads are owned by `WGEN-JIGSAW-VILLAGES-001`. Trial-spawner and vault post-placement runtime
dispatch to `BLK-TRIAL-SPAWNER-001` and `BLK-VAULT-001`; this leaf owns their exact initial block
states/configuration inputs.

**Applies when:**

`trial_chambers` chooses one structure-wide ranged/slow-ranged, melee and small-melee alias roster,
starts at `chamber/end`, expands any chamber/corridor/decor/reward/spawner connector, degrades a
copper-bulb template, or loads a template container, trial spawner or vault. Record
height/adaptation/range/padding/liquid and spawn-override fields remain with
`WGEN-JIGSAW-RECORDS-001`.

**Authoritative state:**

Record/core and alias lookup; 47 ordered rigid pools with their primary/fallback weights; 191
sparse/full single-palette templates and connector/NBT fields; the degradation/protection list and
live target states; rotation, origin, chunk clip and caller structure RNG; 28 resolved trial-spawner
records; two resolved vault configs; 25 exact owned loot records.

**Transition and ordering:**

Forty-five pools fall back to `minecraft:empty`; `chambers/end` and `hallway` fall back to
`trial_chambers/hallway/fallback`. The 213 top-level entries expand to weight `1,534`: 207 rigid
ordinary singles and six Empty elements, with no list, feature or legacy single. All 191 distinct
template locators exist and are referenced; repeated locators account for the other 16 single
entries. The exact pool census is:

| Pool prefix below `trial_chambers/` | Pools | Entries | Expanded weight | Singles | Empty |
|---|---:|---:|---:|---:|---:|
| `atrium` | 1 | 7 | 7 | 7 | 0 |
| `chamber` | 7 | 71 | 97 | 71 | 0 |
| `chambers` | 1 | 4 | 4 | 4 | 0 |
| `chests` | 2 | 2 | 2 | 2 | 0 |
| `corridor` | 2 | 17 | 23 | 17 | 0 |
| `corridors` | 3 | 15 | 37 | 12 | 3 |
| `decor` | 4 | 31 | 97 | 29 | 2 |
| `dispensers` | 1 | 4 | 4 | 3 | 1 |
| `entrance` | 1 | 3 | 3 | 3 | 0 |
| `hallway` | 2 | 34 | 1,235 | 34 | 0 |
| `reward` | 3 | 3 | 3 | 3 | 0 |
| `spawner` | 20 | 22 | 22 | 22 | 0 |

The dominant `hallway` pool alone has 30 entries/weight `1,231`: eight chamber forms each weight
`150`, rubble chamber weight `10`, and 21 weight-one connectors/rubble/hallway/encounter forms. The
fallback pool contains the four equal rubble forms. The six primary Empty entries occur in the three
corridor-addon pools, general/chamber decor and chamber dispensers with weights `8/8/6/22/4/1`; all
other pool weights and order are fixed by the queryable records.

At structure start, the alias lookup chooses one of three equal paired groups, mapping both ranged
aliases consistently to skeleton, stray or poison-skeleton families; poison skeleton resolves to
bogged. Independent equal choices map melee to zombie/husk/spider and small melee to
slime/cave-spider/silverfish/baby-zombie. The four virtual IDs are `spawner/contents/ranged`,
`slow_ranged`, `melee` and `small_melee`; the physical breeze contents record is not aliased. All
later connectors in that start see the same selected lookup. Alias construction/draw order remains
owned by the generic core/record leaf.

**Template and connector census:**

The 191 templates have one palette each, no duplicate coordinates and combined volume `275,099`:
`264,296` encoded cells plus `10,803` absent coordinates. They contain `106,790` raw air, `1,045`
raw water, 834 jigsaws, 46 NBT-bearing non-jigsaws including one save-mode structure block, and no
raw entities, structure void or DATA marker. They use 102 block IDs and 202 exact states:

| Template directory | Templates | Volume | Encoded | Absent | Raw air | Jigsaws | NBT non-jigsaws |
|---|---:|---:|---:|---:|---:|---:|---:|
| `chamber` | 73 | 141,007 | 135,073 | 5,934 | 62,381 | 398 | 2 |
| `chests` | 2 | 27 | 27 | 0 | 14 | 3 | 1 |
| `corridor` | 38 | 72,646 | 68,362 | 4,284 | 26,258 | 254 | 10 |
| `decor` | 28 | 98 | 98 | 0 | 19 | 28 | 7 |
| `dispensers` | 3 | 22 | 14 | 8 | 2 | 3 | 3 |
| `hallway` | 23 | 24,054 | 23,512 | 542 | 6,373 | 100 | 4 |
| `intersection` | 3 | 36,894 | 36,894 | 0 | 11,650 | 22 | 3 |
| `reward` | 2 | 54 | 54 | 0 | 9 | 2 | 2 |
| `spawner` | 19 | 297 | 262 | 35 | 84 | 24 | 14 |

Major raw counts are tuff bricks `106,683`, polished tuff `13,655`, waxed copper block `11,192`,
waxed oxidized copper `9,552`, chiseled tuff bricks `7,131`, waxed oxidized cut copper `2,351`,
grates `1,400`, waxed copper bulbs `923`, waxed oxidized stairs `792`, powder snow `395`, chiseled
tuff `203`, ladder `184`, light-gray glass `111`, magma `105`, pointed dripstone `102`, iron chain
`87`, and the air/water/jigsaw values above. Explicit ordinary-single air and water overwrite live
state because the record disables waterlogging preservation; absent coordinates do nothing.

All connector priorities are exact: selection `0/1/2 = 797/34/3`, placement `0/1/2/3 = 784/44/5/1`;
500 are aligned and 334 rollable. Final states are air `402`, tuff bricks `170`, waxed oxidized
copper `214`, waxed oxidized cut copper `22`, polished tuff `11`, waxed copper grate `7`, waxed
copper block `4`, and one each of chiseled tuff bricks, tripwire, lit waxed copper bulb and waxed
oxidized copper grate. Thus up to `107,192` raw/final air cells are destructive before
clip/protection/write gates. Pool fields span 36 identities: empty `195`, decor `182`, chamber
dispensers `116`, reward `37`, spawner connectors `36`, the chamber/corridor/addon graph and four
alias-only contents IDs; exact name/target counts and every priority are locked template data.

**Copper degradation and protection:**

Of 207 singles, 50 references (46 unique templates) select `trial_chambers_copper_bulb_degradation`;
157 use inline empty processors. `hallway/rubble_chamber` and `rubble_chamber_thin` are reachable in
both modes through the main/fallback pools. Jigsaw replacement runs first. The named list then
applies first-match position-derived rules to waxed copper bulbs: `<0.1` becomes lit unpowered waxed
oxidized; after that miss `<0.33333334` becomes lit unpowered weathered; after both misses `<0.5`
becomes lit unpowered exposed. Effective outcomes are `10%/30%/30%/30%`
oxidized/weathered/exposed/unchanged. The processor-capable templates contain 748 raw candidates—746
lit and two unlit rubble-chamber bulbs—while the sole connector-final lit bulb occurs only in an
inline-empty template.

Protection runs after degradation and drops any processed cell whose current live block is bedrock,
spawner, chest, End-portal frame, reinforced deepslate, trial spawner or vault. It therefore
preserves those live cells even against explicit air/material when the named list is used, and can
suppress NBT loads/seeds on a repeated placement; inline-empty paths have no such protection. The
ordinary-single initial structure-block ignore removes the lone save-mode block in
`chamber/addon/walkway_with_bridge_1` before configured processing. Its author/name/size bookkeeping
is inert payload, not a DATA marker or placed block entity.

**Placed NBT and caller RNG:**

The other 45 NBT cells are 12 chests, seven dispensers, four barrels, five decorated pots, one
hopper, 14 trial spawners and two vaults. The generic barrier/state/type/load transaction applies.
Chests, dispensers, barrels, pots and hopper implement `RandomizableContainer`, so every successful
resulting typed entity consumes caller `nextLong` and injects `LootTableSeed` even when raw NBT has
fixed items or no loot table: 29 possible draws across the complete corpus, in encoded placement
order. Trial spawners and vaults do not consume this seed draw. Named protection, chunk clipping,
failed writes or wrong/missing resulting types suppress the load and draw.

The 12 chests bind supply once, intersection once, reward three times and entrance seven times. Four
barrels are corridor loot, two intersection-barrel loot and one empty disposal inventory; the
disposal hopper is empty with zero cooldown. Seven dispensers are three chamber-loot, three
corridor-loot and one fixed upward water bucket in slot four. Four decor pots bind corridor-pot loot
with undecorated, flow, guster or scrape sherd configuration; one intersection pot fixes three
string and one flow sherd. These fixed item/shard arrays survive their otherwise unconditional
caller-seed draw.

**Trial-spawner and vault initial configs:**

Each of 14 content templates places one non-ominous `waiting_for_players` trial spawner with
normal/ominous holder references for breeze; zombie, husk and spider; skeleton, stray and bogged in
both ranged speeds; baby zombie, cave spider, silverfish and slime. The resolved 28 records share
spawn range `4`, default total/additional mobs `6/2`, default simultaneous mobs `2`, default
items-to-drop-when-ominous, normal consumables/key ejection at equal weight and ominous
key/consumables at `3:7`, except:

#### breeze

**Normal and ominous resolved differences:**

ticks `20`; simultaneous added/player `0.5`; total added/player `1`; normal simultaneous/total
`1/2`, ominous `2/4`

#### zombie, husk

**Normal and ominous resolved differences:**

ticks `20`; simultaneous `3`, added/player `0.5`; ominous uses melee equipment with zero slot-drop
chances

#### spider

**Normal and ominous resolved differences:**

normal simultaneous/total `3/6`; ominous `4/12`; ticks `20`, added/player `0.5`

#### skeleton, stray, bogged

**Normal and ominous resolved differences:**

ticks `20`; simultaneous `3`, added/player `0.5`; ominous uses ranged equipment with zero slot-drop
chances

#### slow skeleton, stray, bogged

**Normal and ominous resolved differences:**

ticks `160`; simultaneous `4`, added/player `2`; ominous uses ranged equipment with zero slot-drop
chances

#### baby zombie

**Normal and ominous resolved differences:**

ticks `20`; simultaneous default `2`, added/player `0.5`; `IsBaby=1`; ominous uses melee equipment
with zero slot-drop chances

#### cave spider, silverfish

**Normal and ominous resolved differences:**

normal simultaneous/total `3/6`; ominous `4/12`; ticks `20`, added/player `0.5`

#### slime

**Normal and ominous resolved differences:**

same counts/timing as preceding row; size-one weight `3`, size-two weight `1`

The raw full config omits runtime state, target cooldown and player range, resolving those latter
fields to `36,000` ticks and `14`. The normal vault is inactive/non-ominous with trial key and
reward loot; the ominous vault is inactive/ominous with ominous trial key and ominous reward loot.
Both resolve activation/deactivation ranges `4/4.5`, no display override and the default inclusive
player detector. Later trial-spawner detection/spawning/ominous conversion/ejection and vault
activation/unlocking/reward dispatch to `BLK-TRIAL-SPAWNER-001` and `BLK-VAULT-001`.

**Linked loot records:**

The exact data-only set is 13 `chests/trial_chambers/*`, three `dispensers/trial_chambers/*`, one
`pots/trial_chambers/corridor`, three trial equipment tables and five trial-spawner ejection/drop
tables. Ten template/vault NBT roots plus seven spawner-config roots are direct. The reward and
ominous-reward wrappers select their common/rare tables at `2:8`, add `1..3` common rolls, and admit
their unique table at probability `0.25/0.75`; their six children plus the shared base equipment
table complete a 24-record transitive closure. The standalone water-dispenser table is present but
unreferenced by the locked chamber payload, yielding 25 owned records. Every table's own random
sequence, rolls, weights, functions, item components and nested reference are queryable and remain
evaluated by `ITM-LOOT-001`.

**Branches and aborts:**

Three paired ranged groups, three melee and four small-melee aliases; 47 primary/fallback pools and
six Empty choices; weighted hallway/chamber/decor branches; connector
priority/attachment/collision/depth; raw/final air and water versus absent; inline versus
degradation/protection; clip/write/NBT/type/seed gates; 14 spawner and two vault initial configs;
direct/nested loot choices.

**Constants and randomness:**

Alias, pool, connector and piece selection use the structure stream. Copper degradation restarts
position-derived streams. Each successfully loaded randomizable container consumes one caller
`nextLong`; exact encoded block order is observable. Trial-spawner/vault runtime and loot evaluation
use their separate later owners.

**Side effects:**

Rigid encapsulated copper/tuff rooms with destructive air/water; position-stable bulb
oxidation/light variants and live protected cells; fixed traps/storage/pots; seeded deferred loot
containers; one of 14 configured trial-spawner contents and normal/ominous vaults; no raw entity
creation.

**Gates:**

Record/core/alias and pool/template availability; connector/collision/depth/range/padding; position
rules and live protected tag; current chunk clip; block/fluid write and resulting block-entity type;
config/loot registries.

**Boundary cases and quirks:**

Four pool IDs are valid aliases without JSON records. Main hallway weights dwarf all ordinary
connectors. A template can be protected/degraded through the main pool but destructive/unmodified
through fallback. Ordinary air and water are payloads, while 10,803 absent cells are not. The
save-mode structure block is ignored despite not being a DATA marker. Fixed inventories still
advance the caller stream. No template entity exists; mobs arise only from later trial-spawner
runtime.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-DATA-001`. Anchors: all 47 trial-chamber pool records; the
trial-chamber record alias list; `trial_chambers_copper_bulb_degradation`;
`features_cannot_replace`; all 191 templates; all 28
`data/minecraft/trial_spawner/trial_chamber/**/*.json` records; 25 owned loot records; generic
single/jigsaw/rule/protected/template block-NBT-liquid paths; `RandomizableContainer`;
`TrialSpawnerConfig`, `TrialSpawner.FullConfig` and `VaultConfig`.

**Test vectors:**

Query/decode 47 pools, the record aliases, processor/tag, 25 owned loot records and 28 spawner
configs; assert 213 entries/1,534 weight, exact fallback/order/weight/processor topology, four
aliases and all 191 exact locators. Decode every template; assert the 9-row census, 102 blocks/202
states, 834 exact connector fields, 46 NBT cells, zero
duplicate/structure-void/DATA-marker/entity/missing/unreferenced inputs and all exact raw payloads.
Replay fixed alias seeds, dominant/fallback/Empty choices, priorities, explicit air/water/absent
cells, every degradation/protection equality/live-state result, ignored save block, all 29 container
seed/no-seed paths, spawner/vault config loads, the 24-record transitive loot closure and standalone
water table through their generic owners.
