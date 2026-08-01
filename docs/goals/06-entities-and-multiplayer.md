# Goal 06 — Entities and Multiplayer

## 1. Objective

Complete the formal server's authoritative entity and multiplayer gameplay path for Minecraft 26.2.
Players must observe one another and the same committed world; entities, combat, projectiles,
vehicles, drops, experience orbs, mobs, spawning, navigation, AI, tracking, transfer, persistence,
and client projection must operate through the composite Region runtime.

The result is a genuinely shared playable world rather than independent client sessions over common
terrain.

## 2. Production evidence contract

Every entity or multiplayer feature must prove:

```text
player/world/spawn input
  -> authoritative entity command and lifecycle mutation
  -> deterministic simulation and cross-system effects
  -> Region tracking or fenced transfer
  -> durable entity state where required
  -> bounded per-observer client projection
  -> two-client or exact-client MCP scenario
```

An entity model, isolated AI decision, lifecycle conformance test, or encoded clientbound packet
without observer-visible production behavior is not completion.

## 3. Scope boundary

### In scope

- stable entity identity, runtime network IDs, UUID/profile association, type/state components, and
  versioned persistence;
- player list, join/leave visibility, other-player spawn, movement, rotation, pose, metadata,
  equipment, animation, effects, and removal;
- observer interest, tracking range, update rate, visibility, chunk dependency, per-session
  backpressure, spawn-before-update, and remove ordering;
- item entities, experience orbs, falling blocks, projectiles, area effects, paintings, displays,
  and other audited nonliving entities required by the production manifest;
- damage sources, invulnerability, armor/effects joins, melee attacks, knockback, projectiles,
  explosions, death, drops, experience, and multiplayer PvP policy;
- boats, minecarts, mounts, passengers, leashes, vehicle input, dismount, and cross-Region movement;
- mob spawning caps and conditions, despawn, navigation, pathfinding budgets, goals/brain behavior,
  sensing, targeting, combat, breeding, taming, trading, raids/bosses, and special audited behavior;
- entity/player Region transfer with dual-generation fencing, duplicate suppression, observer
  convergence, rollback refusal, and restart recovery;
- multiplayer chat/effect broadcasts using the Goal 05 policy and permission boundaries;
- exact-client MCP scenarios using at least two isolated clients where shared observation matters.

### Out of scope

- production multi-process ownership, gateway failover, and remote Region routing owned by Goal 07;
- plugins, custom entities, arbitrary mod synchronization, or cross-version protocol translation;
- autonomous MCP gameplay AI or using direct server mutation as acceptance evidence;
- claiming every rare mob behavior from one aggregate smoke scenario; production rows require
  focused evidence proportional to the behavior.

## 4. Entity and observer rules

- Stable identity is independent from per-session network IDs and survives save/reload and Region
  transfer where the entity is durable.
- An observer receives spawn before state deltas and remove after its final authoritative visibility
  decision; replay and duplicate delivery are idempotent.
- Tracking depends on committed chunk/entity state, not uncommitted simulation or client prediction.
- Combat effects that cross player, entity, inventory, world, and progression systems have an
  explicit deterministic commit order.
- Pathfinding, AI, spawning, tracking, and projection are bounded and report overload without
  silently skipping authority-changing work.
- Cross-Region transfer has one live owner, generation fencing, durable handoff where required, and
  convergence tests for observers on both sides.

## 5. Phased batches

### Phase 0 — Freeze entity and multiplayer truth

| Batch | Outcome |
|---|---|
| `G06-P0-B1` | Commit the entity taxonomy, state/persistence formats, observer contract, production-manifest denominator, budgets, and multi-client acceptance matrix. |

### Phase 1 — Multiplayer player replication

