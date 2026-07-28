# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-VILLAGER-001` — Villagers bind profession, trade, reputation, food and village POIs through one scheduled Brain

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-005`, `ENT-DAMAGE-001`, `ENT-BLOCK-001`,
`ENT-DAMAGE-REDUCE-001`, `ENT-KNOCKBACK-001`, `ENT-006`,
`ENT-EFFECT-001`, `ENT-007`, `ENT-DEATH-001`,
`ENT-ENTITY-DROPS-001`, `ENT-IRON-GOLEM-001`, `MOB-001`,
`MOB-AI-001`, `MOB-002`, `MOB-SPAWN-001`, `MOB-003`,
`MOB-DESPAWN-001`, `MOB-BREED-001`, `MOB-RAID-001`, `ITM-001`,
`ITM-CONTAINER-001`, `ITM-CONTAINER-CONTROL-001`, `ITM-LOOT-001`,
`ITM-ADVANCEMENT-001`, `ITM-BREAD-001`,
`BLK-OVERWORLD-CROP-001`, `BLK-TORCHFLOWER-CROP-001`,
`BLK-PITCHER-CROP-001`, `BLK-BELL-001`, `PLY-AUTOJUMP-001`,
`WGEN-005`, `WGEN-DIMENSION-001`, `WGEN-PORTAL-001`,
`WGEN-JIGSAW-VILLAGES-001`, `WGEN-STRUCTURE-IGLOO-001`, `CLI-001`,
`CLI-006`, `CLI-UI-001`, `CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, complete `Villager`,
`AbstractVillager`, data/profession/type, Brain package and direct behavior
implementations, merchant and gossip state machines, all built-in production,
data, compatibility and client projection close protocol entity ID `140`.

**Applies when:**

`minecraft:villager` is created, finalized, aged, scheduled, assigned a POI
or profession, trading, restocking, displaying or sharing items, gossiping,
breeding, summoning an Iron Golem, struck by lightning, produced by curing,
saved, loaded, synchronized, heard or rendered.

**Authoritative state:**

Protocol entity ID `140` constructs an ordinary, Peaceful-available
`MISC` Mob. Registration fixes adult dimensions `0.6x1.95`, explicit eye
height `1.62`, client tracking range `10` and default update interval `3`.
A baby instead uses scalable dimensions `0.49x0.98` and eye height `0.63`.

The registered attributes are the living/Mob defaults with maximum health
`20`, follow range `16` and raw movement speed replaced by `0.5`. Villagers
register no attack attribute and begin with XP reward `0`. Construction sets
Fire-In-Neighbor path malus `16`, Fire malus `-1`, door opening and floating
true, required navigation path length `48`, loot pickup true and an eight-slot
inventory. They reject leashes and never remove themselves for distance.

Entity, Living Entity and Mob occupy synchronized metadata slots `0..15`.
`AgeableMob` adds Boolean baby slot `16` and Boolean age-locked slot `17`;
Villager is a direct member of `cannot_be_age_locked`, so ordinary component
application cannot enable the latter. `AbstractVillager` and Villager add:

| Slot | Serializer | Fresh value | Meaning |
|---:|---|---|---|
| `18` | ID `1`, `INT` | `0` | unhappy display counter |
| `19` | ID `18`, `VILLAGER_DATA` | plains, none, level `1` | dynamic type holder, profession holder, level VarInt |
| `20` | ID `8`, `BOOLEAN` | false | biome type has been finalized |

The unhappy counter is synchronized but not saved. Each ordinary tick
decrements a positive value by one. Client events `12`, `13`, `14` and `42`
respectively emit five Heart, Angry Villager, Happy Villager or Splash
particles; each particle consumes three Gaussian velocity draws and its
three random position draws.

`VillagerData` clamps level only to a minimum of `1`; it does not clamp a
value above `5`. Its NBT codec defaults missing type/profession/level to
plains/none/`1`; its stream codec carries the two ordered dynamic holders
then a signed VarInt. Level-up thresholds are:

| Current level | Minimum XP label | XP required to advance |
|---:|---:|---:|
| `1` | `0` | `10` |
| `2` | `10` | `70` |
| `3` | `70` | `150` |
| `4` | `150` | `250` |
| `5` or outside `1..4` | `0` | cannot advance |

Changing profession through `setVillagerData` invalidates the cached offers.
Changing type or level while retaining profession does not. The implicit
component `minecraft:villager/variant`, raw component ID `82`, reads/writes
only the type holder; applying it also sets finalized true.

### Type and profession registries

The seven ordered Villager types are raw IDs `0..6`: desert, jungle, plains,
savanna, snow, swamp and taiga. First finalization maps keyed biomes as
follows and uses plains for every other or unkeyed biome:

