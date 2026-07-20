# Blocks mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `BLK-BREAK-HOOK-001` — Concrete block break hooks and loot remain content-owned

**Parent:** `BLK-002`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — exhaustive declaration scanning and live locked-registry resolution find exactly
110 block IDs whose effective `attack`, `playerWillDestroy`, `destroy`, `playerDestroy` or
`spawnAfterBreak` owner is not the base implementation. The 23 resolved owner combinations, every
helper transaction, state/component/occupant branch, locked loot projection, rule/attribute gate,
constant, write result and RNG site are specified below. The other 1,086 block IDs use only
`BLK-BREAK-001`; `EXP-BLK-005` is a regression oracle, not a source gap.

**Applies when:**

A player starts or completes a break on one of those 110 IDs, or a state replacement invokes the
`destroy` hook on `moving_piston`. The hook positions are exactly those in `BLK-BREAK-001`: `attack`
at admitted start, `playerWillDestroy` before generic removal, `destroy` during old-state
replacement, `playerDestroy` only on the survival/correct-tool/`block_drops` branch, and
`spawnAfterBreak(...,true)` after that branch's loot stacks.

**Authoritative state:**

The original and callback-returned block state, live target/counterpart states, captured pre-removal
block entity, player mode/position/main-hand item, exact item/enchantment tags, `block_drops`,
`tnt_explodes`, positional `gameplay/water_evaporates`, locked block loot data, gameplay and
per-table loot sequences, entity/state admission results and nested update effects. All four
prevention enchantment tags in this leaf contain only `silk_touch`; the decorated-pot breaker tag
expands all swords, axes, pickaxes, shovels and hoes plus trident and mace.

**Registry completeness:**

The non-base owner map comprises dragon egg; note block; both redstone ores; moving piston; bee
nest/beehive; ten double plants (`tall_seagrass`, six tall flowers/grass/fern, `pitcher_crop`,
`pitcher_plant`, `small_dripleaf`); ice/frosted ice; turtle egg; fire/soul fire; all 16 beds;
creaking heart; decorated pot; all 21 doors; all 17 shulker boxes; TNT; tripwire; piston head; 17
`DropExperienceBlock` IDs; seven infested blocks; sculk catalyst; both sculk sensors; sculk
shrieker; and spawner. Nine block-entity types participate directly: beehive, creaking heart,
decorated pot, shulker box, piston, mob spawner, sculk catalyst, sculk sensor and sculk shrieker.

**Transition and ordering:**

Execute only the effective registered owner's override at its generic position, then its explicit
base delegation where stated. Pre-removal hooks may write live state or emit entities before generic
removal; the returned state, not an automatic reread, feeds removal/loot. Post-removal hooks receive
the retained pre-removal BE/tool inputs after generic loot ordering. The family transactions below
are exhaustive and preserve ignored-result/no-rollback behavior.

- **Attack hooks:** Dragon egg tries at most 1,000 candidates. Each consumes six gameplay `nextInt`
  calls for offsets `nextInt(16)-nextInt(16)`, `nextInt(8)-nextInt(8)`, `nextInt(16)-nextInt(16)`;
  accept only air with nonair below, inside the world border and build height. Server acceptance
  writes the original state at the target with flags 2 and then removes the origin without drops,
  ignoring both results and returning even if either failed. Client acceptance instead emits 128
  portal particles; each consumes one interpolation double, three velocity floats and three
  position-jitter doubles, then returns. Note-block attack is server-only: if its instrument works
  above note blocks or the above block is air, enqueue block event `(0,0)` then emit
  `NOTE_BLOCK_PLAY`; regardless of that obstruction result, award `PLAY_NOTEBLOCK`. Redstone-ore
  attack visits the six `Direction.values()` faces, skips solid-render neighbors, consumes two
  floats per exposed face for the non-axis coordinates, emits one redstone particle at face offset
  `0.5625`, then writes `lit=true` with flags 3 only if previously false.
- **Pre-removal counterpart/state hooks:** Fire/soul fire emit server level event 1009 before base
  destroy particles/event. Creative breaking a bed foot removes a matching head with flags 35 and
  emits event 2001; a head relies on ordinary shape cleanup. Every door invokes the double-plant
  lower-half cleanup when creative or when the tool is incorrect: breaking a matching upper removes
  the lower to its exact source-water fluid state or air with flags 35 and event 2001. Double plants
  do the same only for creative upper breaks; survival instead runs their current-state loot
  transaction immediately in `playerWillDestroy`, before the generic `block_drops` gate, and later
  `playerDestroy` deliberately delegates with AIR so the loot is not repeated. Piston-head creative
  break destroys without drops the matching extended piston/sticky-piston base behind it only when
  type, facing and extension agree. Replacing a moving piston removes without drops an extended
  piston base opposite its facing, without checking base facing/type.
