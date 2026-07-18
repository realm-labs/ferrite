# Source Lock and Evidence Catalog

- **Target version:** Minecraft: Java Edition `26.2`
- **Release date:** 2026-06-16
- **Verification date:** 2026-07-18
- **Data Pack:** `107.1`
- **Resource Pack:** `88.0`
- **Required Java major:** `25` (the version metadata's `javaVersion.majorVersion`)

This page locks evidence identities without committing copyrighted artifacts. The machine-readable lock is [`lock.toml`](lock.toml). Downloads, extracted jars, report output, libraries, logs, and test worlds live only under the ignored `target/mc-reference/26.2/` cache.

## Official artifacts

| Evidence ID | Artifact | Locked value | Official location and purpose |
|---|---|---|---|
| `OFF-MANIFEST-001` | Version manifest v2 | `latest.release = 26.2` when verified; the `26.2` entry points to the locked metadata | [version_manifest_v2.json](https://piston-meta.mojang.com/mc/game/version_manifest_v2.json); used only to resolve versions, never treating `latest` as permanent. |
| `OFF-META-001` | `26.2.json` | SHA-1 `9e272020887b72a23b1a525c5d8e74d2d7aa8222` | [locked metadata](https://piston-meta.mojang.com/v1/packages/9e272020887b72a23b1a525c5d8e74d2d7aa8222/26.2.json); normative source for download URLs, sizes, Java version, and asset index. |
| `OFF-SERVER-001` | Official server jar | SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a`; size `60,894,273` bytes | [server.jar](https://piston-data.mojang.com/v1/objects/823e2250d24b3ddac457a60c92a6a941943fcd6a/server.jar) locked by `downloads.server` in `OFF-META-001`; used for server classes, bundled data, GameTest, and reports. Official entry: [server download](https://www.minecraft.net/en-us/download/server). |
| `OFF-CLIENT-001` | Official client jar | SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`; size `39,193,383` bytes | [client.jar](https://piston-data.mojang.com/v1/objects/2dc72797acbc1b63fc16a11c4ac393605f453754/client.jar) locked by `downloads.client` in `OFF-META-001`; used for input, prediction, correction, UI, sound, and particle paths. |
| `OFF-DATA-001` | Bundled server data | Implicitly locked by the `OFF-SERVER-001` SHA-1 | `data/minecraft/**` and pack metadata; primary evidence for content constants, tags, loot tables, recipes, worldgen, and related data. |
| `OFF-REPORT-001` | `--reports` output | Rebuildable from `OFF-SERVER-001` with the command below | `reports/blocks.json`, `registries.json`, `commands.json`, `packets.json`, `datapack.json`, `json-rpc-api-schema.json`, and per-item component reports. `blocks.json` contained `1,196` block keys in this verification. |
| `OFF-REL-001` | 26.2 release notes | Data Pack `107.1`; Resource Pack `88.0` | [Minecraft Java Edition 26.2](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-2). |
| `OFF-NAMES-001` | Readable technical-name notice | Release obfuscation removed from 2025 onward | [Removing obfuscation in Java Edition](https://www.minecraft.net/en-us/article/removing-obfuscation-in-java-edition); names are inspectable, but licensing is unchanged. |
| `OFF-EULA-001` | Minecraft EULA | Accessed 2026-07-18 | [Minecraft EULA](https://www.minecraft.net/en-us/eula); governs artifact use and redistribution. |
| `OFF-BUG-001` | Official bug-query guide | Accessed 2026-07-18 | [Bug Us About Bugs](https://www.minecraft.net/en-us/article/bug-us-about-bugs); MC-number, search, and reporting workflow. |

## Rebuild procedure

The normative workflow is the checked Rust tool:

```sh
cargo run -p mc-reference --bin mc-ref -- fetch --version 26.2
cargo run -p mc-reference --bin mc-ref -- reports
cargo run -p mc-reference --bin mc-ref -- verify
```

The commands below document the underlying official operations for independent audit.

Verify SHA-1 values:

```sh
shasum -a 1 26.2.json server.jar client.jar
```

The server jar is a bundler. Select the report entry point through its bundler property:

```sh
java -DbundlerMainClass=net.minecraft.data.Main \
  -jar server.jar --reports --output /tmp/minecraft-26.2-reports
```

Verify the GameTest entry point and options without creating a world:

```sh
java -DbundlerMainClass=net.minecraft.gametest.Main -jar server.jar --help
```

The report set and block count observed here are completeness sentinels, not substitutes for artifact SHA-1. If a later run differs under the same SHA-1, first inspect Java version, working directory, and command-line arguments.

## Community cross-checks

Every Minecraft Wiki page below is pinned to an `oldid` and was accessed on 2026-07-18. These sources cannot alone promote a rule to `Confirmed`, and their prose must not be copied.

| Evidence ID | Pinned page |
|---|---|
| `COM-WIKI-SIM-001` | [Tick, oldid 3665485](https://minecraft.wiki/w/Tick?oldid=3665485); [Chunk, oldid 3582335](https://minecraft.wiki/w/Chunk?oldid=3582335) |
| `COM-WIKI-BLK-001` | [Block states, oldid 3650554](https://minecraft.wiki/w/Block_states?oldid=3650554); [Block update, oldid 3678708](https://minecraft.wiki/w/Block_update?oldid=3678708); [Breaking, oldid 3665464](https://minecraft.wiki/w/Breaking?oldid=3665464) |
| `COM-WIKI-ENV-001` | [Fluid, oldid 3422932](https://minecraft.wiki/w/Fluid?oldid=3422932); [Light, oldid 3631949](https://minecraft.wiki/w/Light?oldid=3631949); [Weather, oldid 3662149](https://minecraft.wiki/w/Weather?oldid=3662149); [Fire, oldid 3676879](https://minecraft.wiki/w/Fire?oldid=3676879) |
| `COM-WIKI-RED-001` | [Redstone mechanics, oldid 3660859](https://minecraft.wiki/w/Redstone_mechanics?oldid=3660859); [Piston, oldid 3666334](https://minecraft.wiki/w/Piston?oldid=3666334); [Explosion, oldid 3679385](https://minecraft.wiki/w/Explosion?oldid=3679385) |
| `COM-WIKI-PLY-001` | [Player, oldid 3675429](https://minecraft.wiki/w/Player?oldid=3675429); [Breaking, oldid 3665464](https://minecraft.wiki/w/Breaking?oldid=3665464) |
| `COM-WIKI-ITM-001` | [Item, oldid 3663845](https://minecraft.wiki/w/Item?oldid=3663845); [Inventory, oldid 3668315](https://minecraft.wiki/w/Inventory?oldid=3668315); [Crafting, oldid 3665598](https://minecraft.wiki/w/Crafting?oldid=3665598); [Smelting, oldid 3672136](https://minecraft.wiki/w/Smelting?oldid=3672136); [Brewing, oldid 3670923](https://minecraft.wiki/w/Brewing?oldid=3670923); [Enchantment, oldid 3679347](https://minecraft.wiki/w/Enchantment?oldid=3679347); [Loot table, oldid 3672885](https://minecraft.wiki/w/Loot_table?oldid=3672885); [Food, oldid 3679471](https://minecraft.wiki/w/Food?oldid=3679471); [Experience, oldid 3667280](https://minecraft.wiki/w/Experience?oldid=3667280); [Advancement, oldid 3677959](https://minecraft.wiki/w/Advancement?oldid=3677959) |
| `COM-WIKI-ENT-001` | [Entity, oldid 3679511](https://minecraft.wiki/w/Entity?oldid=3679511); [Damage, oldid 3678782](https://minecraft.wiki/w/Damage?oldid=3678782); [Effect, oldid 3669051](https://minecraft.wiki/w/Effect?oldid=3669051); [Projectile, oldid 3424874](https://minecraft.wiki/w/Projectile?oldid=3424874); [Minecart, oldid 3677502](https://minecraft.wiki/w/Minecart?oldid=3677502); [Boat, oldid 3674212](https://minecraft.wiki/w/Boat?oldid=3674212) |
| `COM-WIKI-MOB-001` | [Mob spawning, oldid 3664946](https://minecraft.wiki/w/Mob_spawning?oldid=3664946); [Spawn, oldid 3665779](https://minecraft.wiki/w/Spawn?oldid=3665779); [Mob AI, oldid 3678872](https://minecraft.wiki/w/Mob_AI?oldid=3678872); [Breeding, oldid 3675070](https://minecraft.wiki/w/Breeding?oldid=3675070); [Taming, oldid 3615362](https://minecraft.wiki/w/Taming?oldid=3615362) |
| `COM-WIKI-WGEN-001` | [World generation, oldid 3644276](https://minecraft.wiki/w/World_generation?oldid=3644276); [Biome, oldid 3679358](https://minecraft.wiki/w/Biome?oldid=3679358); [Feature, oldid 3630050](https://minecraft.wiki/w/Feature?oldid=3630050); [Structure, oldid 3676676](https://minecraft.wiki/w/Structure?oldid=3676676); [Dimension, oldid 3675296](https://minecraft.wiki/w/Dimension?oldid=3675296); [Nether portal, oldid 3644461](https://minecraft.wiki/w/Nether_Portal?oldid=3644461); [End portal, oldid 3641734](https://minecraft.wiki/w/End_portal?oldid=3641734); [World border, oldid 3644380](https://minecraft.wiki/w/World_border?oldid=3644380) |
| `COM-WIKI-RULE-001` | [Game rule, oldid 3662108](https://minecraft.wiki/w/Game_rule?oldid=3662108) |
| `COM-DATA-001` | [PrismarineJS/minecraft-data commit 7048d46b7c95328e508b2732137a6020acb9971c](https://github.com/PrismarineJS/minecraft-data/tree/7048d46b7c95328e508b2732137a6020acb9971c); names, enumerations, and cross-implementation discrepancy hints only. |

## Source-locator convention

- The actual version jar inside the server bundler is the inspection target; the outer `server.jar` launcher is not gameplay source.
- Class and method names come from the readable names in `OFF-SERVER-001` / `OFF-CLIENT-001`. Descriptors are verified with `javap -s`.
- Citing a class and method proves only that the corresponding logic exists. The behavioral conclusion must also match control flow, data, or experiment.
- Optional bundled packs such as `data/minecraft/datapacks/redstone_experiments` are outside the default baseline unless a rule states the enabling condition.

## Copyright and repository hygiene

[Removing obfuscation](https://www.minecraft.net/en-us/article/removing-obfuscation-in-java-edition) explains that new jars retain readable technical names, but does not grant redistribution rights. The [Minecraft EULA](https://www.minecraft.net/en-us/eula) still applies. The repository therefore stores only independently summarized behavior, symbol locators, hashes, and reproduction steps—never jars, decompiled text, Mojang assets, generated reports, test worlds, or Minecraft Wiki prose.
