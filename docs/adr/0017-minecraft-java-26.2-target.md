# ADR-0017: Lock Initial Compatibility to Minecraft Java 26.2

## Status

Accepted

## Context

Minecraft protocol IDs, registries, data packs, and observable behavior change between releases.
Claiming generic compatibility before one version is complete would weaken the audited denominator.

## Decision

Goal 01 supports an unmodified Minecraft Java Edition 26.2 client against the exact locked official
artifacts:

- server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`;
- client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`;
- Data Pack `107.1`;
- Resource Pack `88.0`;
- protocol inventory digest `f34b0956b6399c749d4638cd6d3c9226685f41fa`.

The required compatibility contract is all specified C0-C3 families. Goal 01 uses offline login and
explicitly refuses the wrong protocol version. C4 services remain disabled or gated exactly as their
reference families specify; listing a C4 family does not claim its enabled external service.

The 327 source-specified slices and source-known parts of four inconclusive slices are required. The
four exact unresolved observations remain `DeferredExperiment`, never guessed vanilla behavior.

## Consequences

- Compatibility claims have a finite, machine-verifiable denominator.
- A later Minecraft release requires a sibling adapter/reference and explicit migration work.
- One runtime may eventually host multiple adapters, but no 26.2 numeric ID becomes a domain ID.

## Alternatives Considered

- Track latest Minecraft continuously: rejected because audit and implementation would never share a
  stable target.
- Support a version range in one codec: rejected because packet and registry differences would leak
  conditionals across the server.
- Implement only a custom client: rejected because Goal 01 explicitly requires an unmodified client.

## Migration or Reversal Plan

Add another version-locked reference and adapter. Share only semantic types proven version-neutral;
retain the 26.2 adapter and acceptance suite until its support policy is separately changed.
