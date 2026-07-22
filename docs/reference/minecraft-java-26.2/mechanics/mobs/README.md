# Mobs mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`MOB-SPAWN-001`](mob-spawn-001.md)

Natural spawning is a category-cap, chunk, position, and mob-rule pipeline

### [`MOB-HOSTILE-GATE-001`](mob-hostile-gate-001.md)

The hostile-spawn gamerule refreshes chunk policy and gates four direct spawn transactions

### [`MOB-PATROL-001`](mob-patrol-001.md)

Patrol spawning advances a pausable timer into one player-relative pillager group attempt

### [`MOB-PHANTOM-SPAWN-001`](mob-phantom-spawn-001.md)

Phantom spawning turns a shared timer into ordered per-player difficulty and insomnia trials

### [`MOB-WANDERING-TRADER-001`](mob-wandering-trader-001.md)

Persisted delay and escalating chance admit one trader with up to two llamas

### [`MOB-AI-001`](mob-ai-001.md)

Mob AI arbitrates goals, navigation, controls, senses, and memory on entity ticks

### [`MOB-DESPAWN-001`](mob-despawn-001.md)

Persistence and distance checks choose immediate removal, random removal, or retention

### [`MOB-BREED-001`](mob-breed-001.md)

Love, mate selection, child creation, cooldown, and ownership inheritance commit together