- **Other pre-removal hooks:** Shears on tripwire write `disarmed=true` with flags 260 and emit
  `SHEAR`, but the hook still returns the original state to the generic transaction. A decorated-pot
  breaker without silk touch writes `cracked=true` with flags 260 and passes that cracked state to
  base; its locked loot table consequently selects dynamic `minecraft:sherds`, while an uncracked
  pot returns one pot carrying only `pot_decorations`. Survival breaking `unstable=true` TNT calls
  the `tnt_explodes`-gated primer before removal; success creates centered `PrimedTnt` with null
  owner, ignores admission failure, then plays `TNT_PRIMED` at volume/pitch 1 and emits
  `PRIME_FUSE`. A creaking heart first asks its BE to remove its protector with a player-attack
  damage source: a resolved protector runs death effects, enters tearing-down, receives health 0,
  and the heart clears its protector record. A noncreative, nonspectator break of `natural=true`
  then samples inclusive 20–24 XP before the `block_drops` check inside `popExperience`.
- **Creative content preservation:** On a server creative break, bee nest/beehive manually emits one
  item at integer block coordinates with default pickup delay only when `block_drops` is true and
  either occupants exist or honey is positive. It applies every collected BE component plus
  `BLOCK_STATE.honey_level`; entity admission is ignored. A server creative shulker break first
  calls `isEmpty`, which materializes a pending loot table with a null player; a resulting nonempty
  inventory emits one centered component-bearing item without consulting `block_drops`, while an
  empty result falls through to `unpackLootTable(player)` (then idempotent). Client and noncreative
  branches call `unpackLootTable(player)` directly. The locked shulker table copies `custom_name`,
  `container`, `lock` and `container_loot`; no hook duplicates a generic creative drop.
- **Beehive survival post-hook:** Base `playerDestroy` (stats, exhaustion, loot and its after-break
  callback) runs first. Without silk touch, release every stored occupant with `EMERGENCY`, which
  bypasses `bees_stay_in_hive` and blocked-exit rejection. Each created bee may consume one float
  for a 0.9 saved-flower copy; it is positioned at the face or center when blocked, then exit
  sound/event and entity admission occur. Only successful admission removes stored data, but the
  created entity enters the returned list before admission. Returned bees within squared distance 16
  target the breaker unless campfire smoke sedates the hive, in which case their stay-out countdown
  becomes 400. Then container neighbors update and every untargeted bee inside an AABB inflated
  `(8,6,8)` selects a uniformly random nearby player, one bounded draw per bee. Silk touch skips
  release, neighbor update and anger. Finally `BEE_NEST_DESTROYED` receives the post-release
  retained occupant count in either branch.
- **Post-removal restoration hooks:** Ice/frosted ice first complete base loot. Without silk touch,
  `water_evaporates=true` removes the already-restored position and returns; otherwise a
  motion-blocking or liquid block below causes a water source write via `setBlockAndUpdate`, while
  unsupported dry space stays air. Turtle egg first completes base loot, consumes one float for
  break sound pitch `0.9+0.2f` at volume `0.7`, then restores `eggs-1` with flags 2 plus
  `BLOCK_DESTROY`/2001 when the original count exceeds one; count one calls
  `destroyBlock(...,false)`. Because `playerDestroy` itself is generic-gated, disabling
  `block_drops` removes the entire egg stack and bypasses this decrement.
- **Experience/infestation after-hooks:** `DropExperienceBlock` samples its provider, applies the
  locked `block_experience` enchantment effect (only silk touch supplies one, setting zero), and
  emits positive XP through the `block_drops`-gated orb path. Exact providers are: 0 for both gold,
  iron and copper ores; uniform 0–2 for both coal ores; 0–1 nether gold; 2–5 both lapis and nether
  quartz; 3–7 both diamond and emerald; and constant 1 sculk. Both redstone ores use 1–5; sculk
  catalyst, both sensor IDs and shrieker use
  5. Spawner uses `15+nextInt(15)+nextInt(15)` (15–43 triangular). These hooks first delegate to
     base and only run when their boolean is true; player breaking supplies true only inside the
     generic gated branch. Each infested block instead delegates to base, then, when `block_drops`
     is true and silk touch absent, creates one triggered silverfish centered horizontally at
     block-bottom Y with zero rotations, ignores admission failure, and calls its spawn animation
     even after rejection.

**Branches and aborts:**

Base-only owner; attack rejected before dispatch; teleport exhaustion/no candidate; note
obstruction; covered redstone face; client/server split; creative/survival/wrong-tool counterpart
branch; mismatched half/base; state write failure; empty/pending-loot BE; `block_drops`; silk touch;
occupant/entity creation/admission failure; sedation/distance/existing bee target/no players;
unsupported ice; `water_evaporates`; egg count; `tnt_explodes`; nonnatural/creative/spectator heart;
false after-break boolean; zero/enchantment-zeroed XP. Writes and entity admissions whose boolean is
ignored never roll back earlier side effects.

