# Ferrite fuzz and property targets

The standalone fuzz package exercises protocol and deterministic runtime boundaries without
sharing the ordinary workspace target directory. Install `cargo-fuzz`, then invoke it through the
repository cache wrapper:

```text
cargo install cargo-fuzz
cargo ferrite cargo fuzz +nightly fuzz run --fuzz-dir fuzz frame_stream \
  target/fuzz-corpus/frame_stream fuzz/corpus/frame_stream
```

The first corpus path is a writable exploration corpus under the ignored workspace target tree.
The second path contains committed seeds and minimized regressions. A discovered crash must be
reproduced and minimized before its input is copied into the matching committed corpus directory.
Raw `fuzz/artifacts` output remains ignored so transient or duplicate failures are not committed.

| Target | Boundary under test |
| --- | --- |
| `frame_stream` | TCP fragmentation, frame limits, compression envelopes, and terminal faults |
| `wire_primitives` | Variable integers, bounded UTF strings, and bounded byte arrays |
| `command_ordering` | Bounded command admission, canonical source ordering, and committed pruning |
| `region_boundary` | Generation fencing, phase/tick routing, canonical source ordering, and capacity |
| `persistence_recovery` | Snapshot/tail decoding, canonical round trips, digests, and corrupted bytes |
| `replay_codec` | Command/event envelopes, replay headers/frames/logs, and primitive decoders |

CI formats and lints every target. A bounded hardening run can use the following shape for each
target, keeping generated corpus growth outside version control:

```text
mkdir -p target/fuzz-corpus/<target>
cargo ferrite cargo fuzz +nightly fuzz run --fuzz-dir fuzz <target> \
  target/fuzz-corpus/<target> fuzz/corpus/<target> -- \
  -runs=10000 -max_len=4096 -timeout=5
```
