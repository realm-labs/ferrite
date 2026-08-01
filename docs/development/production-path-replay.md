# Production path replay evidence

`ProductionTickReplayEvidence` is the canonical local proof that the formal server traversed one
ingress-to-projection path. It is captured only from a completed `CompositeGatewayTickReport` and
binds:

- the committed tick and canonical local ingress command metadata;
- every Region identity in stable order;
- each composite replay identity;
- each committed continuity hash and exact record count;
- each committed projection's owner, sequence, responsibility identity, and semantic payload.

Capture rejects a Region whose composite commit or continuity tick differs from the formal tick,
whose continuity receipt differs from the retained record set, whose projection count differs from
the commit receipt, or whose projection cannot pass the formal session decoder. The top-level
digest therefore cannot certify an ingress-only or conformance-only execution.

Admission order is intentionally absent from the identity. Local command capture, player ownership
reconciliation, composite service execution, continuity normalization, and projection ordering all
use canonical keys. Two formal routes receiving the same commands in different arrival order must
produce byte-identical replay evidence.

Failure is fail-stop. A capacity error before composite commit returns no evidence or commit
receipt and poisons the local executor/composite route so the partially attempted tick cannot be
retried as if it had rolled back. Malformed or unknown committed projections fail evidence capture
and session routing closed.