| Type | Exact keyed biomes |
|---|---|
| desert | Badlands, Desert, Eroded Badlands, Wooded Badlands |
| jungle | Bamboo Jungle, Jungle, Sparse Jungle |
| savanna | Savanna Plateau, Savanna, Windswept Savanna |
| snow | Deep Frozen Ocean, Frozen Ocean, Frozen River, Ice Spikes, Snowy Beach, Snowy Taiga, Snowy Plains, Grove, Snowy Slopes, Frozen Peaks, Jagged Peaks |
| swamp | Swamp, Mangrove Swamp |
| taiga | Old Growth Spruce Taiga, Old Growth Pine Taiga, Windswept Gravelly Hills, Windswept Hills, Taiga, Windswept Forest |

The 15 ordered professions are raw IDs `0..14`: none, armorer, butcher,
cartographer, cleric, farmer, fisherman, fletcher, leatherworker, librarian,
mason, nitwit, shepherd, toolsmith and weaponsmith. The 13 working
professions bind, in that order, Blast Furnace, Smoker, Cartography Table,
Brewing Stand, Composter, Barrel, Fletching Table, Cauldron, Lectern,
Stonecutter, Loom, Smithing Table and Grindstone POIs and their matching
work sounds/trade-set families.

`none` holds no job but may acquire every POI in
`#minecraft:acquirable_job_site`; `nitwit` holds and acquires none. Farmer
alone requests Wheat, Wheat Seeds, Beetroot Seeds and Bone Meal and treats
Farmland as a secondary POI. Every other requested-item and secondary-POI
set is empty.

### Brain construction and schedule

The Brain provider installs, in order, Nearest Living Entities, Nearest
Players, Nearest Items, Nearest Bed, Hurt By, Villager Hostiles, Villager
Babies, Secondary POIs and Golem Detected sensors. Villager Hostiles accepts
only these exact type/radius pairs, using inclusive squared distance after
visibility selection:

| Radius | Types |
|---:|---|
| `8` | Drowned, Husk, Vex, Zombie, Zombie Villager |
| `10` | Vindicator, Zoglin |
| `12` | Evoker, Illusioner, Ravager |
| `15` | Pillager |

Golem detection scans every `200` ticks and, upon finding any Iron Golem in
Nearest Living Entities, writes `GOLEM_DETECTED_RECENTLY=true` with TTL
`599`.

Babies register Play instead of Work. Adults register Work with a required
present Job Site. Core is always present; Meet requires Meeting Point.
Rest, Idle, Panic, Pre-Raid, Raid and Hide are also installed in that order.
The initial active activity is selected immediately after registration from
the current environment attributes, game time and position.

Adult schedule keyframes are Idle `10`, Work `2000`, Meet `9000`, Idle
`11000`, Rest `12000`. Baby keyframes are Idle `10`, Play `3000`, Idle
`6000`, Play `10000`, Rest `12000`. Both come from the reloadable
`villager_schedule` timeline, period `24,000`, against the Overworld clock.
Growing across the age boundary stops and repacks the old Brain, creates a
new provider for the adult package and immediately reselects activity.
Loading similarly refreshes the Brain on the server.

The complete activity package is:

| Activity | Priority and behavior composition |
|---|---|
| Core | `0`: Swim `0.8`, Interact With Door, Look sink `45..90`, panic trigger, wake, react to bell, set raid status, validate Job and Potential Job POIs; `1`: Move To Target; `2`: POI competitor scan; `3`: follow trading player at `0.5`; `5`: wanted item at `0.5`, radius `4`; `6`: acquire job/potential job; `7`: go to potential job at `0.5`; `8`: yield job at `0.5`; `10`: acquire unoccupied Home with event `14`, acquire adult Meeting Point with event `14`, assign profession, reset profession |
| Work | minimal look; `5`: weighted Run-One of work-at-POI `7`, stroll around job `2`, stroll to job `5`, stroll to secondary POI `5`, harvest `2` for Farmer/`5` otherwise, use Bone Meal `4` for Farmer/`7` otherwise; `10`: show trades and look/interact with Player radius `4`; `2`: walk from Job memory, close `9`, far `100`, unreachable `1200`; `3`: hero gift; `99`: update schedule |
| Play | `0`: Move To Target duration `80..120`; full look; `5`: play tag, then when visible-baby memory is absent a weighted Run-One of Villager `2`, Cat `1`, village stroll `1`, look-target walk `1`, bed jump `2`, do nothing `2`; `99`: update schedule |
| Rest | `2`: walk from Home, close `1`, far `150`, unreachable `1200`; `3`: validate Home and sleep; `5`: when Home absent weighted closest-home `1`, indoor walk `4`, closest-village `2`, do nothing `2`; minimal look; `99`: update schedule |
| Meet | `2`: shuffled stroll-around-meeting/bell-socialize weights `2/2`, plus walk from Meeting Point close `6`, far `100`, unreachable `200`; `10`: show trades and Player interaction; `3`: hero gift, validate Meeting Point and ordered one-shot Villager trade; full look; `99`: update schedule |
| Idle | `2`: weighted Villager interaction `2`, breeding interaction `1`, Cat `1`, village stroll `1`, look-target walk `1`, bed jump `1`, do nothing `1`; `3`: hero gift, Player interaction, show trades, ordered Villager trade and Villager breeding; full look; `99`: update schedule |
| Panic | `0`: calm-down test; `1`: flee nearest hostile then hurt-by entity at speed `0.75`, distance `6`; `3`: village stroll at `0.75`, horizontal/vertical `2/2`; minimal look |
| Pre-Raid | `0`: ring bell, then shuffled meeting walk at `0.75` weight `6` or village stroll at `0.75` weight `2`; minimal look; `99`: reset raid status |
| Raid | `0`: on victory shuffled sky-seeing at `0.5` weight `5` or village stroll `0.55` weight `2`, then celebrate `600/600`; `2`: while active locate hiding place radius `24`, speed `0.7`, close `1`; minimal look; `99`: reset raid status |
| Hide | `0`: hidden state `15/3`; `1`: locate hiding place radius `32`, speed `0.625`, close `2`; minimal look |

