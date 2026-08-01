# Goal 05 — Player Survival Systems

## 1. Objective

Make the formal server support a durable, authoritative Minecraft 26.2 survival-player loop. Player
identity, inventory, equipment, item use, block interaction, containers, crafting, health, hunger,
experience, progression, death, respawn, chat, commands, and administration must be driven by
ordinary client packets and projected back to the exact client.

The Goal replaces fixed-stone placement and packet acknowledgement without gameplay semantics with
stateful services that survive disconnect and restart.

## 2. Production evidence contract

Every player-visible feature must prove:

```text
normal exact-client input
  -> decoded packet and admitted semantic action
  -> player/Region-owned authoritative mutation
  -> inventory/menu/world transaction and durable state
  -> revisioned client projection or explicit rejection/correction
  -> disconnect/restart continuity
  -> MCP state, GUI, interaction, and visual assertions
```

An item algorithm, packet codec, menu trace, or client-side prediction without authoritative commit
does not satisfy the Goal.

## 3. Scope boundary

### In scope

- durable player identity, profile, gamemode, position, inventory, equipment, selected slot, health,
  hunger, experience, effects, statistics, recipe book, and progression state;
- online/offline admission policy selected by configuration, plus whitelist, bans, operators, and
  bounded permission checks;
- authoritative hotbar, main inventory, armor, offhand, cursor stack, item components, stacking,
  durability, cooldown, use duration, and equipment changes;
- held-item-driven block placement, interaction, break progress, tool checks, drops, loot, pickup,
  and prediction correction;
- container/menu lifecycle, IDs, state revisions, slot transactions, close/reopen, distance and
  ownership validation;
- crafting, recipe placement, furnace, brewing, enchanting, anvil, smithing, beacon, merchant, and
  other audited workstation services required by the production manifest;
- health, hunger, saturation, exhaustion, regeneration, environmental damage, experience,
  status effects, death, drops, keep-inventory rules, respawn, and bed/spawn-anchor state;
- advancements, statistics, recipes, and progression projections covered by the audited baseline;
- chat, command parsing/execution/suggestions, secure-chat policy, sign/book text handling, and
  responsibility-based administration;
- game rules, difficulty, default gamemode, PvP policy, view/simulation settings, status/MOTD, and
  resource-pack gates required for a usable operator configuration;
- exact-client MCP scenarios for inventory, GUI, block/item, survival, command, disconnect, and
  restart behavior.

### Out of scope

- complete nonplayer entity tracking, mob AI, combat targets, vehicles, and multiplayer replication
  owned by Goal 06;
- real distributed player/session ownership and gateway failover owned by Goal 07;
- plugins, arbitrary mod APIs, cross-version clients, or a general third-party permission API;
- server commands or direct state mutation as gameplay acceptance evidence.

## 4. Authority and transaction rules

- Player state has one authoritative owner and a versioned durable representation.
- Client inventory/menu revisions are requests, not authority; stale or impossible transactions are
  rejected and corrected with the authoritative state.
- Block placement derives item, count, state properties, shape, and permissions from committed
  player/world state; no fixed placement state exists in production.
- Block breaking derives progress and outcome from block, tool, effects, gamemode, and captured
  context; drops and durability commit atomically with removal where required.
- Commands use typed effects and permission checks; they do not bypass Region ownership.
- Disconnect, death, dimension transfer, Region transfer, save, and shutdown have explicit
  inventory/menu/progression continuity boundaries.
- Exact-client MCP tools provide input and observation only and never become an authority oracle.

## 5. Phased batches

### Phase 0 — Freeze player production truth

| Batch | Outcome |
|---|---|
| `G05-P0-B1` | Commit the player state model, authority boundaries, service inventory, protocol/manifest denominator, configuration formats, and acceptance matrix. |

### Phase 1 — Durable player identity and admission

| Batch | Outcome |
|---|---|
| `G05-P1-B1` | Integrate durable player load/save, session epochs, reconnect, position/gamemode continuity, and corrupt-state recovery policy. |
| `G05-P1-B2` | Add configured authentication mode, whitelist, bans, operators, permissions, status identity, and bounded admission outcomes. |
| `G05-P1-B3` | Add authoritative hotbar, inventory, armor, offhand, cursor, equipment, and revisioned client synchronization. |

