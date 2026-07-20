# Redstone mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`RED-EXPLOSION-001`](red-explosion-001.md)

Explosion calculation, entity effects, block effects, and fire are separate phases

### [`RED-UPDATE-001`](red-update-001.md)

Power changes propagate through component callbacks, not a global circuit solve

### [`RED-DAYLIGHT-DETECTOR-001`](red-daylight-detector-001.md)

Daylight detectors sample effective sky light and a positional sun-angle attribute every 20 server
ticks

### [`RED-COMPARATOR-001`](red-comparator-001.md)

Comparators cache an analog result, then expose it through a two-tick directional transaction

### [`RED-DELAY-001`](red-delay-001.md)

Repeaters, observers, and torches schedule component-owned transitions

### [`RED-PISTON-001`](red-piston-001.md)

A piston resolves a finite move plan before executing its block event