Full look weights Cat/Villager/Player/Creature/Water Creature/Axolotl/
Underground Water Creature/Water Ambient/Monster/Do Nothing as
`8/2/2/1/1/1/1/1/1/2`, all entity radii `8`; minimal look weights
Villager/Player/Do Nothing `2/2/8`. Run-One, Gate, activity and same-priority
arbitration retain `MOB-AI-001` ordering.

### POI acquisition and profession transitions

Acquire-POI behaviors schedule their first scan after `0..19` ticks and
later scans after `20..39`. They take at most five closest POIs with space
inside radius `48`, apply the caller predicate, path to the set and reserve
the reachable path target. Failed candidates receive jittered linear retry
delays increasing by `40..79`, capped at `400`, and expire after a strict
`400` ticks since their previous attempt.

Job and Meeting acquisition are adult-only. Home acquisition permits babies
but requires a live Bed with `occupied=false`. Job acquisition requires both
Job Site and Potential Job Site absent and reserves the POI into Potential
Job Site. The potential-job movement behavior runs for at most `1200` ticks;
on stop it releases an existing POI in the remembered dimension and erases
the memory.

Assignment requires the Villager center within `2` of the Potential Job
Site, except a Structure-finalized Villager may bypass distance during its
first Brain tick. It erases Potential, writes Job and broadcasts happy event
`14` before profession selection. A non-none profession is retained.
Otherwise the first registry-ordered profession whose held-job predicate
matches the POI is installed, cached offers are invalidated and the Brain is
refreshed. The Structure bypass flag is cleared immediately after that first
Brain tick even if assignment did not occur.

With no Job memory, a non-none/non-nitwit level-`1` Villager with XP `0`
resets to none and refreshes. A POI competitor scan compares living nearby
Villagers remembering the same GlobalPos with a matching profession: larger
XP wins; an equal comparison selects the later reducer operand. Each loser
erases Job Site without directly releasing the POI.

An unemployed adult holding a Potential Job yields it to the first living
nearby Villager that has no Potential memory, has a matching profession and
either remembers that exact Job or can reach the POI while having no Job.
The yielding Villager clears walk/look/potential; the recipient gets a walk
target and, when it lacked Job, the Potential memory.

Walking from Job, Home or Meeting memory releases the POI and erases memory
when the dimension differs or the strict unreachable duration has elapsed.
When farther than the configured limit it makes up to `1000` random
toward-target samples within `15x7`; failure also releases/erases. Death
attempts release Home, Job, Potential Job and Meeting reservations in their
remembered levels only when the live POI still matches the memory's current
predicate, then continues ordinary death.

### Work, display and hero gifts

Work-at-POI requires the matching dimension and center distance strictly
inside `1.73`. It cannot start until at least `300` ticks since its behavior
instance's last check, then requires `level.random.nextInt(2)==0`; a passed
check stores the current time. Start writes Last Worked At POI, looks at the
site, plays the profession work sound, uses the workstation and finally
checks/restocks.

Farmer work first makes at most three Bread from three Wheat each when
existing Bread is at most `36`, inserting the result and dropping overflow.
It then extracts a full Composter and scans inventory slots `7..0` for Wheat
Seeds/Beetroot Seeds, preserving the first ten counted of each identity and
offering at most `20` items until level `7`; event `1500` reports whether
the state identity changed. Crop harvesting and Bone Meal retain the cited
crop owners.

Show-Trades runs only for a living adult and living Player interaction
target at squared distance at most `17`. It compares only main-hand item
identity, collects assembled results of non-out-of-stock offers whose A or B
cost shares that identity, displays the first, then cycles every `40` ticks
for up to `900`. It writes the displayed result into the Villager main hand
with drop chance `0`; stop or no results clears that hand and restores drop
chance `0.085`.

Each hero-gift behavior begins with counter `600`, decremented only on
eligible start checks while the nearest visible Player has Hero of the
Village. It targets that Player, walks within block-position distance
strictly below `5`, waits strictly more than `20` ticks after start, then
evaluates and throws the baby, unemployed or one of 13 profession gift
tables. Stop clears interaction/walk/look and samples the next delay as
`600+nextInt(6001)`.

### Player trading and offers

