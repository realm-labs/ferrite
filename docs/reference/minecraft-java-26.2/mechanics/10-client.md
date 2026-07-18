# Client-observable leaf rules

## Leaf rule `CLI-PREDICT-001` — Client prediction is provisional and server correction is authoritative

**Parent:** `CLI-001`, `CLI-002`, `CLI-003`, `CLI-004`  
**Applies when:** The client locally predicts movement, interaction, block state, inventory, or use animation before receiving the server result.  
**Authoritative state:** Server world/player/inventory state, client predicted state, sequence/state identifiers, pending teleport/correction and acknowledgement state.  
**Transition and ordering:** Client samples input, performs allowed local prediction and sends request with relevant ordering token; server processes against its current state; on acceptance normal authoritative updates converge the client; on rejection or divergence server sends state/position/menu correction; client applies correction and acknowledges where required before later requests can be trusted.  
**Branches and aborts:** Prediction disabled for operation; accepted exactly; accepted with different result; rejected permission/reach/collision/state; stale sequence/menu state; correction arrives after later predictions; teleport acknowledgement outstanding.  
**Constants and randomness:** Network delay/reordering is not gameplay RNG. Position and inventory comparisons use operation-specific numeric/state identifiers. Do not use render interpolation as authoritative simulation.  
**Side effects:** Temporary local block/pose/slot/swing, request packet, authoritative block/entity/slot/position updates, rollback/resync, acknowledgement and possibly replay of later local input. Sounds/particles may be locally predicted only for explicitly client-originated paths.  
**Gates:** Operation prediction support, connection state, sequence/state ID, pending correction, game mode, server validation and client screen/state.  
**Boundary cases and quirks:** A visible local success can be rolled back. Corrections must not duplicate item consumption or effects. Ordering tokens prevent an old rejection from overwriting newer authoritative state.  
**Evidence:** `Confirmed` authority model; reordering matrix `Implementation blocker`; `OFF-CLIENT-001`, `OFF-SERVER-001`; `EXP-CLI-001`.  
**Test vectors:** Inject latency/reorder for placement, breaking, movement and menu click; server mutates target before request; two predictions before first correction; assert final state and one-time side effects.

## Leaf rule `CLI-UI-001` — Screen gestures translate to menu click operations; the server owns results

**Parent:** `CLI-005`, `ITM-002`  
**Applies when:** A container screen converts mouse/keyboard/touch-like gestures into inventory actions.  
**Authoritative state:** Client screen drag/double-click state and predicted slots; server menu ID/state ID, slots, carried stack and click algorithm.  
**Transition and ordering:** Client maps the gesture to the 26.2 `ContainerInput` semantic operation plus slot/button arguments, predicts changed slots/carried stack, sends it with menu and state identifiers; server invokes `ITM-CONTAINER-001`; accepted deltas or full resync update client; closing sends/removes menu and resolves carried stack. Recipe book, anvil text, beacon choice and similar controls send their dedicated semantic action before menu recomputation.  
**Branches and aborts:** Outside click, touchscreen mode, quick-craft drag phases, double click pickup-all, hotbar swap, clone, throw, stale menu, screen closes during gesture or dedicated control invalid.  
**Constants and randomness:** Slot coordinates are presentation only; semantic slot indices and click enums are gameplay input. Double-click/drag timing is client UI state and must match only where it changes emitted operations. No RNG.  
**Side effects:** Client prediction, click/dedicated request, server slot/container mutations, sounds for UI/device results, resynchronization and carried-stack return/drop on close.  
**Gates:** Screen/menu type, slot hover/index, mouse button/modifiers, touchscreen, player game mode, server state ID and slot policies.  
**Boundary cases and quirks:** Quick-craft is a multi-stage gesture but server validation applies to the complete semantic sequence. Client visual slot coordinates may differ with resource packs without changing operations.  
**Evidence:** `Confirmed`; `OFF-CLIENT-001`, `OFF-SERVER-001`; menu registry mapping; gesture table `EXP-CLI-002`.  
**Test vectors:** Every click type from mouse and keyboard; drag interrupted by close; double-click with matching stacks across containers; stale state; anvil rename concurrent with output take; creative clone in noncreative mode.

## Leaf rule `CLI-EFFECT-001` — Sounds, particles, entity events, and level events are causal observable outputs

**Parent:** `CLI-006`  
**Applies when:** Server or explicitly local client gameplay emits an audible/visual event.  
**Authoritative state:** Event ID/type, position/source entity, audience/dimension, sound category/volume/pitch/seed, particle parameters/count/velocity and client option/range state.  
**Transition and ordering:** Gameplay commits its authoritative transition, then invokes the event API at the specified branch; server selects tracking/audience and sends semantic event; client resolves registry/type and instantiates sound/particles/entity animation. Purely local feedback is emitted only on paths designated client-side and must avoid duplicating the later server event.  
**Branches and aborts:** Excluded initiating player; different dimension; outside tracking/range; particle setting/count suppression; muted category; unknown/removed entity; local prediction later rejected; event maps to block/entity-specific visualization.  
**Constants and randomness:** Volume affects audible distance by client rules; pitch and particle distributions may consume server or client RNG depending on event API. A supplied sound seed must be preserved. Cosmetic client RNG need not match unless it changes observable gameplay timing/count required by a rule.  
**Side effects:** Audible instance, particles, entity animation, subtitles, vibration/game-event listeners only when a separate server game event is emitted. A sound is not itself a game event.  
**Gates:** Commit branch, side (server/local client), audience/tracking, dimension, options, sound category and event-specific exclusion.  
**Boundary cases and quirks:** State update, sound, particle and game event are separate side effects; one does not imply the others. Dedicated servers never synthesize client-only presentation.  
**Evidence:** `Confirmed` separation; exact audience/range per overload `Cross-checked`; `OFF-SERVER-001`, `OFF-CLIENT-001`; `EXP-CLI-003`.  
**Test vectors:** Initiator-excluded placement sound, two observers at range boundary, dimension mismatch, muted/particle-minimal clients, predicted event rejected, compare event counts and seeds under latency.
