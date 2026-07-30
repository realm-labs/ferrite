# Phase 6 Player-Lifecycle Conformance

`G01-P6-B2` closes Phase 6 through the `PlayerLifecycle` root surface and its two phase-owned
cross-system joins. The conformance boundary uses production lifecycle, session-routing and Region
tick code; the testkit supplies only deterministic fixtures, fixed seeds, fault injection and
reports.

## Production ownership

`ferrite-gameplay::player::lifecycle` is split by responsibility:

- `admission` evaluates user ban, whitelist, IP ban and capacity/bypass in the audited strict order;
- `model` names protocol-neutral state, projections and ordered lifecycle effects;
- `runtime` owns live membership and the join, death, replacement respawn, teleport, game-mode,
  permission and disconnect transactions.

Join suspends projection flushing around the ordered initial client trace. Ordinary death and
won-game replacement retain distinct keep masks, and keep-inventory is recorded separately from
keep-all restoration. Spectator entry and exit preserve their asymmetric cleanup. Disconnect emits
player, statistics and advancements persistence in order before membership/index removal.

## Network ingress join

`SessionBridge` continues to own Connected → Routed → Configuration → Play transitions. A successful
Play admission routes exactly one bounded `session/join` command. Closing a Play session now routes
one `session/leave` command to the same Region and removes the connection/profile indexes only after
the route succeeds.

This makes close admission atomic with Region delivery. A full, stale or unavailable Region route
leaves the session in Play so the caller can retry; it cannot silently orphan a Region-owned player.
`PlayerRegionLogic` applies join and leave only in the Ingress phase, journals both, and performs the
entity membership mutation inside the tick transaction.

## Tick-snapshot join

The scheduler suite proves:

- a join admitted before a tick becomes visible at that tick's Ingress capture;
- a leave admitted for the next tick cannot leak backward into the previous snapshot;
- the player disappears at the leave tick's Ingress phase;
- same-tick join then leave follows command sequence and ends absent;
- duplicate join, unknown leave, stale command and excessive-future command faults fail closed.

The fixed-seed sweep repeats this boundary for 64 stable player identities. Death, respawn,
cross-dimension relocation, mode/ability changes and disconnect ordering are covered by the root
surface trace and replay suite.

## Executable suites

The `ferrite-testkit::phase6` harnesses provide:

- a 121-event lifecycle golden/client trace with digest
  `f6124ba1095b689b2e41e81f63ec6521a59007ce4f5bdf6415536b76ab324ea4`;
- 128 ordered-admission properties;
- 256 fixed-seed lifecycle operation fuzz cases;
- eight lifecycle fault vectors;
- six replay frames plus an intentional divergence;
- 64 NetworkIngress transition properties and five route/state faults;
- 64 TickScheduler membership properties and five capture/admission/source faults.

The machine-owned test entry points are:

- `apps/behavior-runner/tests/surfaces/player_lifecycle.rs`;
- `apps/behavior-runner/tests/joins/network_ingress_player_lifecycle.rs`;
- `apps/behavior-runner/tests/joins/tick_scheduler_player_lifecycle.rs`.