Holding the Villager Spawn Egg, being dead, already trading or sleeping
delegates interaction to the generic path. Otherwise:

- a baby sets unhappy to `40`, plays Villager No only on the server and
  returns Success;
- an adult client returns Success without reading offers;
- an adult server lazily creates offers. Main-hand interaction always awards
  Talked To Villager and, for an empty list, also becomes unhappy;
- an empty list returns Consume and opens nothing. An off-hand empty-list
  interaction neither awards the statistic nor becomes unhappy;
- a nonempty list applies prices, binds the exact Player and opens merchant
  menu raw ID `19` with title from the profession and the VillagerData level.

The menu remains valid only for that exact trading Player while the Villager
is alive and the Player remains within ordinary interaction range plus `4`.
Teleport and death stop trading. Changing to profession none stops an open
trade on the next server AI step. Closing clears the player and, server-side,
resets every offer's special-price difference.

Offer generation exists only on the server. Missing cached offers construct
an empty list and call `updateTrades`; a client call throws. The profession
maps levels `1..5` to 65 exact trade sets. All set `allow_duplicates=false`
and use amount `2`, except Librarian level `5` uses amount `3`. Their
candidate counts are:

| Profession | L1 | L2 | L3 | L4 | L5 |
|---|---:|---:|---:|---:|---:|
| Armorer | 5 | 3 | 6 | 3 | 3 |
| Butcher | 4 | 3 | 2 | 1 | 1 |
| Cartographer | 2 | 8 | 3 | 16 | 2 |
| Cleric | 2 | 2 | 2 | 3 | 2 |
| Farmer | 5 | 3 | 2 | 2 | 2 |
| Fisherman | 4 | 3 | 2 | 1 | 6 |
| Fletcher | 3 | 2 | 2 | 2 | 3 |
| Leatherworker | 3 | 3 | 2 | 2 | 2 |
| Librarian | 3 | 3 | 3 | 4 | 2 |
| Mason | 2 | 2 | 7 | 33 | 2 |
| Shepherd | 5 | 37 | 21 | 22 | 1 |
| Toolsmith | 5 | 1 | 6 | 4 | 2 |
| Weaponsmith | 3 | 1 | 2 | 3 | 2 |

The 65 sets reference 291 exact `villager_trade` records. Each set builds a
Villager-Trade loot context with Origin, This Entity and Additional Cost
Component Allowed, using its named random sequence. No-duplicate selection
removes the random candidate before asking it for an offer; null offers
therefore consume a candidate but not an output. Selection ends at requested
amount or an empty candidate list. The unused duplicates-allowed path samples
with replacement but removes only a null-producing candidate.

Starting trade applies total weighted gossip reputation `r` by adding
`-floor(r*priceMultiplier)` to each offer. Hero of the Village amplifier `a`
then adds `-max(floor((0.3+0.0625*a)*baseCostA),1)`. Offer-owned clamping and
payment remain with the merchant/container rules.

Completing a trade increments uses, resets ambient timing, adds the offer XP
to Villager XP, remembers the current trading Player and triggers the Trade
criterion for a `ServerPlayer`. It samples orb XP `3+nextInt(4)`. If the
current level can advance and total XP meets its next threshold, it resets a
merchant timer to `40`, marks level-up and adds `5` to the orb. The orb is
inserted only when the offer rewards XP; insertion success is ignored.

The next server AI step turns a remembered trader into one Trade reputation
event and happy event `14`, then clears it. The level timer decrements only
while not trading. At zero it increments level, appends the new level's
offers to the retained list, clears the flag and adds Regeneration
`200` amplifier `0`; delaying closure delays this entire countdown.

Trade-result updates are server-only and sound-throttled by ambient timing:
a nonempty result selects Villager Yes, an empty result Villager No.
`overrideOffers` and `overrideXp` are deliberately no-ops, while the concrete
conversion helper `setOffers` replaces the cache.

### Demand and restocking

Every Villager reports `canRestock=true`. A Work start calls
`shouldRestock`. It treats either strict `gameTime > lastRestock+12000` or a
strict increase in the reloadable Overworld-day period as a new day. The
day comparison is disabled while its transient previous-day value is `0`.
A new day first sets last-restock time to now, catches demand up for
`2-restocksToday` updates, resetting all uses once when that value is
positive, resends an open menu, then sets today's count to `0`.

Restock is allowed at count `0`, or at count `1` only after strict
`gameTime > lastRestock+2400`, and only when at least one offer needs it.
Actual restock updates demand once, resets uses on every offer, resends the
menu, stores current game time and increments the count. Resend requires a
bound Player and a nonempty list and carries level, XP, progress true and
restock true.

### Food, sharing and breeding

Food points are Bread `4` and Potato, Carrot and Beetroot `1` each.
`canBreed` requires internal food plus inventory points at least `12`, not
sleeping and age exactly `0`. Eating scans slots `0..7`, consumes supported
items one at a time until internal food reaches at least `12`, then digestion
subtracts `12`; Bread can leave `0..3` excess. Inventory "excess" is at least
`24`, "wants more" is below `12`, independently of internal food.

