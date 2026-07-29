# Ferrite fuzz targets

The protocol fuzz package is intentionally isolated from the ordinary workspace target directory.
Run it with a dedicated cache:

```text
cargo install cargo-fuzz
CARGO_TARGET_DIR=target/fuzz cargo +nightly fuzz run frame_stream
CARGO_TARGET_DIR=target/fuzz cargo +nightly fuzz run wire_primitives
```

`frame_stream` exercises arbitrary TCP fragmentation, frame lengths, compression envelopes, and
terminal fault behavior. `wire_primitives` exercises variable integers, bounded UTF strings, and
bounded byte arrays. Neither target may allocate beyond the limits selected by the harness.
