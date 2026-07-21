# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-XP-001` — Player experience normalizes progress across piecewise level costs

**Parent:** `ITM-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — point addition/removal, progress normalization, saturated level arithmetic,
enchantment deductions, level-up sound and death reward are explicit in locked source.

**Applies when:**

Points or levels are added/removed, enchanting spends levels, or player death computes XP reward.

**Authoritative state:**

Level, progress fraction, total experience, score, last level-up sound tick, player tick, enchantment
seed, abilities/spectator state and `keepInventory` gamerule.

**Transition and ordering:**

Required points for level `L` are `7+2L` below `15`, `37+5(L-15)` for `15..29`, and
`112+9(L-30)` at `30+`. Giving `i` points first increases score, adds
`i/currentRequirement` to progress and clamps total `total+i` to `[0,Integer.MAX_VALUE]`.

While progress is negative, convert it to a signed point remainder using the current requirement.
If level is positive, subtract one level and set progress to
`1 + remainder/newRequirement`; at level zero, still call level subtraction, then force progress
zero. While progress is at least one, convert the excess fraction back to points with the old
requirement, add one level, then divide by the new requirement. These loops run until
`0 <= progress < 1`.

Adding levels uses saturated integer addition. A negative result resets level, progress and total to
zero. A positive addition that lands on a level divisible by five plays the level-up sound only if
more than `100` player ticks elapsed since the preceding sound; volume is
`0.75 * min(level/30,1)` and pitch is `1.0`.

Enchanting subtracts the requested cost from level only; underflow resets all three XP fields, then
always replaces the enchantment seed with `player.random.nextInt()`. Player death XP is zero for
spectators or when `keepInventory` is true, otherwise `min(level*7,100)`.

**Branches and aborts:**

Point removal at level zero clamps progress/total rather than retaining debt. Direct level changes
do not recompute total on non-underflow. Sound requires a positive requested amount, exact resulting
multiple of five and strict tick separation.

**Constants and randomness:**

Breakpoints are levels `15` and `30`; death cap `100`; sound interval comparison is strict
`lastSoundTick < tickCount-100`. Only seed refresh consumes RNG. XP save keys are `XpP`, `XpLevel`,
`XpTotal`, `XpSeed`; a loaded zero seed is immediately replaced by a random int.

**Side effects:**

Score, level/progress/total, level-up sound/timestamp and enchantment seed.

**Gates:**

Call site, level/progress boundary, positive level delta for sound, tick interval, spectator and
`keepInventory`.

**Boundary cases and quirks:**

Crossing a breakpoint recalculates the remainder with the destination level's requirement.
`onEnchantmentPerformed` does not subtract total experience when level remains nonnegative. Direct
negative level changes that remain above zero likewise leave progress and total untouched.

**Evidence:**

`OFF-SERVER-001`;
`net.minecraft.world.entity.player.Player#giveExperiencePoints`,
`net.minecraft.world.entity.player.Player#giveExperienceLevels`,
`net.minecraft.world.entity.player.Player#getXpNeededForNextLevel`,
`net.minecraft.world.entity.player.Player#onEnchantmentPerformed`,
`net.minecraft.world.entity.player.Player#getBaseExperienceReward`,
`net.minecraft.world.entity.player.Player#readAdditionalSaveData`.

**Test vectors:**

Point crossings at levels `14/15/29/30`; multi-level positive and negative points; level-zero
underflow; total integer saturation; direct negative levels with/without underflow; sound at exactly
`100`/`101` ticks and levels `5/30/35`; enchanting underflow and seed refresh; death gates/cap.