The pickup predicate requires inventory capacity and either
`#minecraft:villager_picks_up` or the current profession's requested set.
The first tag expands the six plantable seeds/pods plus Bread, Wheat and
Beetroot. Farmers additionally request Wheat, Wheat Seeds, Beetroot Seeds
and Bone Meal. Pickup uses the shared inventory-carrier transaction.

Villager-to-Villager Trade behavior locks gaze/walk within `2`, acts only at
squared distance at most `5`, gossips, then may throw up to one stack for
each applicable category in this order: food, Farmer Wheat, cached requested
items desired by the other profession but not this one. For the first
matching slot, a count above half max sends floor half; otherwise a count
above `24` sends `count-24`; smaller counts send nothing. The thrown stack
is newly constructed from item identity, not copied components. A Farmer
shares food regardless of target need and shares Wheat above `32`; another
profession shares excess food only when the target inventory wants food.

Villager breeding requires a valid visible Villager Breed Target and both
`canBreed`. Start locks both at speed `0.5`, close `2`, broadcasts generic
breeding event `18`, samples `275+nextInt(50)` and runs no longer than
`350`. Each tick within squared distance `5` relocks them; before birth it
has a one-in-35 Heart event `12` for both. At the birth timestamp it first
eats/digests both parents, then reserves a reachable Home POI within `48`.
No bed produces angry event `13` for both with no food refund.

A child type uses one `nextDouble`: below `0.5` selects the biome at the
first parent's current position, below `0.75` selects that parent's type,
otherwise the partner's. The child is created with profession none and
finalized true. Both parents become age `6000`, child age `-24000`; it is
snapped to the first parent and offered with passengers, with insertion
result ignored. Heart event `12` and the reserved Home memory follow even
after a rejected insertion. Only a null child releases the reserved POI;
the concrete Villager factory does not return null.

### Gossip, reputation and Golem voting

Gossip types store an unweighted positive value, multiply it for reputation,
cap ordinary additions and decay as follows:

| Type | Weight | Max value | Daily decay | Transfer decay |
|---|---:|---:|---:|---:|
| major negative | `-5` | `100` | `10` | `10` |
| minor negative | `-1` | `200` | `20` | `20` |
| minor positive | `1` | `25` | `1` | `5` |
| major positive | `5` | `20` | `0` | `20` |
| trading | `1` | `25` | `2` | `20` |

Values below `2` are removed. Daily decay happens at most once when current
game time reaches the saved last-decay time plus `24,000`; it does not catch
up missed days. A fresh zero timestamp is initialized without decay.

Gossip between two Villagers is rejected while the supplied timestamp is
inside either participant's `[lastGossip,lastGossip+1200)` interval.
Otherwise the receiver samples ten times from the sender's unpacked entries,
weighted by absolute weighted reputation. Repeated selections collapse by
entry identity, so fewer than ten can transfer. Each selected value loses
its transfer decay, values below `2` are discarded, and an existing
receiver entry keeps the larger rather than summing. Both cooldown
timestamps update, then the receiver attempts a five-Villager Golem vote.

Reputation events add: cure `20` major positive plus `25` minor positive;
trade `2` trading; hurt `25` minor negative; killed `25` major negative.
Hurt records the event before generic attacker state and, for a living
Villager hurt by a Player, broadcasts angry event `13`. Death sends killed
reputation to every visible current `ReputationEventHandler` witness using
the damage source entity as perpetrator before POI release and generic
death.

A Villager wants a Golem only when Last Slept exists and
`gameTime-lastSlept < 24000`, and Golem Detected Recently is absent. A voter
search inflates its box by `10` on all axes, takes at most five wanting
Villagers and requires the caller-supplied quorum. Gossip uses quorum `5`.
Success uses ten Legacy Iron-Golem spawn attempts with horizontal `8`,
vertical `6` and collision checking false, then marks every Villager in the
original unfiltered box detected for `599`; failure changes no memory.
`ENT-IRON-GOLEM-001` owns the placement search and resulting Golem.

Stopping sleep writes Last Woken to current game time; the sleeping behavior
owns Last Slept.

**Transition and ordering:**

Each server AI step ticks the Brain, clears the one-shot Structure profession
flag, advances a nontrading level timer, publishes the previous trade's
reputation/particles, consumes one raid-splash `nextInt(100)` when AI is
enabled, closes a none-profession trade, then runs generic Mob AI. Ordinary
`tick` finishes generic ticking before unhappy decrement and gossip decay.

Raid splash event `42` requires a zero draw and a raid at the current
position that is active and not over. Within an admitted custom AI step, the
explicit NoAI check guards only this subtype draw; generic Mob effective-AI
admission normally prevents the entire custom step for NoAI entities.

**Other production and spawning:**

Spawn placement is registered as On Ground,
Motion-Blocking-No-Leaves with `Mob.checkMobSpawnRules`, but all 66 bundled
biomes contain zero Villager spawn rows. No Trial Spawner config names
Villager.