**Constants and randomness:**

Dragon attempts 1,000 and client particles 128 with draws above; redstone face offset `0.5625`;
flags 2/3/35/260; events 1009/2001; hive target squared range 16, neighborhood `(8,6,8)`, flower
probability 0.9 and sedation countdown 400; turtle volume 0.7/pitch `0.9+0.2f`; heart 20–24;
redstone 1–5; sculk family 5 except sculk 1; spawner 15 plus two bounds 15. Uniform XP providers
consume one bounded gameplay draw; constant providers consume none. Loot-table RNG and stack
splitting begin at the base loot position and are specified by `ITM-LOOT-001`, while this leaf fixes
the state, BE, tool and order passed to them.

**Side effects:**

Counterpart/live-state writes, base and subtype events, component-bearing item entities, occupant
releases/targets, protector death, TNT/silverfish/orb entities, stats, criteria, sounds/particles,
nested neighbor work and loot-context changes. Exact downstream entity lifecycle and note-block
client rendering remain owned by their entity/client leaves.

**Gates:**

Effective registered class, state property/half/facing, server/client and player mode, correct tool,
main-hand item/enchantment tags, `block_drops`, `tnt_explodes`, `water_evaporates`, BE
presence/content, support/collision/border/build bounds, entity admission and callback boolean.
Difficulty is not read by these hooks.

**Boundary cases and quirks:**

Double plants can drop while `block_drops=false` because their pre-hook evaluates loot before the
generic gate. Tripwire returns the pre-disarm state, while decorated pot returns the cracked state.
Creative shulkers materialize pending loot with a null player and ignore `block_drops`; creative
hives do not. Hive criteria observe remaining occupants after release. A rejected hive or
infestation entity is not rolled back, and an infestation still animates. Dragon-egg target-write
failure still removes the origin. Unsupported ice remains air, and disabling drops bypasses both ice
water restoration and stacked-egg decrement.

**Evidence:**

`Confirmed`; `OFF-SERVER-001`; `OFF-REPORT-001`; exhaustive `javap -p -s -c` declaration scan plus
locked runtime registry resolution. Anchors:
`net.minecraft.world.level.block.BaseFireBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.BedBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
both break hooks in `net.minecraft.world.level.block.BeehiveBlock`,
`net.minecraft.world.level.block.CreakingHeartBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.DecoratedPotBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.DoorBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
both break hooks in `net.minecraft.world.level.block.DoublePlantBlock`,
`net.minecraft.world.level.block.DragonEggBlock#attack(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.DropExperienceBlock#spawnAfterBreak(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.item.ItemStack,boolean)`,
`net.minecraft.world.level.block.IceBlock#playerDestroy(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BlockEntity,net.minecraft.world.item.ItemStack)`,
`net.minecraft.world.level.block.InfestedBlock#spawnAfterBreak(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.item.ItemStack,boolean)`,
`net.minecraft.world.level.block.NoteBlock#attack(net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.entity.player.Player)`,
both hooks in `net.minecraft.world.level.block.RedStoneOreBlock`, the four sculk after-hook classes,
`net.minecraft.world.level.block.ShulkerBoxBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.SpawnerBlock#spawnAfterBreak(net.minecraft.world.level.block.state.BlockState,net.minecraft.server.level.ServerLevel,net.minecraft.core.BlockPos,net.minecraft.world.item.ItemStack,boolean)`,
`net.minecraft.world.level.block.TntBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.TripWireBlock#playerWillDestroy(net.minecraft.world.level.Level,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.entity.player.Player)`,
`net.minecraft.world.level.block.TurtleEggBlock#playerDestroy(net.minecraft.world.level.Level,net.minecraft.world.entity.player.Player,net.minecraft.core.BlockPos,net.minecraft.world.level.block.state.BlockState,net.minecraft.world.level.block.entity.BlockEntity,net.minecraft.world.item.ItemStack)`,
and both piston hook classes. Locked block loot/tag/enchantment data fixes the stated projections.

**Test vectors:**

Exhaustive 110-ID owner-map equality; every state/half/facing/tool/rule/attribute gate; base-only
negative controls; failed callback/generic/counterpart writes; creative and survival
empty/populated/pending-loot hives/shulkers; admitted/rejected/sedated occupants and nearby-target
cardinalities; pot tag/silk/cracked loot inputs; ice support/evaporation; egg counts 1–4 with drops
disabled; all XP providers/silk and exact RNG bounds; teleport attempts 1/1,000 with failed writes;
all redstone exposed-face masks; TNT/infestation admission failure. `EXP-BLK-005` is the conformance
matrix.
