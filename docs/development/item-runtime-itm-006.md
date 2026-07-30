# ITM-006 Enchantment and Loot Runtime

`G01-P6-S007` implements the two audited `ITM-006` slices: the data-driven enchantment compositor
and the generic loot-table evaluator.

## Runtime boundary

`item::runtime::enchantment` owns:

- all eight equipment slots in source order and active-enchantment entry iteration;
- generic typed effect-list composition through resource identities, keeping effect data in the
  registry layer instead of a closed implementation enum;
- non-short-circuit equipment immunity and victim-before-attacker post-hit visitation;
- mutual exclusivity, inclusive cost-range candidate construction and weighted selection;
- exact table offer cost, ordinary-book removal, resource admission, book transmutation,
  `slot + 1` level/lapis spending and seed refresh.

Stored book enchantments remain separate from active runtime hooks. Generic value callbacks retain
the mutable result between effects and expose Java integer truncation plus the helper-specific
zero clamp.

`item::runtime::loot` is divided into four responsibilities:

- `context` defines all 26 registered context sets and rejects missing required or disallowed
  parameters;
- `model` provides codec-neutral descriptors for the three loot data kinds and preserves table,
  pool, entry, condition and function list order;
- `evaluator` drives registry-supplied condition, number-provider, entry and function dispatch,
  recursion breadcrumbs, dynamic weights, nested tables, feature filtering and maximum-stack
  splitting;
- `fill` owns the one-generation container algorithm: empty-slot shuffle, repeatable stack
  splitting, two conditional requeue booleans, final shuffle and end-of-list slot consumption.

Concrete JSON codecs and individual vanilla effect, condition, function, entry, number-provider and
slot-source data stay in the versioned registry/import layer. The gameplay evaluator owns their
shared ordering and branching contract through `LootDispatch`.

## Randomness and Region ownership

Both runtimes depend on the checked caller-owned `GameplayRandom` interface. Invalid bounded or
float draws fail rather than being silently clamped. The Region owner can therefore attach the
appropriate named deterministic stream without embedding topology or process identity into item
logic.

Loot random ownership is explicit: a supplied source wins, then a nonzero explicit seed, then the
table random sequence, then the level stream. Seed zero retains vanilla's randomize behavior.
Nested tables share the context, visited set and random source. A consumer sees one generated
sequence; refusal does not evaluate the table again.

## Validation

`crates/ferrite-gameplay/tests/slices/items/itm_006.rs` verifies hook/effect/equipment ordering,
stored-book inactivity, non-short-circuit immunity, post-hit target order, compatibility,
offer/selection RNG traces, menu spending and seed refresh, the exact 26 context-set catalog,
random-source precedence, condition short-circuiting, single-entry draw elision, dynamic weight
re-query, nested function order, recursion, stack boundaries and partial-container fill.