The complete 1,212-template census contains 16 literal Villagers:

- five village families each contain one baby, one nitwit and one unemployed
  resident, for `15`; placement finalizes them with Structure reason,
  preserves their finalized stored biome type/profession and enables the
  one-tick profession-distance bypass;
- the Igloo basement contains one persistent plains Cleric with two stored
  novice offers; its template path deliberately skips finalization.

Breeding is the direct live child producer. Zombie Villager cure is the
conversion producer: preserved equipment is moved through slots `300+`,
then finalized flag/data, gossip, copied offers and XP are installed;
Conversion finalization and a Brain refresh follow. An online initiating
ServerPlayer then receives the cure criterion and cure reputation event.
The result gets Nausea `200`, and a nonsilent conversion emits level event
`1027`.

Spawn Egg item raw ID `1200` is a common 64-stack with
`entity_data.id=minecraft:villager`; commands, spawners and custom factories
retain generic production. Raw/null-group finalization creates
`AgeableMobGroupData(false)`, finalizes biome type only when the Boolean is
false, and produces no random group baby.

**Lightning conversion:**

On non-Peaceful difficulty lightning converts to Witch using the ordinary
single-entity conversion with keep-equipment and preserve-pickup both false.
That conversion still transfers the first passenger, vehicle, leash and
common Mob state. A created Witch is finalized with Conversion reason, made
persistent, and only then the Villager releases all four POIs. A null
conversion delegates to generic lightning. Peaceful always delegates and
does not convert. Conversion does not run Villager death/witness reputation.

**Loot, XP, tags and progression:**

`entities/villager` contains no pools, so ordinary death yields no table
items. Generic equipment drops remain separate and the subtype XP reward is
zero.

Three direct entity-type tags contain Villager:
`candidate_for_iron_golem_gift`, `cannot_be_age_locked` and
`followable_friendly_mobs`. Trade completion feeds the generic
`villager_trade` trigger used by `adventure/trade` and the Y-at-least-`319`
`adventure/trade_at_world_height`. Cure feeds
`story/cure_zombie_villager`; lightning predicates select Villager in
`adventure/lightning_rod_with_villager_no_fire` and
`adventure/very_very_frightening`.

**Compatibility:**

Locked migration includes legacy Villager ID registration/renaming;
profession/career conversion into plains `VillagerData`; rebuilding an
absent/level-`0..1` level from two offers per level and absent XP from old
thresholds `0/10/50/100/150`; old follow-range `16` replacement with `48`;
default CanPickUpLoot true; Pumpkin trade-stack correction; empty `buyB`
removal; entity/Brain UUID, Spawn Egg and statistics migrations.

Current loading does not rerun those migrations. It first loads generic
offers/inventory, then:

| Saved field | Decode/reconstruction |
|---|---|
| `VillagerData` | optional codec; its presence alone sets finalized true |
| `VillagerDataFinalized` | true also finalizes, using default data if data is absent |
| `FoodLevel` | signed byte widened to int; saving truncates the int to byte |
| `Gossips` | list codec replaces the fresh container; loaded values are not proactively clamped |
| `Xp` | signed int, default `0` |
| `LastRestock` | signed long, default `0` |
| `LastGossipDecay` | signed long, default `0` |
| `RestocksToday` | signed int, default `0` |
| `AssignProfessionWhenSpawned` | written only when true, default false |
| `Offers` | server-written only when the cache is nonnull; absent reconstructs lazy generation |
| inventory | eight generic container slots |

The Brain refresh occurs after the first seven subtype reads but before
Restocks Today and Assign Profession are read. Loaded VillagerData is written
directly rather than through the profession-changing setter, preserving
loaded offers.

**Client projection:**

Raw sound IDs `1695..1714` are Ambient, Celebrate, Death, Hurt, No, Trade,
Yes and the 13 working-profession sounds in registry order. Sleeping has no
ambient sound; trading selects Trade; otherwise Ambient. Hurt and Death are
fixed, and work selects the nullable profession work sound.

Renderer shadow radius is `0.5`, halved for babies. Adult and baby base
textures are separate `64x64` images. A second layer selects seven adult
type or seven baby-type textures, then adults except none add one of 14
profession textures. Nitwit omits the level layer; every other shown
profession maps clamped levels `1..5` to Stone, Iron, Gold, Emerald and
Diamond textures.

Texture metadata marks Desert/Snow type hats and Farmer/Fisherman/Fletcher/
Librarian/Shepherd profession hats full, Butcher partial, and all others
none. A full profession hat suppresses type-hat geometry; a partial
profession hat retains it unless the type hat is full. Invisible Villagers
skip all type/profession/level layers. Custom head and crossed-arms item
layers follow; the custom-head transform is
`(-0.1171875,-0.07421875,1)`.

