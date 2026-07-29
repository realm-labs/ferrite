# Source Policy Exceptions

There are no active source-policy exceptions.

`G01-P1-B6` removed the legacy `tools/mc-reference/src/lib.rs` exception by separating artifact
handling, catalogs, symbols, experiments, protocol, behavior surfaces, and aggregate verification
into named modules. The repository source gate now checks every handwritten Rust file.
