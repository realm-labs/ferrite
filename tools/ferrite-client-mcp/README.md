# Ferrite Client MCP

This client-only Fabric mod instruments the locked Minecraft Java 26.2 client for Ferrite
acceptance testing. It is test infrastructure, not part of the Ferrite server runtime and not an
unmodified-client compatibility claim.

## Build

Use a Java 25 runtime to launch the checked-in Gradle wrapper:

```text
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon check
JAVA_HOME=/path/to/jdk-25 ./gradlew --no-daemon build
```

The remapped mod artifact is written below `build/libs`. Gradle caches, local client state, and run
directories are ignored. Do not place Mojang jars, assets, mappings payloads, access tokens, or a
personal Minecraft game directory in source control.

The complete scope, security boundary, and acceptance requirements are defined in
`docs/goals/02-client-mcp-automation.md` at the repository root.
