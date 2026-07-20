# Environment mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`ENV-LIGHT-001`](env-light-001.md)

Sky and block light propagate as separate bounded channels

### [`ENV-FLUID-001`](env-fluid-001.md)

Fluid propagation recomputes local state through scheduled ticks

### [`ENV-WEATHER-001`](env-weather-001.md)

Weather timers and exposed local effects are separate layers

### [`ENV-FIRE-001`](env-fire-001.md)

Fire aging, extinguishing, spread, and fuel destruction are ordered scheduled-callback branches

### [`ENV-GEYSER-001`](env-geyser-001.md)

Potent sulfur derives five fluid states and runs a positional geyser clock
