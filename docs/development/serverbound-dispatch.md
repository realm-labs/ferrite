# Serverbound application dispatch

## Boundary

`player::dispatch` owns the exhaustive production disposition of every decoded 26.2
`PlayServerboundEntryPacket`. The protocol driver consumes connection-owned teleport
acknowledgement and keep-alive packets before application dispatch. Every remaining packet reaches
`JavaPlayerConnection::dispatch_serverbound`, which determines its responsibility before invoking
gameplay code.

The disposition vocabulary is closed:

| Disposition | Meaning |
|---|---|
| `Handled` | The packet was consumed by its current protocol or application responsibility. |
| `Rejected` | Validation selected an explicit terminal rejection, currently invalid movement or flying. |
| `Gated` | A named transient gate prevented mutation, currently client loading or an active Region transfer. |
| `Unsupported` | The packet belongs to a future Goal and no authority mutation or success update was produced. |

The route match names all 48 required packet variants. Adding a protocol variant without adding an
application disposition is a compile error. The production manifest independently verifies that
the same packet enum is partitioned exactly once among responsibility rows.

## Formal observability

`PlayerDispatchReport` contains the disposition and the optional handled update. Unsupported
packets return no update. `NetworkSession` retains one latest outcome, and the formal gateway
retains the latest outcome observed in stable session polling order. Operators and tests can query
it through `NodeProcess::last_serverbound_dispatch`; the slot is deliberately bounded and does not
become an unbounded packet audit log.

Future Goals replace `Unsupported` routes with typed authority calls. They must preserve the same
explicit result boundary and may not route a packet to `Handled` merely because its codec exists.