### Phase 2 — Items and block interaction

| Batch | Outcome |
|---|---|
| `G05-P2-B1` | Integrate item stacks/components, selection, swap, drop, pickup, equipment, use, cooldown, and durability. |
| `G05-P2-B2` | Replace fixed-state placement with held-item placement, block-state properties, permissions, collision checks, consumption, and correction. |
| `G05-P2-B3` | Integrate timed/tool-aware breaking, block hooks, loot, drops, experience, durability, and atomic world/player commit. |

### Phase 3 — Containers and crafting

| Batch | Outcome |
|---|---|
| `G05-P3-B1` | Install menu ownership, IDs, revisions, cursor/slot transactions, validation, corrections, and close/reconnect behavior. |
| `G05-P3-B2` | Integrate player and crafting-table recipes, recipe placement, result consumption, recipe book, and projections. |
| `G05-P3-B3` | Integrate furnace/brewing and remaining audited workstation menus in responsibility-bounded batches. |
| `G05-P3-B4` | Integrate merchant, beacon, enchanting, anvil, smithing, sign, and book services plus restart continuity. |

### Phase 4 — Survival and progression

| Batch | Outcome |
|---|---|
| `G05-P4-B1` | Integrate health, damage sources needed without Goal 06 entities, hunger, exhaustion, regeneration, effects, and difficulty. |
| `G05-P4-B2` | Integrate death, drops, keep-inventory rules, respawn, spawn points, experience, and recovery continuity. |
| `G05-P4-B3` | Integrate advancements, statistics, recipes, progression, and client projection. |

### Phase 5 — Chat, commands, and operations

| Batch | Outcome |
|---|---|
| `G05-P5-B1` | Integrate chat/session policy, text filtering boundary, signs/books, acknowledgements, and multiplayer-ready broadcast effects. |
| `G05-P5-B2` | Integrate command parsing, suggestions, typed effects, permissions, game rules, difficulty, gamemode, and operator commands. |
| `G05-P5-B3` | Complete status/MOTD, view/simulation configuration, resource-pack gates, rate limits, and administration audit. |

### Phase 6 — Real-client acceptance and completion

| Batch | Outcome |
|---|---|
| `G05-P6-B1` | Add exact-client MCP inventory, placement/breaking, crafting, container, survival, death/respawn, chat/command, and restart scenarios. |
| `G05-P6-B2` | Close all Goal 05 production-manifest rows, fault/replay/security/source audits, clean-checkout gates, and publish the completion record. |

## 6. Required verification

Every Rust batch runs focused affected-crate tests plus:

```text
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Transaction batches require stale revision, duplicate sequence, overflow, disconnect, transfer,
restart, and malicious packet tests. Player-visible batches require Goal 02 MCP operations through
normal client input, structured inventory/screen/player observations, and screenshots where visual
state matters.

## 7. Terminal acceptance

- [ ] Player identity, location, gamemode, inventory, equipment, health, hunger, experience, effects, recipes, and progression survive reconnect and restart.
- [ ] Admission, whitelist, bans, operators, permissions, status identity, and authentication mode are configured and fail closed.
- [ ] Production block placement is driven by the held item and authoritative world/player state; fixed-stone placement is removed.
- [ ] Breaking time, tools, loot, drops, pickup, experience, and durability commit consistently.
- [ ] Inventory and menu transactions are revisioned, bounded, replay-safe, and corrected on invalid input.
- [ ] Required crafting and workstation services are reachable through ordinary client GUIs.
- [ ] Health, hunger, effects, death, drops, respawn, and progression form a durable survival loop.
- [ ] Chat, commands, suggestions, permissions, signs, books, game rules, and operator settings have production semantics.
- [ ] Exact-client MCP scenarios exercise input, GUI, server authority, restart, and visual convergence without direct mutation.
- [ ] All Goal 05 production rows, security/fault checks, universal gates, and clean-worktree acceptance pass.

Goal 05 is complete only when a player can perform a durable survival loop through the formal server
using ordinary client behavior.
