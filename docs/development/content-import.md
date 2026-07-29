# Local Minecraft Content Import

Ferrite does not commit Mojang JARs, generated reports, extracted data, or the generated runtime
bundle. The project commits the importer, its schema, source locks, expected aggregate digests, and
small authored tests. The generated bundle remains under the ignored `target/ferrite-content/`
tree.

## Build the locked 26.2 bundle

Prepare the locked local source cache:

```text
cargo run -q -p mc-reference --bin mc-ref -- fetch --version 26.2
cargo run -q -p mc-reference --bin mc-ref -- reports
```

Import and verify it:

```text
cargo ferrite content import
cargo ferrite content verify
```

The importer independently checks the client and server size and SHA-1 from `lock.toml`, reads
server data from the nested JAR inside the verified server bundler, checks all 32 catalog counts and
sorted ID SHA-1 digests, requires exactly one reviewed family for every ID, validates the project
bundle schema, and compares both aggregate BLAKE3 digests with `content-bundle.lock.toml`.

The default output is:

```text
target/ferrite-content/26.2/content-bundle.json
```

An explicit `--source` may point at another local cache containing the same locked artifacts. An
explicit `--output` is accepted only below `target/ferrite-content/`; the importer will not write
official content into a source or documentation directory.

## Run partition tests against the generated bundle

Ordinary workspace tests validate the committed category contracts without requiring official
artifacts. To additionally load the ignored bundle and validate all 32 runtime partitions:

```powershell
$env:FERRITE_CONTENT_BUNDLE = Resolve-Path target/ferrite-content/26.2/content-bundle.json
cargo test -p ferrite-registry --test catalog --all-features
Remove-Item Env:FERRITE_CONTENT_BUNDLE
```

Any source, schema, count, ID, classification, payload, bundle digest, or manifest digest drift
fails closed. Updating the committed digest lock requires a reviewed migration batch; the importer
never updates it automatically.
