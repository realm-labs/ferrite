# Item, inventory, and progression leaf rules

## Leaf rule `ITM-USE-001` — Item use separates start, per-tick use, release, and finish

**Parent:** `ITM-001`, `ITM-003`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — item-family durations, cadence, returned-stack branches, durability RNG, and cooldown values remain unexpanded.  <br>
**Applies when:** Interaction dispatch reaches an item's use behavior and it starts or performs an action.  
**Authoritative state:** Hand stack/components, active hand and remaining use ticks, cooldowns, player state, target context and returned interaction result.  
**Transition and ordering:** Invoke context/air use; if immediate, apply returned stack/result now; if consuming, record active hand and duration; each player tick invoke use-tick behavior at the item's cadence; on release invoke release behavior with elapsed/remaining duration; on natural completion invoke finish behavior and install its returned stack. Revalidate that the active stack is compatible before each stage. Anchors: `net.minecraft.world.item.Item#use(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.world.InteractionHand)` and `net.minecraft.world.entity.LivingEntity#completeUsingItem()`.  
**Branches and aborts:** Fail/pass/success; instant versus duration use; active stack replaced; player stops; duration reaches zero; cooldown/feature gate; creative exemption; item returns a container/replacement. Release and finish are mutually selected by how use ends.  
**Constants and randomness:** Duration and animation are item/component data. Effects, projectile divergence, food outcomes or durability may consume RNG only in their branch. Tick counters are integers; elapsed calculation must match the source off-by-one boundaries.  
**Side effects:** Stack count/components/replacement, cooldown, active pose, food/effects, projectile/entity spawn, durability, statistics/criteria/game events, sounds/particles and inventory synchronization.  
**Gates:** Interaction result, cooldown, hunger/always-edible, hand, feature flags, player abilities, target conditions and active-stack identity.  
**Boundary cases and quirks:** The stack returned by finish can differ in item type and must replace the correct hand slot. Interrupting on the last apparent client frame may still be release rather than finish depending on server tick receipt.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; locators above; tick boundary `EXP-ITM-001`.  
**Test vectors:** Immediate use, full-duration food, release bow at every boundary tick, replace held stack while using, creative container item, cooldown rejection and simultaneous inventory synchronization.

## Leaf rule `ITM-CONTAINER-001` — Menu clicks are validated transactions over server slots and carried stack

**Parent:** `ITM-002`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — click-type algorithms, quick-move loops, state-ID boundaries, and every menu layout remain unexpanded.  <br>
**Applies when:** A player has a menu open and requests a click/quick-move/drag/swap/throw/clone/pickup-all operation.  
**Authoritative state:** Menu/container ID and state ID, slot list and constraints, carried stack, player inventory, data slots, validity/distance predicate and synchronizer snapshots.  
**Transition and ordering:** Validate menu identity/state; decode the 26.2 `ContainerInput` operation and button/slot arguments; run the operation against current server stacks, honoring `mayPickup`, `mayPlace`, slot limits and item compatibility; mutate slots/carried stack in the operation-specific order; call slot/container change hooks; broadcast incremental changes or a full resync. Quick-move repeatedly invokes menu-specific transfer ranges. Anchor: `net.minecraft.world.inventory.AbstractContainerMenu#clicked(int,int,net.minecraft.world.inventory.ContainerInput,net.minecraft.world.entity.player.Player)`.  
**Branches and aborts:** Stale/wrong menu; invalid slot/button; nonmatching client changed-slot report; slot restriction; stack cannot merge; capacity reached; menu no longer valid; creative-only clone. A rejected prediction causes full authoritative synchronization and no duplicated items.  
**Constants and randomness:** Stack maxima are the minimum of item component, slot and container constraints. Counts are integers. Click algorithms consume no RNG. Slot ranges/order are menu-family behavior and are mapped by `menu` registry ID.  
**Side effects:** Slot/carried/player inventory changes, crafting result consumption, recipe remainder placement/drop, container callbacks, achievements/statistics, sounds for special menus and network resynchronization.  
**Gates:** Current menu/container/state ID, validity/distance, player mode, slot policies, item components, click type/button and recipe/output rules.  
**Boundary cases and quirks:** Client changed-slot maps are validation evidence, not authoritative mutations. Quick-move order affects destination stacks. Closing returns/drops a carried stack through menu removal semantics.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`; `AbstractContainerMenu#clicked`; client prediction in `CLI-UI-001`; transaction matrix `EXP-ITM-002`.  
**Test vectors:** Every click type with empty/partial/full stacks; restricted result/fuel/armor slot; stale state ID; quick-move across partial matches; close while carrying; two viewers mutate one container in the same tick.