The adult model is the fixed crossed-arm Villager mesh. Baby uses its
separate mesh. Positive unhappy state fixes head X rotation `0.4` and sets
Z rotation to `0.3*sin(0.45*ageInTicks)`; otherwise Z is zero. Legs use the
ordinary opposite cosine walk. Spawn Egg uses a direct generated `16x16`
texture. Merchant menu/offers, raw packet IDs and GUI sprites retain
`ITM-CONTAINER-001`, `CLI-UI-001` and the protocol C3 owner.

**Constants and randomness:**

Exact private RNG streams include activity behaviors; POI scan/retry/path
selection; Work one-in-two; raid one-in-100; trade-set random sequences;
trade orb `3..6`; Hero gift `600..6600`; breeding type/duration/hearts;
gossip weighted selection; Golem placement; particles; structure
finalization; and lightning/conversion owners. Do not merge level, entity,
loot-context, named-sequence, template or client RNGs.

**Side effects:**

Brain memories and navigation; POI reservations/releases; profession/type/
level and offers; menu binding and packets; inventory/equipment/item
entities; offer uses/demand/prices/XP; effects, statistics and criteria;
food/age/child insertion; gossip/reputation; Iron Golem or Witch creation;
damage/death; sounds, particles, events, debug POI synchronization and
client model layers.

**Gates:**

Logical side; alive/baby/sleep/trading/NoAI state; activity and memory
requirements; reloadable schedule/clock/raid; visibility, distance, path and
POI capacity/type; profession/level/XP; inventory capacity/contents; offer
and payment state; restock time/day/demand; food/age/bed; gossip cooldown;
sleep/Golem memory/quorum; difficulty and factory/insertion outcomes.

**Branches and aborts:**

Every metadata/NBT presence/type/value; all type/profession/level holders;
adult/baby schedule and activity requirements; every package arbitration;
POI absent/full/unreachable/wrong-dimension/changed-type paths; assignment,
yield, conflict and reset; offer cache/set/record/null-output paths; all
interaction hands/states; price, trade, timer and restock boundaries; every
food slot/count/share/breed/bed/insertion branch; gossip entry/cooldown/
transfer/decay/witness path; Golem vote/placement; all producers,
conversion, loot/tags/criteria/migrations/sounds and render layers.

**Boundary cases and quirks:**

Level is minimum-clamped but not maximum-clamped. Food is byte-persisted.
The Structure assignment bypass lasts only the first Brain tick. Equal-XP
POI competition is encounter-order sensitive. POI release validates current
live type and does not itself erase the memory.

An off-hand no-offer interaction differs from main hand. The level timer
pauses while trading. Level-up appends rather than replaces offers.
Restock's half-day and minimum-delay comparisons are strict; the previous-day
cursor is transient. Work sound/workstation effects happen before restock.

Breeding consumes food before bed acquisition and never refunds it. Child
insertion is ignored before Heart/Home publication. Item sharing reconstructs
an identity-only stack. Gossip can transfer fewer than ten entries and never
catches up missed daily decays. Golem success marks nonvoters too.

**Failure semantics:**

Missing trade sets produce an empty/no-growth offer list after a debug log.
Null trade records consume candidate ownership as specified. XP-orb, child
and several item/entity offers do not roll back earlier state when insertion
fails. No-bed breeding retains consumed food. POI release failures retain
the memory unless the calling behavior explicitly erases it. Failed Golem
or Witch creation retains preexisting state except where the cited conversion
owner says otherwise. Empty death loot is a successful zero-result table.

**Client/server authority split:**

The server owns Brain execution, navigation, POIs, profession/type
finalization, offers/prices/trades/restocking, inventory/food, breeding,
gossip/reputation, Golem/Witch production, persistence, loot and
progression. Clients consume metadata, equipment, events, sounds, menu
snapshots and resources and independently select render layers/animation.
Client Success prediction cannot create offers or complete a trade.

**Observability:**

Observe registration/attributes/dimensions and slots `16..20`; every data,
field and component reconstruction; complete sensor/activity/package state;
schedule and age refresh; POI reservations/memories/profession/offer cache;
display/gift/work/crop/item effects; interaction/menu/price/offer/trade/
level/restock order; food/share/breeding transaction; gossip values,
reputation/witnessing/Golem vote; every spawn/conversion/loot/tag/criterion/
migration path; all sounds/events/particles/textures/hats/models.

**Persistence and reload:**

