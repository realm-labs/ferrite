# Minecraft Java Edition 26.2 Behavioral Reference

This is Ferrite's version-locked reference for observable gameplay behavior in Minecraft: Java Edition `26.2`. Before implementing or testing a mechanic, consult its mini-specification and turn unresolved details into evidence instead of filling gaps from memory.

The baseline is locked by the [official 26.2 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-2) and the official version manifest: Data Pack `107.1`, Resource Pack `88.0`. See the [source lock](sources.md) for artifact SHA-1 values, report-generation procedures, and legal boundaries.

This English library is the single normative documentation source. Keeping one maintained language avoids mirror drift; rule IDs and evidence IDs remain the stable references used by implementation and tests.

## Scope

The library covers every observable gameplay subsystem at mini-specification granularity. It does not restate every one of roughly 1,196 blocks or every item and mob. Content-specific facts must be read from the locked jars' bundled data and `--reports` output.

In scope:

- server-authoritative state, transitions, and ordering;
- edge cases and quirks that a player can observe or exploit;
- observable client prediction, server rejection, and correction semantics;
- the way data-driven content parameterizes generic algorithms.

Out of scope:

- vanilla packet formats, save formats, and renderer internals;
- repository copies of decompiled sources, Mojang assets, Wiki prose, or generated reports;
- block-for-block same-seed world-generation identity. Ferrite retains the existing architecture's player-visible-equivalence goal.

## Specification index

| # | Specification |
|---:|---|
| 1 | [Ticks, time, and chunks](behavior/01-tick-time-and-chunks.md) |
| 2 | [Blocks and updates](behavior/02-blocks-and-updates.md) |
| 3 | [Environment](behavior/03-environment.md) |
| 4 | [Redstone and explosions](behavior/04-redstone-and-explosions.md) |
| 5 | [Player movement and interaction](behavior/05-player-movement-and-interaction.md) |
| 6 | [Items, inventories, and progression](behavior/06-items-inventories-and-progression.md) |
| 7 | [Entities and combat](behavior/07-entities-and-combat.md) |
| 8 | [Mobs, AI, and spawning](behavior/08-mobs-ai-and-spawning.md) |
| 9 | [World generation and dimensions](behavior/09-worldgen-and-dimensions.md) |
| 10 | [Client-observable behavior](behavior/10-client-observable-behavior.md) |

Companion documents:

- [Methodology](methodology.md)
- [Source lock](sources.md)

## Usage

1. Use the stable rule ID to find the behavioral boundary and current evidence status.
2. A `Confirmed` rule may directly drive implementation and black-box tests. A `Cross-checked` rule still needs an experiment when its edge cases matter.
3. Never silently turn a `Provisional` or `Conflict` rule into implementation. First add an official source locator or a minimal reproducible observation as described by the methodology.
4. Read content-specific constants from Data Pack `107.1`, Resource Pack `88.0`, or generated reports; do not infer them from this prose.
5. Give later Minecraft versions sibling directories. Never silently rewrite conclusions locked to `26.2` here.

“Source” in this library means a class-and-method locator in an official jar. The prose is an independent behavioral specification and contains no Mojang implementation code.