## Leaf rule `ITM-CRAFT-001` — Recipe matching, result assembly, ingredient consumption, and remainder placement are one ordered operation

**Parent:** `ITM-003`, `ITM-004`, `ITM-005`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — serializer-specific matching and each processor's timers, fuel, remainders, and hopper ordering remain unexpanded.  <br>
**Applies when:** A crafting grid or processing device recomputes a recipe or takes/finishes an output.  
**Authoritative state:** Input slots, recipe manager/serializer, selected recipe, processing timers/fuel, output slot, recipe-used bookkeeping and player inventory.  
**Transition and ordering:** Build the recipe input view; select a matching enabled recipe of the device's recipe type; assemble/display output without consuming inputs; when output is taken or processing completes, revalidate match, consume exact ingredients, compute per-slot remainders, place compatible remainders back or inventory/drop them, award recipe/stat/XP effects and recompute output. Processing devices advance lit/cook/brew timers before committing their completion branch according to their block entity tick.  
**Branches and aborts:** No match; feature/recipe-book restriction; output cannot accept result; fuel absent; input changes mid-process; remainder cannot merge; player shift-crafts repeated outputs; special recipe serializer has custom match/assembly.  
**Constants and randomness:** Recipe data and serializer define dimensions/ingredients/result. Cook/brew/fuel times and XP are locked data/source values; fractional XP rounding and RNG belong to the device rule and `EXP-ITM-003`. Generic matching is deterministic.  
**Side effects:** Input/output/remainder slots, fuel/container items, timers, recipe-used list, experience/statistics/criteria, sound/particles/block state and inventory sync.  
**Gates:** Recipe type/serializer, enabled features, gamerules only where a particular recipe checks them, slot capacity, device validity, fuel and player access.  
**Boundary cases and quirks:** Merely displaying an output never consumes. Remainders are per consumed slot, not a single global result. Shift-crafting repeats only while the complete transaction remains valid.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`, `OFF-DATA-001`; serializer and recipe IDs via `mc-ref query`; processing boundary experiments `EXP-ITM-003`.  
**Test vectors:** Shaped/mirrored, shapeless duplicate ingredient, special serializer, remainder into occupied grid, full inventory remainder drop, fuel ends on completion tick, input swap during progress, shift-craft until one constraint fails.

## Leaf rule `ITM-LOOT-001` — Loot is generated from a context and consumed exactly once by its caller

**Parent:** `ITM-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — caller context construction, pool/function ordering, RNG consumption, and every special loot hook remain unexpanded.  <br>
**Applies when:** A block/entity/container/gameplay event invokes a loot table.  
**Authoritative state:** Selected table ID, loot context parameter set, luck, tool/components/enchantments, killer/source/origin, table RNG/seed policy and destination consumer.  
**Transition and ordering:** Construct the table-specific context; validate required parameters; evaluate pools in data order, rolls/conditions/functions and nested entries with the supplied random source; normalize/split generated stacks as the API requires; pass each stack once to the caller, which spawns, inserts, or stores it. Data at `data/minecraft/loot_table/**/*.json` is normative and queryable.  
**Branches and aborts:** Missing/invalid required context; conditions fail; zero rolls; empty entry; explosion decay; player/tool/enchantment predicate; nested table; destination full. Empty generation is successful evaluation, not an error.  
**Constants and randomness:** Every number provider and weighted selection consumes the supplied RNG at its evaluation site. Integer/floating providers and stack splitting retain their native rounding. A table with an explicit seed uses that path; otherwise caller RNG determines results.  
**Side effects:** Generated stacks, item entities/container contents, XP through separate functions/callers, criteria/statistics and later merge/pickup behavior. Loot evaluation itself must not silently insert into a player inventory.  
**Gates:** Caller event, gamerules such as block/entity drops, context predicates, data-pack table, difficulty/biome/dimension predicates and destination capacity.  
**Boundary cases and quirks:** Re-evaluating after a partial consumer failure duplicates loot; generate once then consume the produced sequence. Explosion decay and surviving explosion are context-driven.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`, `OFF-DATA-001`; exact data through catalog/query; RNG sequence `EXP-ITM-004`.  
**Test vectors:** Same seeded context twice; missing required parameter; fortune/silk/explosion contexts; nested table; oversized count splitting; full container consumer; ensure a caller retry does not regenerate.

## Leaf rule `ITM-ENCHANT-001` — Enchantment behavior is component/effect driven and applies at defined hook sites

**Parent:** `ITM-006`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Cross-checked`  <br>
**SourceConclusion:** `SourceInconclusive` — effect hook ordering, compatibility, slot iteration, and random-effect consumption remain unexpanded.  <br>
**Applies when:** An item stack carries enchantments and gameplay reaches a matching enchantment effect hook.  
**Authoritative state:** Stack enchantment component/levels, registry definitions and tags, entity/equipment context, damage/mining/projectile/loot context and RNG.  
**Transition and ordering:** Read active enchantments from the participating stacks; filter definitions/effects for the current hook and requirements; evaluate level-based values in the hook's defined equipment iteration order; combine modifiers using the effect's operation; apply post-effects such as durability, entity effects or sounds at that hook. Enchanting-table offer generation is a separate random selection transaction.  
**Branches and aborts:** Wrong slot/context; requirements false; incompatible/disabled definition; level absent; victim/attacker/direct entity mismatch; value operation yields no change; creative/infinite material exception.  
**Constants and randomness:** Definitions under `data/minecraft/enchantment` are DataOnly inputs. Level-based values specify exact arithmetic and clamping. RNG is consumed by random value effects, durability checks and offer selection only when their branch evaluates.  
**Side effects:** Modified damage/protection/mining/loot/projectile values, durability, status/entity effects, item transformations, sounds/particles, criteria and XP/lapis/offer seed for enchanting UI.  
**Gates:** Equipment slot/group, effect requirements, tags, levels, damage type/context, feature flags, player mode/resources and hook invocation.  
**Boundary cases and quirks:** Do not hard-code enchantments as one enum switch: 26.2 definitions compose typed effects. Multiple equipped stacks may participate, and order/RNG consumption can be observable.  
**Evidence:** `Confirmed` data-driven architecture; combination order for multi-slot random effects `Cross-checked`; `OFF-SERVER-001`, `OFF-DATA-001`; `EXP-ITM-005`.  
**Test vectors:** One effect at min/max level, unmet predicate, two equipment pieces, incompatible table offers, durability random branch, damage-type-specific protection and data-reload stability.

