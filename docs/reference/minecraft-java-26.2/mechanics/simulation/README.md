# Simulation mechanics

[Back to the leaf-rule manual](../README.md).

Each file contains one implementation-level leaf rule. Stable rule IDs remain the normative
references used by behavior pages, the completion ledger, the catalog, and tests.

## Leaf rules

### [`SIM-PIPELINE-001`](sim-pipeline-001.md)

One server tick has ordered ownership boundaries

### [`SIM-SCHEDULE-001`](sim-schedule-001.md)

Scheduled block and fluid ticks are bounded priority queues

### [`SIM-RANDOM-001`](sim-random-001.md)

Random ticks are sampled attempts, never accumulated obligations

### [`SIM-COMMAND-LIMIT-001`](sim-command-limit-001.md)

Command contexts snapshot strict fork and cost budgets
