# G02-P5-B1 — Client MCP fault hardening

## Result

`Satisfied`. The pure-Java client MCP now fails closed across its authenticated HTTP, JSON-RPC,
tool, render, input, process, and locked-artifact boundaries. A tool implementation or synchronous
renderer failure produces a generic MCP tool error without terminating or poisoning the active
session. Invalid launcher readiness data terminates the owned process tree before the acceptance
runner returns an error.

## Fault matrix

| Boundary | Injected fault | Required outcome | Evidence |
|---|---|---|---|
| Authentication and origin | Missing/wrong bearer and hostile origin | `401`/`403`, no session allocation, later valid initialization succeeds | `McpHttpServerTest.authenticationOriginMediaBoundsAndMethodsFailClosed` |
| MCP framing and lifecycle | Invalid JSON, batch/non-object message, wrong protocol header, second concurrent session | Bounded JSON-RPC or HTTP error; first session remains authoritative | `McpProtocolTest.malformedBatchWrongVersionAndConcurrentSessionFailClosed` |
| Tool isolation | Tool throws with a sensitive exception message | Generic `isError` result, no detail disclosure, same session still answers ping | `McpProtocolTest.crashingToolFailsClosedWithoutDisclosingOrLosingTheSession` |
| Overload | Oversized HTTP body, full 64-action queue, concurrent session admission | `413` or explicit rejection; priority input release remains admitted | `McpHttpServerTest`, `ClientActionQueueTest.priorityReleaseBypassesAFullQueueAndCancelsOutstandingWork`, `McpProtocolTest` |
| Disconnect and stuck input | An applied held input is cancelled by a world/connection transition | Receipt becomes `Cancelled`; reservation is released and can be reacquired | `ClientActionQueueTest.disconnectCancellationReleasesAStuckInputReservation` and the exact-client disconnect proof in [G02-P3-B1](g02-p3-b1-tick-fenced-client-control.md) |
| Render absence | Unavailable renderer, synchronous renderer crash, busy capture, asynchronous failure, and timeout | Bounded redacted tool error; timed-out future is cancelled | `TakeScreenshotToolTest` |
| Process crash and cleanup | Already-crashed child and live Java parent with a live Java descendant | Termination is idempotent; owned parent and descendant are both gone within the bound | `ProcessTreeTest` |
| Launcher readiness | Missing, malformed, unsafe, or rejected first readiness record | Owned process tree is terminated before an `IOException` escapes | `ManagedLauncher.start` guarded readiness boundary |
| Artifact mismatch | Wrong-size client/server and same-size client with the wrong SHA-1 | Rejected before launch or server readiness | `LauncherConfigTest.rejectsAnArtifactThatDoesNotMatchTheLockedClient`, `rejectsASameSizeClientWithTheWrongDigest` |

The process test launches its fixtures with the current Java executable and therefore exercises
the same cross-platform `ProcessHandle` boundary used by unattended acceptance. The artifact digest
test creates a sparse file with the exact locked byte length, ensuring the digest path rather than
only the size check is exercised.

## Implementation notes

- `McpProtocol` catches runtime failures at the individual tool boundary and returns stable,
  secret-free tool content. It does not catch VM errors.
- `TakeScreenshotTool` handles both a failed future and a capture implementation that throws before
  returning a future.
- `ManagedLauncher` validates the complete readiness record inside a cleanup guard.
- `ProcessTree` snapshots owned descendants before graceful termination and force-terminates any
  survivor after the grace period.
- Existing action duration bounds, queue admission, priority `release_all_inputs`, client-thread
  world/disconnect release, and shutdown release remain unchanged.

## Verification

The following commands passed on 2026-08-01:

```text
JAVA_HOME=/Users/mikai/Library/Java/JavaVirtualMachines/azul-25/Contents/Home ./gradlew --no-daemon clean check build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

All handwritten Java sources remain below the repository's 1,200-line limit; the largest is 356
physical lines.