## Leaf rule `ITM-ADVANCEMENT-001` — Advancement criteria are event listeners with requirement-matrix completion

**Parent:** `ITM-007`  
**FidelityClass:** `ExactObservableBehavior`  <br>
**EvidenceStatus:** `Confirmed`  <br>
**SourceConclusion:** `SourceInconclusive` — hunger and XP have no leaf specification yet; advancement trigger ordering and listener mutation branches remain unexpanded.  <br>
**Applies when:** A player-relevant trigger fires or a command revokes/grants progress.  
**Authoritative state:** Advancement definition, criterion progress timestamps, requirement matrix, per-player listener registration, rewards and visibility progress.  
**Transition and ordering:** Register listeners for incomplete criteria; on trigger evaluate player/context predicate; grant the criterion once and unregister it; recompute completion by requiring each requirement group to contain a satisfied criterion; on first transition to done, apply rewards and dependent visibility/listener updates. Revoke clears requested criteria and restores listeners where incomplete.  
**Branches and aborts:** Predicate false; criterion already done; definition disabled/missing; partial matrix not complete; reward function/recipe/loot absent; command mode selects only/subtree/ancestors/everything.  
**Constants and randomness:** Requirement structure and rewards are locked advancement JSON. Criterion timestamp uses wall-clock instant for display/storage but gameplay completion order is the server event order. Loot reward consumes its supplied RNG at reward time.  
**Side effects:** Progress, toast/chat visibility, recipes, loot, XP, reward function and network progress updates. Rewards run once per transition to completed, and can run again only after revocation permits a new transition.  
**Gates:** Trigger listener, per-player predicate, requirement matrix, feature/data pack, command permissions for manual mutation and reward validity.  
**Boundary cases and quirks:** Requirements are AND across groups and OR within a group. Granting an already-complete criterion is idempotent. A definition may be complete without every named criterion.  
**Evidence:** `Confirmed`; `OFF-SERVER-001`, `OFF-DATA-001`; catalog snapshot; listener/revoke trace `EXP-ITM-006`.  
**Test vectors:** Two-by-two requirement matrix; repeated trigger; revoke one member of an OR group versus the only satisfied group; reward function changes another criterion; reload definition while players are online.
