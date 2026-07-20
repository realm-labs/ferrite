# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-ADVANCEMENT-001` — Advancement criteria are event listeners with requirement-matrix completion

**Parent:** `ITM-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceInconclusive` — hunger and XP have no leaf specification yet; advancement trigger ordering
and listener mutation branches remain unexpanded.

**Applies when:**

A player-relevant trigger fires or a command revokes/grants progress.

**Authoritative state:**

Advancement definition, criterion progress timestamps, requirement matrix, per-player listener
registration, rewards and visibility progress.

**Transition and ordering:**

Register listeners for incomplete criteria; on trigger evaluate player/context predicate; grant the
criterion once and unregister it; recompute completion by requiring each requirement group to
contain a satisfied criterion; on first transition to done, apply rewards and dependent
visibility/listener updates. Revoke clears requested criteria and restores listeners where
incomplete.

**Branches and aborts:**

Predicate false; criterion already done; definition disabled/missing; partial matrix not complete;
reward function/recipe/loot absent; command mode selects only/subtree/ancestors/everything.

**Constants and randomness:**

Requirement structure and rewards are locked advancement JSON. Criterion timestamp uses wall-clock
instant for display/storage but gameplay completion order is the server event order. Loot reward
consumes its supplied RNG at reward time.

**Side effects:**

Progress, toast/chat visibility, recipes, loot, XP, reward function and network progress updates.
Rewards run once per transition to completed, and can run again only after revocation permits a new
transition.

**Gates:**

Trigger listener, per-player predicate, requirement matrix, feature/data pack, command permissions
for manual mutation and reward validity.

**Boundary cases and quirks:**

Requirements are AND across groups and OR within a group. Granting an already-complete criterion is
idempotent. A definition may be complete without every named criterion.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`, `OFF-DATA-001`; catalog snapshot; listener/revoke trace
`EXP-ITM-006`.

**Test vectors:**

Two-by-two requirement matrix; repeated trigger; revoke one member of an OR group versus the only
satisfied group; reward function changes another criterion; reload definition while players are
online.