| Batch | Outcome |
|---|---|
| `G06-P1-B1` | Integrate player list, profile visibility, join/leave, spawn/remove, and per-session network identity. |
| `G06-P1-B2` | Project committed player movement, rotation, pose, metadata, equipment, effects, animations, and corrections to observers. |
| `G06-P1-B3` | Add bounded interest/tracking, chunk dependency, visibility policy, update rates, and per-observer backpressure. |

### Phase 2 — Entity lifecycle and tracking

| Batch | Outcome |
|---|---|
| `G06-P2-B1` | Integrate durable entity spawn, mutation, removal, persistence, restore, network IDs, and metadata projection. |
| `G06-P2-B2` | Integrate item entities, experience orbs, falling blocks, projectiles, area effects, and audited special nonliving entities. |
| `G06-P2-B3` | Integrate fenced Region transfer, duplicate suppression, observer convergence, restart recovery, and fault injection. |

### Phase 3 — Combat and vehicles

| Batch | Outcome |
|---|---|
| `G06-P3-B1` | Integrate melee interaction, damage, armor/effects, invulnerability, knockback, death, drops, experience, and PvP policy. |
| `G06-P3-B2` | Integrate projectile launch/flight/hit, explosions, area effects, and cross-Region consequences. |
| `G06-P3-B3` | Integrate boats, minecarts, mounts, passengers, leashes, vehicle input, dismount, tracking, and transfer. |

### Phase 4 — Mobs, spawning, navigation, and AI

| Batch | Outcome |
|---|---|
| `G06-P4-B1` | Integrate spawning categories, caps, conditions, persistence, despawn, and client visibility. |
| `G06-P4-B2` | Integrate bounded navigation/pathfinding, sensing, target selection, goals/brain scheduling, and deterministic replay. |
| `G06-P4-B3` | Integrate passive behavior, breeding, taming, ownership, trading, and population joins. |
| `G06-P4-B4` | Integrate hostile combat, special abilities, raids, bosses, and remaining audited mob production rows. |

### Phase 5 — Multi-client acceptance and completion

| Batch | Outcome |
|---|---|
| `G06-P5-B1` | Add two-client MCP scenarios for mutual visibility, chat, equipment, block effects, combat, death/respawn, entities, and disconnect. |
| `G06-P5-B2` | Add exact-client vehicle, projectile, mob, AI, Region-transfer, restart, overload, and visual scenarios. |
| `G06-P5-B3` | Close Goal 06 production rows, replay/fault/performance/source audits, clean-checkout gates, and publish the completion record. |

## 6. Required verification

Every Rust batch runs focused affected-crate tests plus:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Tracking and transfer batches require loss, duplication, reordering, stale generation, capacity,
disconnect, restart, and observer convergence tests. Player-visible shared-world batches require
two isolated Goal 02 clients, structured observations from both clients, and screenshots when visual
state is part of the claim.

## 7. Terminal acceptance

- [ ] Two formal-server clients see consistent player list, spawn, movement, pose, equipment, effects, actions, and removal.
- [ ] Observer tracking is interest-based, ordered, bounded, replay-safe, and sourced from committed state.
- [ ] Durable entities preserve stable identity and state across restart and Region transfer.
- [ ] Items, experience orbs, projectiles, falling blocks, area effects, and required special entities have production behavior and projection.
- [ ] Melee, damage, armor/effects, knockback, projectiles, explosions, death, drops, experience, and PvP converge across clients.
- [ ] Vehicles, mounts, passengers, input, dismount, tracking, and Region transfer work through ordinary client controls.
- [ ] Mob spawning, despawn, navigation, AI, targeting, breeding/taming/trading, raids, bosses, and required special behaviors close their production rows.
- [ ] Cross-Region entity transfer preserves single ownership and observer convergence under faults and restart.
- [ ] Multi-client MCP scenarios prove shared behavior without server-side mutation shortcuts.
- [ ] Universal gates, performance/fault evidence, source review, and clean-worktree acceptance pass.

Goal 06 is complete only when formal-server clients share one authoritative entity world with
production multiplayer semantics.
