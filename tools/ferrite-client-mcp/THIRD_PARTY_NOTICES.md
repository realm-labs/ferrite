# Third-party inventory

The build resolves, but does not redistribute in the Ferrite Client MCP mod JAR, these primary
dependencies:

| Component | Locked version | License / terms | Purpose |
|---|---:|---|---|
| Gradle | 9.6.1 | Apache-2.0 | Checked-in build wrapper |
| Fabric Loom | 1.17.17 | LGPL-3.0-only | Minecraft/Fabric build plugin |
| Fabric Loader | 0.19.3 | Apache-2.0 | Client mod loader API |
| Fabric API | 0.154.1+26.2 | Apache-2.0 | Client lifecycle and event APIs |
| Gson | 2.14.0 | Apache-2.0 | Bounded JSON-RPC parsing and serialization |
| JUnit | 6.0.3 | EPL-2.0 | Test framework |
| Minecraft Java Edition | 26.2 | Mojang EULA | External client under test |

The design review also used `cuspymd/mcp-server-mod` at
`43dcec547ad3a5ca6b6e0e2e1b37f5c2a6581cfe` (CC0-1.0) and `lucasoyen/MCCTP` at
`50ffa27b04a934d105c2ae9b79f10fc50651f20d` (MIT) as references. Their repositories and binaries
are not vendored, and this initial project skeleton does not copy their source code.

Generated dependency locks and verification metadata are authoritative for resolved transitive
artifacts. Mojang game artifacts remain only in the local Gradle cache and must never be committed.
