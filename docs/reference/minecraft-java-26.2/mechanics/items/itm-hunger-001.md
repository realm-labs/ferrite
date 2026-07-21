# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-HUNGER-001` — Exhaustion is spent before regeneration or starvation selects its timer branch

**Parent:** `ITM-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — food, saturation, exhaustion, timer order, regeneration and difficulty-specific
starvation floors are explicit in locked source.

**Applies when:**

A server player gains food/exhaustion or their `FoodData` ticks.

**Authoritative state:**

Food level, saturation, exhaustion, shared branch timer, health, hurt state, difficulty,
`naturalHealthRegeneration` gamerule and invulnerable ability.

**Transition and ordering:**

Eating adds nutrition clamped to `[0,20]`, then adds saturation clamped to `[0,newFoodLevel]`.
Before choosing the tick branch, exhaustion strictly above `4.0` loses exactly `4.0`; if saturation
is positive it loses `1.0` to a zero floor, otherwise food loses one to zero except on Peaceful.

Then choose exactly one branch. With natural regeneration, positive saturation, hurt player and
food `>=20`, increment the shared timer; at `>=10`, spend
`min(saturation,6.0)`, heal that value divided by `6.0`, add the same value as exhaustion, and zero
the timer. Otherwise, with natural regeneration, food `>=18` and hurt, increment it; at `>=80`, heal
`1.0`, add `6.0` exhaustion and zero it. Otherwise, with food `<=0`, increment it; at `>=80`, deal
`1.0` starvation damage only above health `10.0`, or at any health on Hard, or above `1.0` on
Normal, then zero it whether damage occurred or not. Any other state zeros the timer.

**Branches and aborts:**

Only one exhaustion quantum is processed per tick even above `8.0`. Saturation shields food loss.
Peaceful blocks exhaustion-driven food loss but not saturation spending. Ability invulnerability
prevents `Player.causeFoodExhaustion` from adding exhaustion; that method also adds only server-side.

**Constants and randomness:**

Initial values are food `20`, saturation `5.0`, exhaustion `0.0`, timer `0`. Exhaustion additions
cap at `40.0`. `hasEnoughFood` means food strictly greater than `6`; `needsFood` means below `20`.
There is no RNG in this state machine.

**Side effects:**

Food/saturation/exhaustion/timer mutation, healing or starvation damage. Save keys are `foodLevel`,
`foodTickTimer`, `foodSaturationLevel` and `foodExhaustionLevel` with the initial defaults above.

**Gates:**

Server player tick, difficulty, gamerule, hurt status, food/saturation thresholds and abilities.

**Boundary cases and quirks:**

Exhaustion exactly `4.0` is not spent. An exhaustion step can change food/saturation before branch
selection in the same tick. Switching between the two regeneration branches does not reset their
shared timer; entering an unmatched branch does. Starvation also resets the timer at its deadline
when health/difficulty prevents damage.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.world.food.FoodData#eat`,
`net.minecraft.world.food.FoodData#tick(net.minecraft.server.level.ServerPlayer)`,
`net.minecraft.world.food.FoodData#addExhaustion`,
`net.minecraft.world.entity.player.Player#causeFoodExhaustion`.

**Test vectors:**

Exhaustion `4.0`/next float; saturation `0`/sub-unit/`6+`; branch swap at timer `9`/`79`;
regeneration gamerule toggle; starvation at every difficulty and health floor; exhaustion cap;
Peaceful saturation and food behavior.
