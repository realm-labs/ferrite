# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-BREED-001` — Feeding, mate approach, offspring finalization and tame/trust state are separate commits

**Parent:** `MOB-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — age/love clocks, item consumption, mate search, child/failure ordering, XP and
criteria, generic tame ownership and every breed/tame/trust family override are fixed by locked
source, tags and variant registries.

**Applies when:**

An `AgeableMob` ages or is fed/age-locked, an `Animal` enters love or runs its breeding goal, or a
species interaction attempts tame/trust. Brain-based egg/offspring producers use the content-family
dispatch below but preserve the same authoritative entity/addition rules.

**Authoritative state:**

Signed age, forced age, age-lock state, love timer/cause, species food tags and compatibility,
nearby partner list, navigation and panic state, child factory/variant data, parent tame/owner state,
player abilities, gamerules and mob/level RNG.

**Transition and ordering:**

**Age and feeding:**

Default baby age is `-24000`. On a living server age tick, an unlocked baby increments age by one;
a positive adult cooldown decrements by one; zero stays zero. Crossing zero updates baby synced data
and the species boundary hook. `ageUp(seconds, forced)` adds `seconds * 20`, caps a positive result
to zero, applies the delta, and for forced growth accumulates the delta in `forcedAge` and starts a
`40`-tick client particle timer. Reaching zero then sets age to accumulated `forcedAge`, which can
create the post-growth positive cooldown. Feeding growth uses
`floor((ticksUntilAdult / 20) * 0.1)` seconds. A golden dandelion used on a baby at cooldown zero
and outside `CANNOT_BE_AGE_LOCKED` toggles age lock, resets age to the species baby start, consumes
one, plays lock/unlock sound, starts a `40`-tick lock-particle timer and sets persistence required
only on the transition to locked.

`Animal#mobInteract` asks the subtype `isFood`. For a server player and age zero with no current
love, it consumes through `usePlayerItem`, sets love to `600`, records the server player as cause,
emits entity event `18`, plays the subtype eating sound and returns server success. Otherwise an
unlocked baby consumes, applies forced growth and succeeds. A client-side food use that does neither
returns consume; all remaining branches delegate. Nonzero age clears love during server custom AI
and ordinary AI; damage also clears it. Love decrements each ordinary AI step and emits a heart
every ten remaining ticks.

**Mate selection and approach:**

Base compatibility rejects self or a different runtime class and otherwise requires both love
timers positive. Subtypes may strengthen or replace it. `BreedGoal` owns `MOVE|LOOK`, starts only
when the actor is in love, and searches noncombat targets of its configured partner class inside an
8-block inflated box/range while ignoring line of sight. It chooses the strictly nearest candidate
that passes `canMate` and is not panicking; equal distance retains earlier query order. Continue
requires a live, in-love, non-panicking partner and `loveTime < 60`.

Each goal tick looks at the partner, requests navigation and increments `loveTime`. Breeding occurs
when `loveTime >= adjustedTickDelay(60)` and squared distance is strictly below `9`. Because the
base goal is not every-tick, its delay is `30` and it normally ticks on full selector phases; stop
clears partner and time. Navigation failure alone is not a continue predicate.

**Generic child commit and failure semantics:**

`spawnChildFromBreeding` first calls the actor's `getBreedOffspring(level, partner)`. Null performs
no child, cooldown, love clear, criterion or XP work, so the still-running goal can call it again.
A nonnull child is set baby, snapped to the actor with zero rotation, then the parent finalization
runs before insertion. It chooses the first nonnull love cause (actor, then partner), awards
`animals_bred` and triggers `bred_animals` with the child; sets each parent age to `6000`; clears
both love timers; broadcasts event `18` on the actor; and, when `mob_drops` is true, inserts XP of
`nextInt(7)+1`. Only after all of those effects does the caller invoke
`addFreshEntityWithPassengers(child)`. Addition failure has no rollback and XP is inserted before
the child.

**Breed content-family catalog:**

Species `isFood`, `canMate`, `getBreedOffspring` and special breeding-goal/Brain methods are the
closed dispatch. A factory method is not admission: the direct `Animal` food selectors are the
locked `HOGLIN_FOOD`, `STRIDER_FOOD`, `ARMADILLO_FOOD`, `AXOLOTL_FOOD`, `BEE_FOOD`, `CAMEL_FOOD`,
`CHICKEN_FOOD`, `COW_FOOD`, `HORSE_FOOD`, `LLAMA_FOOD`, `CAT_FOOD`, `OCELOT_FOOD`, `FOX_FOOD`,
`FROG_FOOD`, `GOAT_FOOD`, `PANDA_FOOD`, `PIG_FOOD`, `RABBIT_FOOD`, `SHEEP_FOOD`, `SNIFFER_FOOD`,
`TURTLE_FOOD` and `WOLF_FOOD` tags. Adult untamed nautilus instead admits its taming tag and a
tamed/adolescent one its food tag. Parrot and polar bear return false from `isFood`; happy ghast,
camel husk and zombie horse reject love; mule rejects mating; and sulfur cube is only a custom
`AgeableMob` feeding/growth path. An inherited or callable child factory does not override those
gates. After admission, the observable inheritance families are:

