# G02-P1-B1 — Reproducible client mod build

## Outcome

The repository now owns a standalone, client-only Fabric project at
`tools/ferrite-client-mcp`. It compiles against Java 25 and the exact Minecraft 26.2 boundary,
produces a remapped test-instrumentation mod, and does not embed Minecraft or Fabric classes.

## Locked build boundary

| Component | Version |
|---|---:|
| Gradle wrapper | `9.6.1` |
| Fabric Loom | `1.17.17` |
| Java language/runtime | `25` |
| Minecraft | `26.2` |
| Fabric Loader | `0.19.3` |
| Fabric API | `0.154.1+26.2` |
| JUnit | `6.0.3` |

The wrapper distribution has a committed SHA-256, resolved dependency versions are stored in
`gradle.lockfile`, and artifact checksums are stored in `gradle/verification-metadata.xml`. The
primary dependency and reviewed-reference licenses are recorded in `THIRD_PARTY_NOTICES.md`.

## Verification

The following commands passed on 2026-08-01:

```text
JAVA_HOME=<local-jdk-25> \
  ./tools/ferrite-client-mcp/gradlew --no-daemon check
JAVA_HOME=<local-jdk-25> \
  ./tools/ferrite-client-mcp/gradlew --no-daemon build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
git diff --check
```

Two forced builds produced the same mod digest:

```text
e0a5259e585655c7f464b11fdd6c4ca7e4e5bd367cf9494aba8e0d0ed822fb39
```

The mod JAR contains only `fabric.mod.json` and
`dev/ferrite/client/mcp/FerriteClientMcp.class` apart from archive metadata. Mojang jars, assets,
mappings payloads, Gradle caches, and generated client state remain untracked.
