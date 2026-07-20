# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `MOB-BREED-001` — Love, mate selection, child creation, cooldown, and ownership inheritance commit together

**Parent:** `MOB-006`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Cross-checked`

**SourceConclusion:**

`SourceInconclusive` — species eligibility, inheritance, taming chances, ownership, and special side
effects remain unexpanded.

**Applies when:**

A breedable/tameable mob is fed, enters love, finds a compatible mate, or completes breeding.

**Authoritative state:**

Age/cooldown, love timer and cause player, mate compatibility, species variant/genetics, tame/owner
state, child entity and gamerules.

**Transition and ordering:**

Item interaction validates food/age/state and consumes item under ability rules; set love state; AI
selects compatible mate and approaches; on completion create species child, apply
parent/variant/owner rules, set both parent cooldown ages and clear love, add child, award
player/stat/criterion/XP according to branch. Taming uses its own item/RNG attempt and owner
assignment but shares interaction authority.

**Branches and aborts:**

Baby/cooldown; not food; not compatible/same invalid entity; path/partner lost; spawn child
null/rejected; `mobGriefing` for food pickup rather than direct feeding; tame roll fails/succeeds;
already tamed owner interaction.

**Constants and randomness:**

Love and age durations, XP range, tame chance and variant inheritance are species/source constants.
RNG is consumed at child-variant/tame/XP sites only after prerequisites. Exact per-family values are
`EXP-MOB-004`.

**Side effects:**

Item decrement, hearts/smoke, love/cooldown/age/owner, child/XP entities, statistics/criteria,
sounds/game events and AI memories.

**Gates:**

Species food tag, age, health/state, mate compatibility, ownership, player abilities, entity-add
validity, gamerules for related environmental behavior and difficulty where species checks it.

**Boundary cases and quirks:**

Feeding into love and successful child creation are separate transitions; item can be consumed even
if no mate is found. Ownership inheritance is species-specific, not universal.

**Evidence:**

`Confirmed` generic lifecycle; species inheritance/chances require family rules; `OFF-SERVER-001`;
`Animal`/`TamableAnimal` families; `EXP-MOB-004`.

**Test vectors:**

Adult/baby/cooldown; compatible/incompatible variants; mate removed at completion; full entity
rejection; creative item; tame fail/success fixed RNG; owner versus non-owner interaction.