- armadillo, bee, camel, ocelot and strider create a same-type child without inherited variant
  state. Pig, chicken and cow choose one parent's variant uniformly. Mooshroom chooses a parent
  variant unless equal parents take their `1/1024` mutation to the other variant. Rabbit begins
  with the biome-selected variant, then with probability `19/20` replaces it with the actor or,
  only when the partner is a rabbit and a boolean succeeds, the partner. Axolotl takes the rare
  registry variant with probability `1/1200`, otherwise a random parent's variant, and marks the
  child persistent. Goat initializes child Brain memories, chooses a parent uniformly, and makes
  the child screaming when that parent screams or otherwise with probability `0.02`. Hoglin marks
  its child persistent and additionally requires both parents not pacified;
- cat and wolf require tame partners. Both randomly select a parent variant; when the initiating
  parent is tame the child inherits that parent's owner, becomes tame and receives the parents'
  mixed collar color. Wolf additionally chooses a random sound variant. Fox selects a parent
  variant; its specialized goal clears both parents' active states at start and does not call
  generic finalization: it creates the child, records each distinct available love-cause player in
  the baby's two trusted references, awards/triggers using actor cause then partner fallback, sets
  both ages to `6000`, clears love, sets child age `-24000`, snaps and inserts the child, then
  broadcasts event `18` and finally inserts `1..7` XP when `mob_drops` is true. Trust is not
  ownership;
- sheep child color is the dye-recipe mix of both parent colors when that recipe yields a dye,
  otherwise a random parent color. Panda independently derives main and hidden genes from the two
  parents and gives each a `1/32` random mutation opportunity. Horse accepts a parentable horse or
  donkey: donkey crossing creates a mule; horse crossing selects coat `4/9` actor, `4/9` partner,
  `1/9` random and markings `2/5`, `2/5`, `1/5`. Donkey has the symmetric donkey/mule dispatch;
  llama child strength is uniform `1..max(parent strengths)` with a `3%` one-point increase capped
  at `5`, then takes a random parent variant. Horse health, jump and movement bases each use the
  average plus a three-uniform triangular term over parent difference plus a 15% range margin,
  reflected back inside the allowed attribute interval. Equine parentability additionally requires
  tame, adult, full health, in love, neither vehicle nor passenger;
- turtle breeding does not create an immediate child: the specialized goal attributes the player,
  sets the initiating turtle's egg flag, applies both `6000` cooldowns/love clears and optional
  `1..7` XP; its home-near sand lay goal later waits past adjusted delay `200`, writes `1..4` turtle
  eggs, clears the egg/digging state and sets love time `600`. Frog generic-finalizes with a null
  child and then sets `IS_PREGNANT`; its Brain later lays frogspawn. Sniffer constructs an egg item
  with default pickup delay, generic-finalizes with null, consumes two pitch RNG floats for the plop
  sound, then inserts the item; it only mates while both states are `IDLING`, `SCENTING` or
  `FEELING_HAPPY`;
- villager and hoglin breeding are Brain behaviors with memory/activity, willingness/food or
  population/space predicates and their own child transactions rather than `BreedGoal` timing.
  Allay duplication is also separate: while dancing, a `DUPLICATES_ALLAYS` item and zero cooldown
  create a snapped persistent allay, reset both cooldowns to `6000`, insert it, then the interaction
  broadcasts event `18`, plays the chime and consumes one item. A failed allay factory leaves the
  cooldowns unchanged but the outer interaction still emits/consumes.

**Tame, owner and trust commits:**

`TamableAnimal` stores tame in flag bit `4`, sitting pose in bit `1`, an optional living-entity
reference as owner, and a separate persisted `orderedToSit`. `tame(player)` sets the tame bit with
subtype side effects, assigns owner, and triggers `tame_animal` for a server player. Entity events
`7` and `6` render seven heart or smoke particles but do not themselves mutate authority. Tame
alliance delegates through the root owner; a tame animal refuses to attack its owner. Base owner
teleport is attempted at squared distance `>= 144` unless sitting, passenger, leashable/leashed as
tested, or owner spectator. It samples ten offsets: each horizontal delta is inclusive `[-3,3]`,
at least one absolute horizontal delta must be `>=2`, and vertical delta is `[-1,1]`; success
requires WALKABLE, no leaves for nonfliers and no collision, snaps to block center and stops path.

