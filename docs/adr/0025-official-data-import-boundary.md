# ADR-0025: Import Official Data Locally Without Committing Mojang Artifacts

## Status

Accepted

## Context

Ferrite must cover 9,078 locked catalog IDs and exact protocol/registry projections, but official
JARs, assets, generated reports, and decompiled sources are not project-authored source code.
Production cannot depend on the audit tool or on a developer's extraction cache.

## Decision

The repository commits:

- project-owned schemas, algorithms, mappings, import code, and small authored fixtures;
- version locks, expected counts/digests, and provenance;
- validation and drift tests;
- documented legal and operational instructions.

The repository does not commit:

- Mojang client/server JARs or assets;
- generated Mojang reports or copied registry/data tables;
- decompiled source or `javap` output;
- experiment worlds containing redistributed official content.

A deterministic local import step consumes user-provided or officially fetched artifacts matching
`docs/reference/minecraft-java-26.2/lock.toml`. It verifies size and SHA-1 before extraction, validates
the generated bundle schema and all catalog/protocol digests, records provenance, and writes an
ignored build artifact. Production crates consume the validated project bundle schema, never
`mc-reference` APIs or raw extraction paths.

The import step fails closed on version, hash, count, schema, classification, or mapping drift.
Generated runtime numeric IDs are bundle-local and never persisted as stable content identity.

## Consequences

- A clean source checkout does not redistribute official artifacts.
- Developers need a documented import/bootstrap step before full content tests.
- Runtime startup can validate one compact bundle digest instead of re-auditing JARs.
- Reference tooling remains a development dependency outside the production graph.

## Alternatives Considered

- Commit extracted official data: rejected due to provenance, repository size, and redistribution
  concerns.
- Parse official JARs in production: rejected because startup, deployment, and trust boundaries would
  be uncontrolled.
- Hand-maintain copied tables: rejected because they drift and obscure provenance.

## Migration or Reversal Plan

New Minecraft versions get sibling locks/import schemas. A bundle-schema change is versioned and
supports deterministic regeneration; legal-policy changes require explicit review before artifacts
move across the boundary.
