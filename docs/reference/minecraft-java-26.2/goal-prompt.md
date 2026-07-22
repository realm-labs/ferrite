# Ferrite Minecraft 26.2 Reference Goal Prompt

This is the copy-ready Codex Goal Prompt for continuously completing the version-locked behavior
and protocol reference. Copy only the contents of the block below.

```text
Continuously complete Ferrite's English, version-locked Minecraft Java Edition
26.2 compatibility reference for both observable server behavior and
unmodified-client wire-protocol compatibility.

Treat docs/architecture.md as the read-only architectural contract. Limit
changes to docs/reference/minecraft-java-26.2/, tools/mc-reference/, and
supporting Cargo workspace configuration. Do not modify Ferrite runtime code,
server implementation code, or public runtime APIs. Tooling CLI and validation
changes are allowed only when they directly support the reference workflow.

At the start of every continuation, inspect:

- the current Git status and recent commits;
- docs/reference/minecraft-java-26.2/completion.toml;
- docs/reference/minecraft-java-26.2/catalog/catalog.toml;
- docs/reference/minecraft-java-26.2/behavior-surfaces.toml;
- docs/reference/minecraft-java-26.2/protocol/completion.toml;
- current gameplay, behavior-surface, and protocol coverage/readiness output.

Preserve unrelated or user-owned worktree changes. Do not redo completed
batches or keep following stale one-time repair instructions. Fix currently
reproducible integrity defects before expanding coverage. Trust the current
ledgers and validation output over historical counts or priorities in this
prompt.

Work in independently reviewable compatibility-slice batches. A slice should
combine the relevant behavior, content, behavior-surface, and protocol paths
when they form one observable interaction, for example:

- handshake, status, login, configuration, and join;
- registry projection, chunks, lighting, dimension entry, and respawn;
- movement, teleport acknowledgement, prediction, and correction;
- block prediction, placement, breaking, and authoritative updates;
- entities, metadata, attributes, equipment, effects, and passengers;
- inventories, menus, state IDs, clicks, and resynchronization;
- sounds, particles, game events, commands, chat, and disconnect behavior;
- save/reload continuity and cross-system ordering at any of those boundaries.

Maintain four recoverable reference artifacts across three independently
inspectable validation gates:

1. completion.toml owns observable gameplay slices and registry scopes.
2. catalog/catalog.toml owns exhaustive locked content-ID classification.
3. behavior-surfaces.toml owns root behavior-entry and behavior-exit surfaces.
4. protocol/completion.toml owns protocol inventory and compatibility.

The three gates are gameplay slice/catalog completion, behavior-surface
readiness, and protocol readiness. The aggregate gameplay readiness command may
report both slice/catalog and surface blockers, but each source of failure must
remain independently inspectable so one green ledger cannot hide another.

Preserve the existing gameplay requirements:

- cover every independent semantic of all 65 parent rules and all ten
  subsystems;
- cover all 95 official registries with justified scopes;
- classify every observable registry ID as an audited inherited behavior
  family, explicit special rule, genuinely data-only entry, or explicit
  recoverable Unreviewed work;
- require FidelityClass, EvidenceStatus, SourceConclusion, ownership, evidence,
  boundaries, and executable test vectors for every leaf;
- resolve catalog rule, owner, semantic-link, matcher, fallback, and coverage
  integrity defects;
- never treat structural one-owner catalog coverage as proof that every ID's
  behavior has been audited.

Maintain all ten behavior-surface root kinds exactly once:

- TickScheduler;
- NetworkIngress;
- CommandAdministration;
- ContentDispatch;
- PlayerLifecycle;
- WorldLifecycle;
- PersistenceReload;
- ClientProjection;
- DataReload;
- CrossSystemOrdering.

Every behavior surface must record:

- its observable boundary and triggers;
- authoritative inventory sources and exhaustive selectors;
- semantic rule owners and any joined protocol families;
- authoritative state domains read or changed;
- persistence, unload/reload, reconnect, and restart boundaries;
- client projection and correction boundaries;
- evidence, status, exact unknowns, and reproducible remaining work.

Every referenced rule owner and protocol family must resolve. A field that does
not apply must explicitly say why rather than remaining empty. Treat behavior-
surface status as ownership progress, not mechanic completion. Mapped means the
root inventory and owners are explicit; it must not promote or hide Todo,
InProgress, Unreviewed, or SourceInconclusive work in referenced ledgers.

Use the surface-by-state-domain cross-product to find missing joins. Every
non-empty join must have an owner for admission, ordering, atomicity, conflict
resolution, persistence, and client projection, or an explicit justified
non-interaction conclusion. Do not assume that separate domain rules fully
specify their interaction.

For every protocol packet family, record:

- connection state and direction;
- namespaced packet identity and locked numeric ID;
- exact field order, primitive encoding, conditional fields, and bounds;
- legal sender state and semantic preconditions;
- normalized Ferrite ingress or egress mapping;
- connection-local state read or changed;
- ordering, acknowledgement, prediction, and correction relationships;
- vanilla-relevant rejection behavior for invalid, stale, duplicate, or
  illegal-state input;
- wire-visible registry, entity-metadata, data-component, and other mapping
  requirements;
- primary official jar symbols and report evidence;
- golden bytes, boundary cases, rejection cases, and end-to-end trace vectors.

Use the protocol compatibility levels in
docs/reference/minecraft-java-26.2/protocol/README.md as organization, not as a
stale fixed work order:

- C0: status discovery and ping;
- C1: offline-mode login, configuration, and minimal play entry;
- C2: chunks, movement, correction, keepalive, and block interaction;
- C3: entities, inventories, containers, effects, commands, and core survival;
- C4: optional authenticated online-mode client interoperability and broad
  supported-gameplay conformance.

Choose the next batch from current readiness blockers. Do not revisit a
completed protocol level unless a newly reproducible integrity defect or a
gameplay/surface join exposes missing work. When protocol readiness is already
green, prioritize explicit surface Todo/InProgress roots, Unreviewed catalog
families, incomplete gameplay slices, and source-inconclusive experiments.

Derive behavior and protocol independently from the locked official 26.2
source, client/server codecs and handlers, bundled data, generated reports, and
directed experiments. Do not copy Mojang code, generated packet tables, jars,
assets, captures, Wiki prose, or other copyrighted artifacts into the
repository. Do not guess from older versions, community descriptions, or
memory.

Treat reports/packets.json as evidence for state, direction, packet identity,
and numeric ID only. Derive field layouts, bounds, state transitions, ordering,
and semantic consequences from the locked client/server artifacts and directed
experiments. Encode/decode round trips alone are not compatibility evidence.

Keep protocol work strictly within vanilla-client interoperability against the
locked local artifacts. Do not expand the task into offensive security
analysis, authentication circumvention, credential or session collection, or
testing against third-party infrastructure. Authentication and encryption are
in scope only to the extent required to document the official client/server
compatibility boundary.

When official source is inconclusive, record the exact unknown, inspected
source boundary, affected behavior surface or compatibility level, owner, and
reproducible experiment. Store raw jars, reports, logs, traces, captures, test
worlds, and generated artifacts only under ignored
target/mc-reference/26.2/ paths.

After every batch:

1. Run the relevant mc-ref query and symbol checks.
2. Run catalog coverage and aggregate gameplay readiness.
3. Run behavior-surface coverage, readiness, and offline verification.
4. Run protocol inventory, coverage, readiness, and offline verification.
5. Run experiment-definition verification and full offline reference
   verification.
6. Run workspace tests and formatting in proportion to the changed tooling.
7. Run diff checks, link checks, generated-artifact checks, and repository
   hygiene checks.
8. Review the complete batch diff for copied artifacts, stale counts, false
   completion, unsupported conclusions, and unrelated changes.
9. Create one atomic local Conventional Commit containing only that batch.
10. Do not push.
11. Immediately continue with the next highest-priority incomplete slice.

A readiness command is expected to exit nonzero while its recoverable work
remains. Do not weaken validation, delete explicit backlog, or broaden a family
merely to make a gate green. Structural verification should still pass while
truthful Todo, InProgress, Unreviewed, or SourceInconclusive entries remain.

Continue until all of the following are true:

- aggregate gameplay readiness reports no unfinished or unaudited work;
- all 65 parent rules, ten subsystems, 95 scoped registries, and observable
  registry IDs satisfy their completion contract;
- every locked catalog ID has an audited exact/pattern family or a justified
  data-only classification, with no Unreviewed fallback remaining;
- behavior-surface readiness has zero Todo and zero InProgress roots;
- all ten behavior-surface kinds have exhaustive inventories and valid owners;
- every non-empty surface/state-domain join has an ordering, atomicity,
  persistence, projection, or justified non-interaction owner;
- protocol readiness inventories every locked connection state, direction, and
  packet identity from the official packet report;
- every protocol packet is assigned to a complete packet-family specification,
  an explicit gated/optional path, or a justified non-server responsibility;
- every implemented or required packet family has exact field, bounds,
  transition, semantic-mapping, ordering, acknowledgement, and conformance
  coverage;
- every wire-visible registry and metadata mapping required by C0 through C4
  has an audited owner and verification path;
- all source-derivable behavior and protocol semantics are specified;
- all source-inconclusive items have exact unknowns, owners, and reproduction
  procedures;
- gameplay slice/catalog, behavior-surface, and protocol readiness are
  independently complete;
- all consistency, validation, formatting, workspace-test, diff, link, and
  repository-hygiene checks pass;
- a final atomic Conventional Commit records compatibility-reference
  completion.

Do not stop merely because the prose is extensive, the original client can
connect, one readiness ledger is green, or the remaining work is difficult.
Stop only at the stated completion conditions or at a genuine external blocker
that cannot be resolved from the locked artifacts and reproducible experiments.
```
