# Required Play Clientbound Entry Protocol

Ferrite implements the 18 packets in the locked
`PROTO-PLAY-CLIENTBOUND-ENTRY-001` family as the configuration-to-terrain bridge for an
unmodified Minecraft Java 26.2 client:

| Identity | ID | Entry responsibility |
|---|---:|---|
| `minecraft:change_difficulty` | 10 | difficulty and lock state |
| `minecraft:commands` | 16 | Brigadier command graph |
| `minecraft:entity_event` | 34 | local permission tier |
| `minecraft:game_event` | 38 | level flags and terrain-load start |
| `minecraft:initialize_border` | 43 | initial border state |
| `minecraft:login` | 49 | client level creation |
| `minecraft:player_abilities` | 64 | local ability flags and speeds |
| `minecraft:player_info_update` | 70 | ordered player-list projection |
| `minecraft:player_position` | 72 | initial position and teleport acknowledgement |
| `minecraft:recipe_book_add` | 74 | displayed recipe entries |
| `minecraft:recipe_book_settings` | 76 | four recipe-book open/filter pairs |
| `minecraft:server_data` | 86 | list-record MOTD and icon update |
| `minecraft:set_default_spawn_position` | 97 | dimension-qualified spawn |
| `minecraft:set_held_slot` | 105 | selected hotbar slot |
| `minecraft:set_time` | 113 | game time and registry-backed clocks |
| `minecraft:ticking_state` | 127 | tick rate and frozen state |
| `minecraft:ticking_step` | 128 | frozen-step count |
| `minecraft:update_recipes` | 133 | complete recipe and stonecutter projection |

Packet lookup uses the generated locked catalog. A catalog entry outside this family is refused as
unsupported, an absent ID is refused as unknown, and trailing bytes are never accepted.

## Connection-local registries

Configuration produces a `PlayRegistries` snapshot for each connection. Numeric references for
dimension types, world clocks, command argument types, recipe and slot display types, recipe-book
categories, items, data-component types, and trim patterns resolve through that snapshot.
Encoding performs the inverse lookup. Missing registries, unknown raw IDs, and unknown identities
fail closed, so process-local registry indices cannot leak into the wire adapter.

Data-component values are type-dependent and not self-delimiting. Their value grammar is delegated
to the generated version-specific `ComponentValueDecoder`; the generic slot codec owns patch
counts, component IDs, duplicate detection, and exact encoded bytes. Added and removed components
share one uniqueness set.

## Structured payloads

The command codec covers every locked argument payload shape: numeric bounds, string kind, entity
and score-holder flags, time minimum, and registry identity. It validates only the graph reachable
from the root, rejects reachable cycles and invalid indices, and turns unknown argument types into
terminal placeholder nodes after consuming their optional suggestion provider.

Player-info actions are interpreted in bit order for all 256 masks. Profile names and properties,
chat keys and signatures, optional display components, and every action-specific field use their
locked bounds. A game-mode value outside `0..=3` falls back to survival.

Recipe dispatch covers all five recipe-display and all eleven slot-display types. Holder sets,
optional groups, shaped dimensions, item-stack component patches, recipe properties, and
stonecutter selections are validated. Slot traversal uses an explicit work stack: an adversarial
nest reaches the exact 512-depth refusal without consuming the process stack.

## Entry projection and ordering

`PlayEntryProjection` installs the client level exactly once and requires the source-observed
initial sequence:

1. login, difficulty, abilities, held slot, and complete recipe projection;
2. permission event and command tree;
3. recipe-book settings and replacement entries;
4. initial position;
5. zero or more server-data and player-info updates;
6. border, world clocks, default spawn, and terrain-load-start game event;
7. ticking state and ticking step.

The projection becomes `ReadyForTerrain` only after step 7. A level-dependent packet before login,
a duplicate login, or a core packet at the wrong stage is rejected without advancing the stage.
Optional scoreboard and join messages belong to separate families and may be interleaved by their
later integration owner.

Difficulty uses the source modulo mapping. Only hotbar indices `0..=8` replace the selected slot.
Permission events `24..=28` for the local entity map to tiers `0..=4`. Player-list action updates
apply only after an add action has created the entry, and secure-chat enforcement discards a
present remote chat session as observed by the client.

Position projection implements all relative position, rotation, motion, and rotate-delta flags,
then clamps pitch. A riding client retains its local state. Both riding and non-riding paths emit
one ordered action requiring teleport acknowledgement before the movement echo and reset block
prediction.

Server-data updates apply only when a server-list record exists. Icons that lack a PNG signature,
an `IHDR` header, or positive dimensions are omitted. A nonpositive border lerp duration becomes
an immediate size; a positive duration retains both endpoints.

## Conformance evidence

`crates/ferrite-protocol/tests/c1/play_clientbound_entry.rs` owns:

- all 16 locked empty/default packet goldens and fixed-width entity-event framing;
- every command payload form, graph validation, and unknown-type placeholder behavior;
- all 256 player-info action masks and field-presence validation;
- every recipe-display and slot-display dispatch, generated component-value delegation, and
  malformed recipe rejection;
- the complete entry-order trace, level lifecycle, player-list behavior, relative teleport
  projection, riding acknowledgement, server-data filtering, and terrain-ready boundary;
- duplicate registry references, unknown packet and registry IDs, non-finite float preservation,
  duplicate components, and iterative 512-depth refusal on both encode and decode.
