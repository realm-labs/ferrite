# G04-P4-B2 — Authoritative portal travel

## Result

The formal Minecraft gateway now drives Nether and End travel from committed Region-owned chunks.
Portal contact, destination generation, safe creation, player ownership transfer, and Java dimension
transition are one bounded production flow; no `MinimalTerrain` or collision fixture participates.

## Production chain

- Compact and exact-report registry modes map obsidian, both Nether portal axes, and End portal
  surfaces to formal block-state identities with their real collision and opacity semantics.
- A session observes portal contact from dimension-scoped projectable snapshots and applies the
  audited cooldown and wait processor. Nether travel scales coordinates and searches bounded loaded
  POIs; End entry creates the audited platform and exit target.
- Portal tickets drive the destination dimension's ordinary formal chunk lifecycle. Resolution
  waits until every queried chunk and owning Region is present.
- Multi-block world writes preflight authority, duplicate positions, chunk presence, and every
  expected revision before cloning, mutating, relighting, and replacing an owning Region's columns.
  Failed preflight leaves all columns unchanged.
- Dimension transfer now permits different dimensions only under the same world and mapping domain.
  Commit installs the destination pose and resets movement, floating, loading, velocity, and
  correction baselines.
- Java Respawn, destination border, global spawn, End level event, correction, portal cooldown, and
  a restarted chunk stream are emitted only after the transfer appears in the committed tick.

## Focused evidence

- `minecraft::portal::formal_portal_contact_generates_a_durable_exit_and_commits_dimension_transfer`
  boots a configured three-dimension formal world, writes a source portal through authoritative
  block service, waits through contact, drives destination generation, creates obsidian/portal
  authority, and observes a committed player transfer into the Nether.
- Portal unit tests cover both Nether axes, End contact, coordinate scaling, safe portal creation,
  exact End-platform writes, and player target selection.
- `world_service::runtime::block_transaction_preflights_every_revision_and_commits_all_writes_together`
  proves all-or-nothing mutation within an owning Region.
- Region-runtime and player-session tests prove same-world cross-dimension transfer while retaining
  rejection across world or mapping domains.
- The world-service architectural golden digest advances to
  `19140a5608d4549ca22a1895d8f83ecfa96cfd216eae22de83089e7829e33fd1` because formal portal
  states now contribute their empty-collision and zero-opacity semantics during deterministic
  generated-state relighting; repeated conformance runs produce the same digest.
- Focused affected-crate tests and Clippy pass before the universal batch gates.

## Boundary

The transaction is atomic inside each Region and fenced before all cross-Region admissions. Formal
checkpoint recovery already persists the resulting chunks and entity state, but interrupted
multi-Region publication, stale-generation failure injection, and restart convergence are claimed
only after `G04-P4-B3`. Exact Java 26.2 client observation remains `G04-P5-B1`.
