# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ENCHANT-001` — Enchantment behavior is component/effect driven and applies at defined hook sites

**Parent:** `ITM-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — effect hook ordering, compatibility, slot iteration, and random-effect
consumption remain unexpanded.

**Applies when:**

An item stack carries enchantments and gameplay reaches a matching enchantment effect hook.

**Authoritative state:**

Stack enchantment component/levels, registry definitions and tags, entity/equipment context,
damage/mining/projectile/loot context and RNG.

**Transition and ordering:**

Read active enchantments from the participating stacks; filter definitions/effects for the current
hook and requirements; evaluate level-based values in the hook's defined equipment iteration order;
combine modifiers using the effect's operation; apply post-effects such as durability, entity
effects or sounds at that hook. Enchanting-table offer generation is a separate random selection
transaction.

**Branches and aborts:**

Wrong slot/context; requirements false; incompatible/disabled definition; level absent;
victim/attacker/direct entity mismatch; value operation yields no change; creative/infinite material
exception.

**Constants and randomness:**

Definitions under `data/minecraft/enchantment` are DataOnly inputs. Level-based values specify exact
arithmetic and clamping. RNG is consumed by random value effects, durability checks and offer
selection only when their branch evaluates.

**Side effects:**

Modified damage/protection/mining/loot/projectile values, durability, status/entity effects, item
transformations, sounds/particles, criteria and XP/lapis/offer seed for enchanting UI.

**Gates:**

Equipment slot/group, effect requirements, tags, levels, damage type/context, feature flags, player
mode/resources and hook invocation.

**Boundary cases and quirks:**

Do not hard-code enchantments as one enum switch: 26.2 definitions compose typed effects. Multiple
equipped stacks may participate, and order/RNG consumption can be observable.

**Evidence:**

`Confirmed` data-driven architecture; combination order for multi-slot random effects
`Cross-checked`; `OFF-SERVER-001`, `OFF-DATA-001`; `EXP-ITM-005`.

**Test vectors:**

One effect at min/max level, unmet predicate, two equipment pieces, incompatible table offers,
durability random branch, damage-type-specific protection and data-reload stability.
