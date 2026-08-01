# G01-P10-B2 Fuzz and Property Hardening

## Result

Ferrite's standalone fuzz package now covers six protocol, deterministic ordering, Region,
persistence, corruption, and replay boundaries. Every target completed a 10,000-run bounded
AddressSanitizer/libFuzzer campaign from a committed seed without a crash, timeout, or retained
artifact.

## Coverage-guided campaigns

| Target | Runs | Final coverage / features | Final generated corpus | Result |
|---|---:|---:|---:|---|
| `frame_stream` | 10,000 | 211 / 334 | 50 inputs / 840 bytes | Pass |
| `wire_primitives` | 10,000 | 240 / 391 | 103 inputs / 1,133 bytes | Pass |
| `command_ordering` | 10,000 | 460 / 1,532 | 89 inputs / 3,387 bytes | Pass |
| `region_boundary` | 10,000 | 449 / 1,705 | 103 inputs / 3,805 bytes | Pass |
| `persistence_recovery` | 10,000 | 606 / 1,307 | 74 inputs / 1,250 bytes | Pass |
| `replay_codec` | 10,000 | 190 / 219 | 56 inputs / 950 bytes | Pass |

The command and Region harnesses assert bounded admission, canonical drain ordering, generation
fencing, phase/tick routing, and committed-state pruning across arbitrary operation sequences. The
persistence and replay harnesses combine arbitrary decoding with canonical encode/decode and
digest properties, then exercise corrupted recovery bytes and all replay envelope/log forms.

## Failure-corpus policy and hardening

Each target has a small committed seed in `fuzz/corpus/<target>`. Exploration output is directed to
ignored `target/fuzz-corpus/<target>` directories. If a future campaign finds a failure, the input
must be reproduced and minimized before it becomes a committed regression corpus entry; transient
`fuzz/artifacts` output stays ignored.

Coverage-guided decoding identified attacker-controlled count fields that could reserve their full
declared maximum before proving that an element existed. Snapshot records, journal tails, and
replay sequences now grow only after successful element decoding. Unit regressions cover truncated
maximum-count inputs, and the persistence/replay fuzz campaigns completed without excessive
allocation or failure.

## Commands

```text
cargo fmt --all -- --check
cargo fmt --manifest-path fuzz/Cargo.toml --all -- --check
cargo ferrite cargo fuzz clippy --manifest-path fuzz/Cargo.toml --bins -- -D warnings
cargo test -p ferrite-persistence -p ferrite-replay -p ferrite-simulation
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo ferrite task check
git diff --check
```

Each fuzz campaign used this isolated-cache form with its corresponding target name:

```text
cargo ferrite cargo fuzz +nightly fuzz run --fuzz-dir fuzz <target> \
  target/fuzz-corpus/<target> fuzz/corpus/<target> -- \
  -runs=10000 -max_len=4096 -timeout=5
```