Species tame/trust admission remains distinct:

- untamed non-angry wolf consumes a bone and succeeds on `nextInt(3)==0`; success tames, clears
  navigation/target, orders sit and emits `7`, failure emits `6`. Cat and adult untamed nautilus
  consume their tags and use the same one-in-three success; cat orders sit, while nautilus stops
  navigation and accepts bucket food with a water-bucket remainder. Parrot consumes `PARROT_FOOD`
  and succeeds one-in-ten, and is never breedable;
- ocelot requires its tempt goal absent/running, untrusted state, tagged food and squared player
  distance `<9`; it consumes and becomes trusting on one-in-three, emitting trust-specific events
  `41/40`. No owner is assigned. Fox trust is inherited from love causes/egg spawner as above and
  likewise has no tame bit;
- horse-family taming is rider/temper based. While untamed, uncontrolled and ridden, the run-crazy
  goal checks on `nextInt(adjustedTickDelay(50))==0`; a player succeeds when
  `maxTemper > 0 && nextInt(maxTemper) < temper`, otherwise temper increases by `5`, all passengers
  are ejected, the horse becomes mad and event `6` is sent. Feeding effects on temper/health/growth
  are the locked `AbstractHorse` table;
- subtype owner interactions—healing, collar dye/mix, armor, inventories, sitting, riding and
  breeding eligibility—run in their declared `mobInteract` precedence. Tame does not generically
  set `persistenceRequired`; nautilus has a custom tame-persistence override, while cat/wolf rely on
  their own removal policy or other stored state.

**Branches and aborts:**

Nonfood, wrong age, locked baby, love already active, incompatible/panicking/dead partner, range
equality, null child, insertion failure, `mob_drops` false, wrong tame item/state/range, tame roll
failure, missing owner or invalid teleport candidate. Item consumption happens at feeding/tame
attempt, not when a mate/owner-follow path later succeeds.

**Constants and randomness:**

Base baby/love/parent ages are `-24000`, `600` and `6000`; both age-particle timers are `40`;
partner range is `8`, completion squared distance is strict `<9`, and adjusted breed delay is `30`
goal ticks. XP is uniform `1..7`. Wolf/cat/nautilus and ocelot attempts are one-in-three, parrot is
one-in-ten, horse checks adjusted `50` and its temper inequality, and owner teleport uses squared
`144`, ten attempts and the stated inclusive offsets. Variant/gene/egg subtype RNG follows the
locked family methods after their prerequisites.

**Side effects:**

Item/remainder mutation, age/love/forced age, sounds/particles/entity events, navigation/look,
trusted/owner/tame/sit state, parent cooldowns, child/egg/XP entities or blocks, stats/criteria,
variant/attribute inheritance and persistence/removal consequences.

**Gates:**

Living age tick, species baby/lock capability and food tags, server authority/player ability, love
and compatibility, partner life/panic/range, child factory, `mob_drops`, subtype tame/trust state and
item, owner identity, navigation/teleport safety and all family-specific Brain/memory predicates.

**Boundary cases and quirks:**

Love feeding is committed even if no mate ever appears. Null child leaves both parents in love,
while generic nonnull finalization commits parents/criterion/XP before child insertion and does not
roll back. Fox uses a distinct insertion-before-event/XP order. Age zero is adult eligibility;
positive age is a breeding cooldown. Trust and tame ownership are different persistent models, and
taming does not generically imply the base persistence flag.

**Evidence:**

`OFF-SERVER-001`; `OFF-DATA-001`; `net.minecraft.world.entity.AgeableMob`;
`net.minecraft.world.entity.animal.Animal`; `net.minecraft.world.entity.ai.goal.BreedGoal`;
`net.minecraft.world.entity.TamableAnimal`; `RunAroundLikeCrazyGoal`; every locked subtype
`isFood`, `canMate`, `getBreedOffspring`, `mobInteract` and specialized breed/egg/duplication method,
including cat/wolf/fox/ocelot/parrot/nautilus/equine/panda/sheep/turtle/frog/sniffer/villager/hoglin/
allay families; locked item tags and variant registries; `EXP-MOB-004`.

**Test vectors:**

Age `-24000/-1/0/1/6000`, forced growth and lock toggle; love `600/1/0`, damage and nonzero age;
partner tie/panic/death/range `9`; null and rejected child; criterion cause precedence and
`mob_drops`; every direct/special inheritance family with fixed RNG; cat/wolf ownership and collar
mix; fox trust; tame odds and consumed item; horse temper equality; owner teleport offsets/leaves/
collision; save/reload of age, love, tame, owner, trust and sitting state.
