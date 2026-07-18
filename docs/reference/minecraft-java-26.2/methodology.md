# Evidence and Authoring Method

## 1. Normative language

“Must,” “must not,” and “should” describe requirements Ferrite needs to satisfy to reproduce `26.2`. “Vanilla” means only the official Minecraft: Java Edition `26.2` artifacts in the source lock, not a Wiki, mod loader, or another version.

Mini-specifications describe observable semantics. They do not require Ferrite to reuse vanilla class structure, threading, or data structures. Packet, save, and renderer internals enter a rule only when they affect a player-observable result.

## 2. Rule record

Every rule must contain the fields below and use a stable, unique rule ID:

```text
Rule ID
FidelityClass
Evidence status
Primary evidence
Applies when
Behavior / state transition / ordering
Boundary cases and known quirks
Open verification
```

Empty fields with no semantic value may be combined, but an unresolved `Open verification` must not be omitted. A source locator has the form `fully.qualified.Class#method(parameter types)`. Overloads require parameter types; fields and data use their complete path inside the jar.

## 3. FidelityClass

| Value | Meaning |
|---|---|
| `ExactObservableBehavior` | Player-, command-, and black-box-observable state, ordering, numbers, and quirks should match. Internal architecture may differ. |
| `EquivalentPlayerVisibleBehavior` | Player results should be equivalent, while random sampling, asynchronous scheduling, or internal representation may differ. The rule must define the equivalence boundary. |
| `IntentionallyImprovedBehavior` | A deliberate vanilla deviation. Its rationale, compatibility impact, and migration path must be recorded. |
| `Unimplemented` | Not implemented yet. Target behavior and evidence remain documented so it is not later invented from memory. |

This reference library does not itself authorize `IntentionallyImprovedBehavior`. Such a deviation requires a separate architecture decision.

## 4. Evidence status

| Status | Admission criterion | Implementation meaning |
|---|---|---|
| `Confirmed` | Directly supported by official `26.2` source/bundled data, or stably reproduced by a minimal vanilla experiment. Ambiguous ordering normally requires one of these to expose it explicitly. | May drive a behavioral test; the evidence locator must remain. |
| `Cross-checked` | Official evidence supports the main conclusion and a second kind of evidence cross-checks it, but not every edge is covered. | The main path may be implemented; experiment before relying on an uncovered edge. |
| `Provisional` | Supported only by community material, a cross-version inference, or an incomplete locator. | Must not be treated as an exact compatibility conclusion. |
| `Conflict` | Evidence disagrees, or observations depend on a condition not yet isolated. | Do not guess; record the conflict and design a discriminating experiment. |

“Not implemented” is not an evidence status. Implementation progress belongs in `FidelityClass` and project tracking.

## 5. Evidence precedence

From strongest to weakest:

1. Classes, methods, and constants in the locked official client/server jars;
2. bundled Data Pack / Resource Pack data and `--reports` output from that server;
3. a minimal vanilla GameTest or dedicated-server black-box observation;
4. the [Mojang 26.2 release notes](https://www.minecraft.net/en-us/article/minecraft-java-edition-26-2) and official bug records verified for the version;
5. community sources pinned to an `oldid` or Git commit, for cross-checking only.

A conclusion supported only by community material can be at most `Provisional`. When evidence conflicts, do not vote by source count: record `Conflict` and add an experiment that distinguishes the interpretations.

## 6. Source and data inspection

1. Resolve the `26.2` metadata URL from the official version manifest and verify the metadata SHA-1 first.
2. Download jars only from URLs in that metadata, then verify length and SHA-1.
3. Inspect classes and methods in a temporary directory. Prefer `jar tf` and `javap -p -s -c`; if a decompiler is needed, its output still must not enter the repository.
4. Generate reports with `java -DbundlerMainClass=net.minecraft.data.Main -jar server.jar --reports --output <temp-dir>`.
5. Write an independent state-machine or timing description. Record only class/method/descriptor and data paths; do not copy implementation source.

Source control flow can prove call ordering and branch conditions, but does not automatically prove what a client observes. Conclusions involving latency, prediction, or UI also require client-path inspection or black-box observation.

## 7. Black-box and GameTest records

Each new experiment should retain rebuildable steps or a script outside the repository and summarize at least the following in its rule or a dedicated issue:

- experiment ID, rule ID, and `26.2` artifact SHA-1;
- the fresh world's or structure template's initial state, including seed, dimension, difficulty, game rules, and player count;
- server tick of each input;
- commands, block/entity state, and observation points;
- at least one control and enough repetitions to expose randomness;
- result summary, reproduction stability, and possible confounders.

The GameTest entry point `net.minecraft.gametest.Main` was verified in the locked server. Prefer experiments for ambiguous update ordering, scheduled/random ticks, falling blocks, fluids, redstone, movement, combat, and spawn caps.

## 8. Randomness, time, and ordering

- Equal values do not imply equal ordering. If behavior depends on queue order, neighbor notification, or entity iteration, the rule and test must state the event sequence.
- Lock an RNG algorithm only when a stable seed-to-result mapping is observed and the project needs exact reproduction; otherwise specify distributions and player-visible constraints.
- Keep “game tick,” “client tick,” “render frame,” and “wall time” distinct.
- Separately verify unloaded-area suspension, catch-up after loading, and server-lag handling.

## 9. Bugs and quirks

An official bug record must include its MC number, affected conditions, reproducible symptom, and verified version. Always write `Replication decision: Undecided` until an architecture decision is explicit. Resolved, duplicate, or works-as-intended status does not by itself decide Ferrite's policy. See [Bug Us About Bugs](https://www.minecraft.net/en-us/article/bug-us-about-bugs) for the official reporting and search workflow.

## 10. Single-source maintenance and acceptance

- English is the single normative documentation source; do not add independently maintained language mirrors to this version directory.
- Rule IDs and evidence IDs remain stable after publication. A semantic change must record why its locked `26.2` conclusion changed.
- Every community URL must pin a revision and record its access date.
- Before merging, check Markdown links, cited symbols, `git diff --check`, `cargo test`, and repository hygiene.
- No jar, decompiled source, generated report, test world, or Mojang asset may enter the repository.

Legal and naming evidence is centralized in the [source lock](sources.md).
