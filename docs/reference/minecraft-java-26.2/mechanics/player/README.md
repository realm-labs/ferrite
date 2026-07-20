# Player mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`PLY-MOVE-001`](ply-move-001.md)

Ordinary ground and air travel integrates input, jump, gravity and drag in source order

### [`PLY-COLLISION-001`](ply-collision-001.md)

Generic entity movement clips a swept box, selects a step candidate and derives collision state

### [`PLY-MOVE-SPECIAL-001`](ply-move-special-001.md)

Fluid, swimming, fall-flying and ability-flight dynamics remain separate modes

### [`PLY-MOVE-VALIDATE-001`](ply-move-validate-001.md)

Server movement-packet admission and correction are a distinct authority transaction

### [`PLY-INPUT-001`](ply-input-001.md)

Key state, posture and item-use modifiers shape movement intent before travel

### [`PLY-AUTOJUMP-001`](ply-autojump-001.md)

Obstacle geometry schedules a later synthetic jump press

### [`PLY-INTERACT-001`](ply-interact-001.md)

Use selects entity, block, and item paths with explicit pass/fail semantics

### [`PLY-BREAK-001`](ply-break-001.md)

Client breaking predicts progress and rolls retained states forward on acknowledgement
