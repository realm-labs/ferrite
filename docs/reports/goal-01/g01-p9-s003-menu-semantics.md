# G01-P9-S003 — client menu semantics

## Result

Complete. `CLI-MENU-SEMANTICS-001` now has an executable Java 26.2 client model in
`behavior-runner`, joined through `ferrite-testkit` to the existing serverbound container
transaction and clientbound container projection implementations.

## Observable behavior

The gesture model locks expanded half-open slot hover geometry and first-active-slot ordering;
strict 249/250-ms double-click separation; empty-carried press commitment and one-release
suppression; creative clone, Shift quick move, outside throw, pickup, offhand, and hotbar mappings;
and carried-stack release behavior.

Quick craft retains its initiating button and type, admits compatible slots once through all gates,
recomputes capacity-aware preview remainder, cancels on a mismatched release, and emits the exact
start/add/end mask sequence. Double-left wins over the drag tail, including ordered Shift
quick-moves for matching pickup-allowed slots in the same container. Closing discards all
screen-owned gesture state. Keyboard handling preserves inventory-close priority, clone/drop
priority, Control throw strength, and the empty-carried offhand/hotbar gate.

The dialog model validates and submits all four registered input controls. It covers boolean
defaults and byte tags; descending, equal, stepped, integer, and float number ranges; encoded-order
single options; single- and multiline text defaults, limits, heights, and escaped template
substitution. Unknown controls produce an explicit logged/no-widget/no-getter disposition.

## Cross-system convergence

The testkit fixture verifies that a mismatched container ID does not run local prediction, a
matching prediction emits once, and a stale state-ID click still executes before the server sends a
full content resync. It then applies matching content, proves delayed wrong-container content cannot
overwrite the open menu, and proves server close abandons the open client menu regardless of the
packet's container ID.

Server click replay, 15-bit state arithmetic, remote stack mirrors, dedicated menu controls, and
individual `ContainerInput` mutations remain in their completed `ITM-CONTAINER-*` owners rather
than being duplicated in the client model.

## Evidence

Implementation owners:

- `behavior_runner::client::menu`;
- existing `ferrite_protocol::java_26_2::play::{serverbound,clientbound}::container` behavior.

Committed test owner:

- `apps/behavior-runner/tests/client/cli_005.rs`.

Focused validation:

```text
cargo test -p behavior-runner --test client cli_005
8 passed; 0 failed
cargo clippy -p behavior-runner -p ferrite-testkit --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Phase 9 surface and cross-system aggregation remains owned by `G01-P9-B1`.
