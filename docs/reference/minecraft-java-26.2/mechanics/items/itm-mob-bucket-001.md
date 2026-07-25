# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-MOB-BUCKET-001` — Live-mob buckets capture one entity state and release it after a subtype-specific empty transaction

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-DISPENSER-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-005`, `MOB-001`,
`MOB-004`, `MOB-AI-001`, `ENV-FLUID-001`, `CLI-001`, `CLI-006`, `CLI-UI-001`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked item/entity registration, bucket/container, capture, subtype state,
dispenser and client bytecode plus item reports, advancements, trades and assets close all seven
identities and their capture, release, state-transfer, persistence and projection joins.

**Applies when:**

A cod, salmon, pufferfish, tropical fish, axolotl, tadpole or sulfur cube is captured into its live
mob bucket; one of those buckets is used or dispensed; its component or bucket-entity payload is
patched; or the resulting item/entity is persisted, reloaded, traded, progressed or rendered.

**Authoritative state:**

| item | raw item ID | entity | protocol entity ID | contained fluid | empty sound |
|---|---:|---|---:|---|---|
| `pufferfish_bucket` | `1047` | `pufferfish` | `107` | water | `item.bucket.empty_fish` |
| `salmon_bucket` | `1048` | `salmon` | `110` | water | `item.bucket.empty_fish` |
| `cod_bucket` | `1049` | `cod` | `27` | water | `item.bucket.empty_fish` |
| `tropical_fish_bucket` | `1050` | `tropical_fish` | `137` | water | `item.bucket.empty_fish` |
| `axolotl_bucket` | `1051` | `axolotl` | `7` | water | `item.bucket.empty_axolotl` |
| `sulfur_cube_bucket` | `1052` | `sulfur_cube` | `130` | empty | `item.bucket.empty_sulfur_cube` |
| `tadpole_bucket` | `1053` | `tadpole` | `131` | water | `item.bucket.empty_tadpole` |

Every identity is a common, nondamageable `MobBucketItem`, has maximum stack size one, no direct
item-tag membership and a default empty `bucket_entity_data` component. The four fish buckets also
carry `food`: pufferfish/tropical fish have nutrition/saturation `1/0.2`, salmon/cod have
`2/0.4`. None carries `consumable`, so those food values do not make the bucket directly edible.
Sulfur cube is the sole dry subtype: it captures with an ordinary empty bucket and releases without
placing fluid. Every other subtype captures with an exact water bucket and contains water.

**Transition and ordering:**

### Entity capture

Fish and axolotl offer capture before their superclass interaction. Tadpole first handles food and
golden-dandelion age locking, then offers capture before its fish superclass. Sulfur cube handles
its priming, shearing and swallowable-item branches before capture; an ordinary empty bucket reaches
capture because it matches none of them.

The common capture helper first requires the entity alive and the held stack accepted by
`canBePickedUpWithBucket`: exact `water_bucket` for the six aquatic entities and exact `bucket` for
sulfur cube. Failure returns no result and leaves superclass interaction in control. Success then:

1. plays the subtype pickup sound at the mob with volume/pitch `1/1`;
2. constructs that mob's default bucket stack and writes its captured state;
3. replaces the hand through `ItemUtils.createFilledResult(input,player,filled,false)`;
4. on the server, triggers `filled_bucket` for a `ServerPlayer` with the filled stack;
5. drops any leash, discards the source mob and returns `SUCCESS`.

The six aquatic entities use respectively `item.bucket.fill_fish`,
`item.bucket.fill_axolotl` or `item.bucket.fill_tadpole`; sulfur cube uses
`item.bucket.fill_sulfur_cube`. This transaction runs on both logical sides except the criterion.
The `false` remainder mode deliberately bypasses creative stack preservation: an ordinary
one-count water/empty bucket is consumed and its filled result replaces the hand even for an
infinite-material player. For a component-patched input stack containing more than one, one is
consumed and the filled result is inserted or dropped if inventory insertion fails.

Common capture copies `custom_name` onto the filled item. It updates `bucket_entity_data` with
truthy `NoAI`, `Silent`, `NoGravity`, `Glowing`, `Invulnerable` and
`PersistenceRequired` flags and always writes `Health` as a float. False flags are omitted.
Release reads optional values; absent fields retain the newly created entity defaults, except a
present true persistence flag asserts persistence. The bucket payload does not serialize the
entity wholesale.

### Subtype state transfer

| subtype | additional captured state | release/default consequence |
|---|---|---|
| cod | none | only common bucket data round-trips |
| pufferfish | none | puff state is not captured; release starts at the new entity's state `0` |
| salmon | implicit `salmon_size` component | captured small/medium/large overrides the release spawn's weighted `30/50/15` finalize choice; a default componentless bucket keeps that choice |
| tropical fish | implicit pattern, base-color and pattern-color components | captured variant overrides finalize; a componentless bucket keeps finalize's 90% common-variant or 10% independent pattern/color choice |
| axolotl | implicit `axolotl_variant`; payload `Age`, `AgeLocked`, optional remaining `HuntingCooldown` | `BUCKET` finalize skips natural variant selection; a componentless/default payload yields default Lucy, age `0`, unlocked age and no hunting cooldown |
| tadpole | payload `Age` and `AgeLocked` | missing `Age` retains the newly constructed age `0`; missing lock is false; `fromBucket()` is permanently true and its setter is a no-op |
| sulfur cube | implicit optional `sulfur_cube_content`; payload lowercase `age` and `age_locked` | absorbed BODY stack, age and age lock round-trip; primed/fuse and other transient state do not, so release is unprimed |

The salmon weights total `95`. Tropical fish has 12 patterns and 16 colors for each of its two
color roles; finalize chooses from the 22 common variants on the `nextFloat()<0.9` branch and
otherwise draws all three roles independently. Captured implicit components apply after
`finalizeSpawn` and before bucket payload loading, so they authoritatively replace those random
choices. Axolotl age/cooldown, tadpole age and sulfur age payload then apply after implicit
components.

### Held release and fluid admission

Use raycasts blocks with fluid context `NONE`; a miss returns `PASS`. A block hit computes clicked
position, face and adjacent position, then requires both `level.mayInteract(player,clicked)` and
`player.mayUseItemAt(adjacent,face,stack)`, otherwise it returns `FAIL`. When the clicked block is
a `LiquidBlockContainer` and the bucket content is exactly water, the clicked position is the
placement target; every other case targets the adjacent position.

Sulfur cube's empty-fluid override plays its subtype empty sound at the target and immediately
returns true without inspecting or changing the target. A water subtype delegates target
admission and placement to `BucketItem`: it requires flowing-fluid content and either replaceable
terrain or an accepting liquid container, can recursively retry one position along the hit face,
destroys a replaceable nonliquid target with drops, and writes the legacy source-fluid block with
flags `11`. A water-accepting container receives the source directly. Failed admission/write
returns false.

`ENV-FLUID-001` owns the exact water-evaporation predicate and particles. At this join, successful
evaporation is still successful emptying: it plays extinguish/smoke effects, writes no water and
then continues into mob release. The mob-specific empty-sound override plays in `NEUTRAL` at
volume/pitch `1/1` and emits no `FLUID_PLACE`, including ordinary water placement.

After any successful emptying, `checkExtraContent` runs. On the server it calls entity spawn and
then unconditionally emits `ENTITY_PLACE` at the target with the player as source. Spawn creates
the mapped mob with reason `BUCKET`, enables the create helper's downward-position adjustment and
disables its moved-up collision expansion, calls `finalizeSpawn`, applies default stack
configuration, copies and loads `bucket_entity_data`, and sets `fromBucket=true`. A nonnull mob is
offered through `addFreshEntityWithPassengers` and then plays its ambient sound; insertion success
is not observed. Null creation still leaves the unconditional `ENTITY_PLACE`.

A server player using any water subtype next triggers `placed_block` with the original filled
stack and target; dry sulfur does not. The player then receives the item-used statistic. Survival
consumes the filled bucket and returns one new plain empty bucket. Infinite-material release
returns the original filled stack, allowing repeated spawns. Success is returned with that
transformed hand stack. A failed water empty returns `FAIL` without spawn, event, statistic or
transformation; dry sulfur cannot reach that failure after permission admission.

### Dispenser release

All seven share the filled-container dispenser behavior. It calls
`emptyContents(null,serverLevel,front,null)`. Success calls
`checkExtraContent(null,...)` and consumes the selected bucket through the dispenser remainder
transaction, producing one plain empty bucket. Water evaporation therefore still releases its mob,
and dry sulfur always releases directly in front. Failure delegates to nested default ejection,
including the duplicate inner/outer event behavior owned by `ITM-DISPENSER-001`. Dispenser release
does not run player permissions, item-used statistics or `placed_block`; `ENTITY_PLACE` has a null
source.

### Persistence, progression and client projection

Released cod, salmon, pufferfish and tropical fish persist synchronized `FromBucket`; axolotl and
sulfur cube persist their corresponding from-bucket fields. Fish/axolotl require custom persistence
when that flag is true and refuse distance removal; sulfur cube likewise requires persistence when
from a bucket or carrying BODY content. Tadpole always reports from-bucket, so its inherited fish
persistence path treats every tadpole as persistent. Ordinary entity save data owns the restored
variant, age, body content and other ongoing state after release; the capture stack owns only the
components and payload stated above.

There are no locked recipes or direct loot-table emissions for the seven items. A novice fisherman
trade record sells one default cod bucket for three emeralds with maximum uses `16` and reputation
discount `0.05`; its level-one trade set selects two offers. The wandering trader common set
selects five offers and contains separate three-emerald, discount-`0.05` records for default
tropical-fish and pufferfish buckets. No locked trade directly emits the other four.

The `filled_bucket` criterion advances Tactical Fishing for any one of the four fish buckets, and
separate advancements require axolotl and tadpole buckets. The axolotl advancement is a parent of
the later axolotl-assistance advancement. Sulfur cube capture has no scoped built-in advancement.
Trading or command-created default buckets do not imply a prior `filled_bucket` trigger.

Every identity has one direct item model; captured components do not select a different model.
Tropical-fish pattern tooltip projection uses the two color components: one of 22 predefined
variants emits its predefined gray italic name, while any other combination emits gray italic
pattern then one or two color names. Sulfur content emits a gray italic translated line naming the
absorbed stack. Salmon size and axolotl variant add no dedicated tooltip line. Tools & Utilities
orders water bucket, cod, salmon, tropical fish, pufferfish, axolotl, tadpole, sulfur cube, then
lava bucket.

**Branches and aborts:**

Seven identities; water/empty/wrong bucket; alive/dead; client/server; ordinary/patched stack count;
leashed/unleashed; every common flag/health; every subtype component/payload present/absent;
captured/default/randomized variant; permission/miss/container/adjacent/recursive target; dry,
placed, evaporated and failed water; null/nonbucketable/inserted/rejected spawn; survival/infinite
materials; held/dispenser; trade/progression/tooltip/resource state.

**Constants and randomness:**

Item IDs `1047..1053`; entity IDs `107/110/27/137/7/130/131`; maximum stack `1`; pickup/empty
volume/pitch `1/1`; salmon weights `30/50/15`; tropical common probability `0.9`, 22 common
variants, 12 patterns and 16 colors; fluid-write flags `11`; puffer/tropical food `1/0.2`,
salmon/cod `2/0.4`. The create helper draws one yaw float and subtype `finalizeSpawn` consumes its
branch-dependent random choices before implicit component override.

**Side effects:**

Capture sound, hand/inventory/drop transformation, filled-bucket criterion, leash drop and source
discard; fluid write/destruction or evaporation effects; empty sound, mob creation/configuration,
insertion, ambient sound, entity-place event, optional placed-block criterion, item-used statistic
and held/dispenser remainder; durable from-bucket/entity state, trades, advancement state, tooltip
and model projection.

**Gates:**

Entity life and exact capture item; subtype interaction precedence; logical side; component/payload
presence; hit and both permission checks; liquid admission, container, recursion, write and
evaporation; entity creation; player ability; dispenser selection; current trade, advancement,
data-component and resource snapshots.

**State read/written:**

Reads held stack/hand/player ability, mob life/leash/name/flags/health/subtype state, hit/permission/
target block/fluid/dimension predicate, bucket components, level difficulty/RNG and current data/
resources. Writes hand/inventory/drop, source removal, fluid/block/effect state, new mob position/
rotation/components/payload/from-bucket state, world entity/event/statistic/progress, dispenser
slot/remainder and durable entity/item data.

**Failure behavior:**

Wrong bucket or dead mob falls through to superclass interaction. A held miss passes and permission
denial fails. Failed water admission/write fails without spawn or consumption. Evaporation is not
failure. Null entity creation still emits `ENTITY_PLACE` and completes item/stat consequences;
failed entity insertion is ignored and ambient sound is still invoked. Dispenser empty failure
ejects through nested default behavior. Absent payload fields retain the stated defaults rather
than reconstructing uncaptured entity state.

**Persistence boundary:**

Filled stacks persist identity, implicit subtype components, custom name and `bucket_entity_data`
generically. Capture itself does not resume. Released entities persist their ordinary fields and
from-bucket marker; reload does not recover failed insertion or replay sounds/events/statistics.
Data reload replaces trades and advancements without altering already captured payloads. Resource
reload replaces item models, textures, translations and tooltip presentation without changing
authority.

**Boundary cases and quirks:**

Creative capture consumes its water/empty bucket, but creative release retains the filled bucket.
Mob bucket raycasts ignore fluids. A water bucket can release a mob into an evaporating dimension
without placing water. Mob-specific empty sounds suppress the base bucket's `FLUID_PLACE` event.
Sulfur cube succeeds without fluid or `placed_block`. Spawn/event/stat/item completion ignores
entity insertion success, and null creation still emits the event. Puffer inflation and sulfur
priming do not round-trip. Captured variants override finalize randomness, while componentless
salmon/tropical buckets expose it. Tadpole's from-bucket state is constant true.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.BucketItem`;
`net.minecraft.world.item.MobBucketItem`; `net.minecraft.world.item.ItemUtils`;
`net.minecraft.world.item.DispensibleContainerItem`;
`net.minecraft.core.dispenser.DispenseItemBehavior`;
`net.minecraft.world.entity.EntityType`; `net.minecraft.world.entity.Bucketable`;
`net.minecraft.world.entity.animal.fish.AbstractFish`;
`net.minecraft.world.entity.animal.fish.Cod`;
`net.minecraft.world.entity.animal.fish.Salmon`;
`net.minecraft.world.entity.animal.fish.Pufferfish`;
`net.minecraft.world.entity.animal.fish.TropicalFish`;
`net.minecraft.world.entity.animal.axolotl.Axolotl`;
`net.minecraft.world.entity.animal.frog.Tadpole`;
`net.minecraft.world.entity.monster.cubemob.SulfurCube`;
`net.minecraft.world.item.component.SulfurCubeContent`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:{item,entity_type}`;
`reports/minecraft/components/item/{pufferfish,salmon,cod,tropical_fish,axolotl,sulfur_cube,tadpole}_bucket.json`;
`data/minecraft/advancement/husbandry/{tactical_fishing,axolotl_in_a_bucket,kill_axolotl_target,tadpole_in_a_bucket}.json`;
`data/minecraft/{villager_trade,tags/villager_trade,trade_set}/{fisherman,wandering_trader}/**/*.json`;
`assets/minecraft/{items,models/item,textures/item}/{pufferfish,salmon,cod,tropical_fish,axolotl,sulfur_cube,tadpole}_bucket.*`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-DISPENSER-001`; `ITM-LOOT-001`;
`ITM-ADVANCEMENT-001`; `ENT-LIFECYCLE-001`; `MOB-AI-001`; `ENV-FLUID-001`;
`CLI-UI-001`; `CLI-EFFECT-001`; `EXP-ITM-028`.

**Test vectors:**

Capture every alive/dead/leashed subtype with correct/wrong buckets on both sides and in both
ability modes; assert exact common and subtype state, criterion, hand result and source removal.
Release default/captured/patched stacks across misses, both permissions, ordinary/container/
recursive/failing targets, evaporation and dry sulfur; force null and rejected entity insertion.
Repeat from every dispenser facing. Persist all components/payload/from-bucket states, reload data
and resources, exercise trades/advancements, and inspect every tooltip/model/tab projection.

**Limits:**

This leaf does not duplicate generic interaction packets, fluid propagation/container admission,
dispenser scheduling/remainder insertion, entity AI/damage, trade selection, advancement listener,
component codec or resource-pack algorithms. Those remain with the cited owners; this rule fixes
the seven identities and the exact capture/release/state-transfer joins.