Generic entity/Living/Mob/Ageable/Brain/equipment state, eight-slot
inventory, optional offers and the subtype fields listed above persist.
Trading player, unhappy counter, last gossip time, last restock-check day,
merchant timer/level flag/last trader, active behaviors, navigation and
behavior-local cooldowns do not. Schedule, professions, trade sets/trades,
tags, loot, advancements, POIs, biome attributes and timelines reload
server-side. Registries used by synchronized VillagerData must remain
connection-consistent. Language, sounds, models, textures and metadata
reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.npc.villager.AbstractVillager`;
`net.minecraft.world.entity.npc.villager.Villager`;
`net.minecraft.world.entity.npc.villager.VillagerData`;
`net.minecraft.world.entity.npc.villager.VillagerDataHolder`;
`net.minecraft.world.entity.npc.villager.VillagerType`;
`net.minecraft.world.entity.npc.villager.VillagerProfession`;
`net.minecraft.world.entity.ai.behavior.VillagerGoalPackages`;
`net.minecraft.world.entity.ai.behavior.AcquirePoi`;
`net.minecraft.world.entity.ai.behavior.AssignProfessionFromJobSite`;
`net.minecraft.world.entity.ai.behavior.ResetProfession`;
`net.minecraft.world.entity.ai.behavior.YieldJobSite`;
`net.minecraft.world.entity.ai.behavior.PoiCompetitorScan`;
`net.minecraft.world.entity.ai.behavior.SetWalkTargetFromBlockMemory`;
`net.minecraft.world.entity.ai.behavior.WorkAtPoi`;
`net.minecraft.world.entity.ai.behavior.WorkAtComposter`;
`net.minecraft.world.entity.ai.behavior.ShowTradesToPlayer`;
`net.minecraft.world.entity.ai.behavior.GiveGiftToHero`;
`net.minecraft.world.entity.ai.behavior.TradeWithVillager`;
`net.minecraft.world.entity.ai.behavior.VillagerMakeLove`;
`net.minecraft.world.entity.ai.behavior.VillagerCalmDown`;
`net.minecraft.world.entity.ai.sensing.VillagerHostilesSensor`;
`net.minecraft.world.entity.ai.sensing.GolemSensor`;
`net.minecraft.world.entity.ai.gossip.GossipContainer`;
`net.minecraft.world.entity.ai.gossip.GossipType`;
`net.minecraft.world.entity.monster.zombie.ZombieVillager`;
`net.minecraft.util.datafix.fixes.VillagerDataFix`;
`net.minecraft.util.datafix.fixes.VillagerFollowRangeFix`;
`net.minecraft.util.datafix.fixes.VillagerRebuildLevelAndXpFix`;
`net.minecraft.util.datafix.fixes.VillagerSetCanPickUpLootFix`;
`net.minecraft.util.datafix.fixes.VillagerTradeFix`;
`net.minecraft.util.datafix.fixes.EmptyItemInVillagerTradeFix`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.VillagerRenderer`;
`net.minecraft.client.renderer.entity.layers.VillagerProfessionLayer`;
`net.minecraft.client.renderer.entity.state.VillagerRenderState`;
`net.minecraft.client.model.npc.VillagerModel`;
`net.minecraft.client.model.npc.BabyVillagerModel`;
`reports/registries.json#minecraft:{entity_type,item,data_component_type,
villager_type,villager_profession,point_of_interest_type,menu,sound_event,
particle_type,loot_table,trade_set,villager_trade,worldgen/biome,
advancement,environment_attribute,timeline}`;
`reports/minecraft/components/item/villager_spawn_egg.json`;
`data/minecraft/timeline/villager_schedule.json`;
`data/minecraft/tags/{entity_type/{candidate_for_iron_golem_gift,
cannot_be_age_locked,followable_friendly_mobs},item/{villager_picks_up,
villager_plantable_seeds},point_of_interest_type/acquirable_job_site}.json`;
`data/minecraft/{trade_set,villager_trade,tags/villager_trade}/**/*.json`;
`data/minecraft/loot_table/{entities/villager,
gameplay/hero_of_the_village/*}.json`;
`data/minecraft/advancement/{adventure/{trade,trade_at_world_height,
lightning_rod_with_villager_no_fire,very_very_frightening},
story/cure_zombie_villager}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/villager_spawn_egg.*`;
`assets/minecraft/textures/entity/villager/**/*.png`;
`assets/minecraft/textures/gui/{container/villager.png,
sprites/container/villager/*.png}`;
`assets/minecraft/sounds.json`; `assets/minecraft/lang/en_us.json`;
`ENT-IRON-GOLEM-001`; `MOB-BREED-001`; `MOB-RAID-001`;
`WGEN-DIMENSION-001`; `WGEN-JIGSAW-VILLAGES-001`;
`WGEN-STRUCTURE-IGLOO-001`; `ITM-CONTAINER-001`; `CLI-UI-001`.

**Test vectors:**

Run `EXP-ENT-039` across fresh/finalized/Structure/Igloo/cured/bred/loaded
Villagers; all age/type/profession/level/metadata/NBT/component states; every
sensor, activity, package and schedule boundary; all POI acquisition,
assignment, conflict, reset and release paths; work/display/gift/share;
offer generation, interaction, price, trade, level and restock transactions;
food/breeding; gossip/reputation/Golem; lightning conversion; all producers,
loot/tags/progression/migrations and exact merchant/client projection.

**Limits:**

Generic lifecycle, Brain arbitration, navigation/pathfinding, sleep/raid/
bell/crop behavior, damage/death, Mob pickup, Ageable clocks, POI manager,
merchant offer/payment/menu protocol, inventory insertion, loot,
advancements, conversion, Iron-Golem placement and renderer submission
retain their cited owners. This leaf owns Villager selectors, overrides,
constants, data joins and their exact composition.
