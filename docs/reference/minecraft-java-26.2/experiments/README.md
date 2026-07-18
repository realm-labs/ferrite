# Directed behavior experiments

Experiments resolve high-impact behavior that readable source/data does not unambiguously fix, especially same-tick order, floating-point boundaries, RNG consumption and client correction. [`definitions.toml`](definitions.toml) contains original, machine-validated procedures; [results](results.md) records observations only after execution against the locked artifacts.

Statuses:

- `planned`: complete procedure and expected invariant, not yet executed; the linked behavior remains `Cross-checked` or an implementation blocker.
- `automated`: a committed runner exists and `mc-ref experiment run <id>` must reproduce it.
- `observed`: the result summary contains artifact hash, environment, repetitions and observations; it is not automatically repeatable.

Generated packs, server libraries, logs and worlds belong under `target/mc-reference/26.2/`. An automated runner may create them there but must not place Mojang code/data in the repository.

`mc-ref experiment run <id>` always materializes the normalized procedure at `target/mc-reference/26.2/experiments/<id>/procedure.json`. A `planned` experiment then stops without claiming evidence. An `automated` runner receives `MC_REF_CACHE`, `MC_REF_EXPERIMENT_DIR`, and `MC_REF_SERVER_JAR`; it must write a fresh `result.json` containing `passed: true` and a nonempty `observations` array. This prevents a successful process exit or a stale result file from being mistaken for behavioral evidence.
