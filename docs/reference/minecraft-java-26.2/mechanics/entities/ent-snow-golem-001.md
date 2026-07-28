# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-SNOW-GOLEM-001` — Snow Golems alternate a persisted pumpkin shell with ranged defence, melting and snow trails

**Parent:** `ENT-001`, `ENT-LIFECYCLE-001`, `ENT-002`,
`ENT-VEHICLE-001`, `ENT-004`, `ENT-PROJECTILE-001`, `ENT-005`,
`ENT-DAMAGE-001`, `ENT-BLOCK-001`, `ENT-DAMAGE-REDUCE-001`,
`ENT-KNOCKBACK-001`, `ENT-006`, `ENT-EFFECT-001`, `ENT-007`,
`ENT-DEATH-001`, `ENT-ENTITY-DROPS-001`, `MOB-001`, `MOB-AI-001`,
`MOB-002`, `MOB-SPAWN-001`, `MOB-003`, `MOB-DESPAWN-001`,
`ITM-001`, `ITM-DISPENSER-001`, `ITM-ADVANCEMENT-001`,
`BLK-CARVED-PUMPKIN-001`, `BLK-SNOW-FAMILY-001`,
`ENV-WEATHER-001`, `PLY-AUTOJUMP-001`, `WGEN-005`,
`WGEN-DIMENSION-001`, `WGEN-PORTAL-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked registration, the complete `SnowGolem`
implementation, effective golem/Mob behavior, pumpkin construction and
shearing callers, Snowball hit behavior, environment attributes, tags,
loot, Spawn Egg, compatibility and exact client projection close protocol
entity ID `121`.

**Applies when:**

`minecraft:snow_golem` is constructed from Snow Blocks and a Carved Pumpkin
or Jack o'Lantern, created by Egg, spawner, command or custom code, choosing
an Enemy target, firing or hitting with a Snowball, touching water or rain,
melting, placing Snow, being sheared by a player or dispenser, dying,
persisting its pumpkin state, synchronized, heard or rendered.

**Authoritative state:**

Protocol entity ID `121` constructs `SnowGolem` in the Peaceful-available
`MISC` category. Registration fixes dimensions `0.7x1.9`, eye height `1.7`,
client tracking range `8` and default update interval `3`. There is no
passenger or riding-offset override.

The attributes are maximum health `4`, movement speed
`0.20000000298023224` and inherited follow range `16`. No attack-damage
attribute is registered. `AbstractGolem` makes distance despawn unreachable,
and construction leaves the inherited XP reward at `0`. The ambient-sound
interval is `120`.

Inherited Entity, Living-Entity and Mob state occupies synchronized metadata
slots `0..15`. Snow Golem adds slot `16`, serializer ID `0` (`BYTE`), with
default byte `16`. Bit `0x10` is the pumpkin flag:

- `hasPumpkin` tests `(data & 16) != 0`;
- setting the pumpkin ORs `16`; and
- clearing it ANDs `-17`.

Every other bit is preserved. NBT key `Pumpkin` saves the Boolean flag.
Loading uses `getBooleanOr("Pumpkin", true)`, so an absent or wrongly typed
field restores the pumpkin. There is no timer or natural regrowth path.

Registration binds block-danger immunity to
`#minecraft:snow_golem_immune_to`, whose locked membership is exactly
Powder Snow. Direct entity-type tags separately include
`fall_damage_immune` and `freeze_immune_entity_types`. These three
admissions are independent: Powder Snow is not dangerous, fall damage is
rejected, and `canFreeze` is false.

The leash offset is `(0, 0.75*eyeHeight, width*0.4)`, exactly
`(0,1.275,0.28)`.

**Transition and ordering:**

### Complete goal and target graph

`registerGoals` installs four movement goals and one target selector:

| Selector | Priority | Goal and direct configuration |
|---|---:|---|
| goal | `1` | Ranged Attack, speed `1.25`, interval `20`, radius `10` |
| goal | `2` | Water-Avoiding Random Stroll, speed `1`, probability `1.0000001E-5` |
| goal | `3` | Look At Player, range `6` |
| goal | `4` | Random Look Around |
| target | `1` | nearest `Mob`, interval `10`, must see, need not reach, selector `instanceof Enemy` |

The superclass contributes no additional golem goals. There is no Hurt-By
target goal, retaliation graph, anger state or owner defence. An attacker
that is not an `Enemy` does not become a target through this subtype.
Conversely every nearby visible `Enemy` Mob is eligible even though the
locked Snowball implementation damages only a Blaze.

NoAI suppresses selector/goal execution through the generic Mob owner. It
does not suppress the subtype's environmental work in `aiStep`.

### Snowball construction, release and hit

When the ranged goal calls `performRangedAttack`, the server computes:

```text
dx = targetX - selfX
baseY = targetEyeY - 1.100000023841858
dz = targetZ - selfZ
lift = sqrt(dx*dx + dz*dz) * 0.20000000298023224
dy = baseY + lift - projectileY
```

It constructs one `Snowball` carrying a fresh default Snowball stack, raw
item ID `1044`, and spawns it through `Projectile.spawnProjectile` at speed
`1.6` and inaccuracy `12`. The insertion result is ignored.

After that attempt it unconditionally requests Snow Golem Shoot, protocol
sound ID `1573`, volume `1` and pitch
`0.4/(0.8+0.4*nextFloat)`. Its subtitle is the generic
`subtitles.entity.snowball.throw`, English `Snowball flies`. A failed
projectile insertion therefore still consumes the pitch draw and requests
the sound.

On an entity hit, Snowball attempts thrown-projectile damage `3` only when
the target is an exact Blaze instance; every other entity receives a
zero-damage attempt. It then follows the common hit path regardless of the
damage result. On any hit, the server broadcasts entity event `3` and
discards the projectile. Clients turn event `3` into eight item particles.
Projectile sweep, ownership, damage admission and discard retain
`ENT-PROJECTILE-001` and `ENT-DAMAGE-001`.

This creates an intentional selector/effect asymmetry: the Snow Golem
targets all `Enemy` Mobs, but ordinary Snowballs can remove health only
from Blaze under the locked hit code.

### Water, rain, heat and snow trail

The general Living-Entity tick runs before the Snow-Golem-specific
`aiStep`. `isSensitiveToWater` is true, so `isInWaterOrRain()` attempts
Drown damage `1` through the generic sensitive-to-water transaction. It can
repeat each tick subject to ordinary damage admission and invulnerability
time.

The subtype then continues only on `ServerLevel` and evaluates environment
attribute `minecraft:gameplay/snow_golem_melts` at the entity position.
When true, it attempts On-Fire damage `1` and ignores the Boolean result.
The locked attribute is true throughout the Nether dimension type and in
exactly seven of the 66 bundled biomes:

| Biome |
|---|
| Badlands |
| Desert |
| Eroded Badlands |
| Savanna |
| Savanna Plateau |
| Windswept Savanna |
| Wooded Badlands |

All other bundled biome values use the false baseline. The dimension
override wins independently of biome, so every Nether position melts a
Snow Golem.

The melt attempt occurs before the `mobGriefing` check. When that game rule
is false, the method returns after melting. When true, it evaluates four
foot-level positions in index order `0..3`:

```text
x = floor(selfX + (((i % 2) * 2 - 1) * 0.25))
y = floor(selfY)
z = floor(selfZ + ((((i / 2) % 2) * 2 - 1) * 0.25))
```

Integer division is used for `i/2`. For each position, the code requires
the live block to be Air and the default Snow layer to survive there. It
then calls `setBlockAndUpdate` with that Snow state and ignores the result,
followed unconditionally by `GameEvent.BLOCK_PLACE` at the candidate with
the Snow Golem as source and the Snow state as context.

The four coordinate expressions can collapse onto fewer unique block
positions depending on fractional entity coordinates. A later duplicate
sees the earlier successful Snow state and skips; a failed write can allow
a later duplicate to retry. No biome temperature or melt predicate gates
the trail. A living or lethally damaged Snow Golem can therefore attempt to
place Snow in a hot biome or the Nether during the same `aiStep`.

The ordering is exact:

1. superclass living/Mob work, including possible water-or-rain damage;
2. server-side melt-attribute evaluation and possible On-Fire damage;
3. `mobGriefing` admission; and
4. four Air/survival/write/event trail attempts.

Damage rejection does not abort later work. NoAI does not bypass steps
2..4.

### Player shearing

`readyForShearing` returns exactly `hasPumpkin`; it does not itself test
alive state. `mobInteract` returns `PASS` immediately unless the held item
is exact Shears and the pumpkin bit is set. The matching branch returns
`SUCCESS` on both logical sides. Only the server mutates:

1. call `shear(serverLevel, PLAYERS, shearsStack)`;
2. emit `GameEvent.SHEAR` with the player as source; and
3. damage the Shears by one in the used-hand equipment slot.

The public `shear` implementation does not repeat the readiness test. It:

1. plays Snow Golem Shear, protocol ID `1574`, at the entity in the passed
   sound source, volume/pitch `1/1`;
2. clears the pumpkin bit; and
3. evaluates built-in loot table `minecraft:shearing/snow_golem` with the
   tool, spawning every result at eye-height offset `1.7`.

That table has one unconditional roll of exactly one Carved Pumpkin, raw
item ID `385`. State clears before loot evaluation and entity insertion, so
failure cannot restore the pumpkin. The sound uses generic subtitle
`subtitles.item.shears.shear`, English `Shears click`.

### Dispenser shearing

The Shears dispenser delegate first handles a full beehive, then scans the
front-block AABB for nonspectating entities in encounter order. For each
candidate it first calls `shearOffAllLeashConnections(null)` and returns
success if any leash was removed. Only otherwise does the first alive,
ready `Shearable` receive:

1. `shear(level, BLOCKS, shearsStack)`;
2. `GameEvent.SHEAR` at the dispenser-front position with null source; and
3. one tool damage.

A leashed Snow Golem may therefore be unleashed without losing its pumpkin
on the first activation. Player and dispenser shearing differ in sound
source, event position/source and caller-side admission; both reuse the
same state-clear and loot transaction. Scheduling, selection and outer
events remain `ITM-DISPENSER-001`.

### Pumpkin construction

`BLK-CARVED-PUMPKIN-001` owns the complete fixed-order Snow/Iron/Copper
search and shared destructive spawn transaction. The Snow full pattern is:

```text
^
#
#
```

`^` accepts exact Carved Pumpkin or Jack o'Lantern at any facing; each `#`
requires exact Snow Block. The base admission pattern leaves the top cell
unconstrained and requires the two Snow Blocks.

On a full match, `EntityTypes.SNOW_GOLEM.create(level, TRIGGERED)` constructs
the entity without Mob finalization. A nonnull result uses matched cell
`(0,2,0)`, the bottom Snow Block, as the spawn coordinate. The shared
transaction clears all three cells to Air with flags `2`, emits level event
`2001` for each cached state, snaps the golem to
`(x+0.5,y+0.05,z+0.5)` at zero rotation, ignores `addFreshEntity` failure,
triggers `SUMMONED_ENTITY` for every server player within the entity box
inflated by `5`, and finally notifies neighbors at every cleared cell.

The default byte already supplies the pumpkin bit. A failed entity insertion
still consumes the structure and can trigger generic data-pack criteria.
A null entity factory clears nothing and allows the later Iron/Copper
pattern attempts. No locked vanilla advancement has a Snow-Golem-specific
criterion.

**Production and spawning:**

Spawn placement registers `ON_GROUND`, heightmap
`MOTION_BLOCKING_NO_LEAVES` and `Mob.checkMobSpawnRules`. Nevertheless all
66 bundled biome spawn lists contain zero Snow Golem rows. No Trial Spawner
configuration names it, and an exact scan of all 1,212 locked structure
templates finds zero literal Snow Golem entity payloads.

The only dedicated survival producer is pumpkin construction. Generic
spawners, commands, custom code and the Spawn Egg remain possible. Snow
Golem Spawn Egg is raw item ID `1198`, common stack size `64`, with
`entity_data.id=minecraft:snow_golem`.

The MISC category is allowed in Peaceful, has no natural creature cap path
for this zero-row identity, and the golem never distance despawns.

**Loot, XP and progression:**

Entity loot table `minecraft:entities/snow_golem`, random sequence of the
same key, has one unconditional pool producing Snowball item ID `1044` with
an integer-uniform count `0..15`. There is no Looting function, player-kill
condition or fire conversion. XP is `0`.

Shearing uses the separate one-Carved-Pumpkin table above and does not alter
the death table. Snow Golem is outside hostile-mob advancement selectors.
Pumpkin construction still fires the generic `SUMMONED_ENTITY` trigger for
nearby players, allowing data packs to observe it.

**Compatibility:**

Legacy compatibility first maps `SnowMan` to `minecraft:snowman`, then
renames `minecraft:snowman` to `minecraft:snow_golem`. Schemas V705 and
V1460 register the old `minecraft:snowman` identity; V1510 transfers it to
`minecraft:snow_golem`. The spawn-egg migration maps the entity identity to
`minecraft:snow_golem_spawn_egg`. Generic entity UUID and equipment fixes
retain their owners. The Boolean `Pumpkin` key remains the subtype's stable
save form.

**Client projection:**

`SnowGolemRenderer` uses `SnowGolemModel`, model layer
`minecraft:snow_golem`, shadow radius `0.5` and one
`SnowGolemHeadLayer`. Render-state extraction copies the authoritative
pumpkin flag: true supplies default Carved Pumpkin block state with a block
display context; false clears the head block.

The head layer submits nothing when the extracted head state is empty. It
also suppresses an ordinary head when the entity is invisible, but an
invisible glowing entity submits the outline render type. Otherwise it
follows the model head transform, translates Y by `-0.34375`, rotates Y
`180` degrees, scales `(0.625,-0.625,-0.625)`, translates
`(-0.5,-0.5,-0.5)` and submits the Carved Pumpkin block state.

The 64x64 model uses cube deformation `-0.5`: an `8x8x8` head,
`10x10x10` upper body, `12x12x12` lower body and two `12x2x2` arms.
Head yaw/pitch convert directly from render-state degrees to radians.
Upper-body yaw is `0.25*headYaw`; both arms rotate around it in opposite
directions at radius `5` using sine/cosine placement.

The entity texture is
`assets/minecraft/textures/entity/snow_golem/snow_golem.png`, 64x64,
477 bytes, SHA-256
`904567a77915ad2de4ded6a44e767667f4bd20c94126b6265c7de55500ad1eb5`.
The English entity and Egg labels are `Snow Golem` and
`Snow Golem Spawn Egg`.

Registered species sounds are Ambient `1570`, Death `1571`, Hurt `1572`,
Shoot `1573` and Shear `1574`. Ambient has no locked sounds-file mapping,
so the registered event is silent and subtitle-less. Death and Hurt use
`Snow Golem dies` and `Snow Golem hurts`; Shoot and Shear use the generic
subtitles given above.

**Constants and randomness:**

Entity/Egg IDs `121/1198`; dimensions/eye `0.7x1.9/1.7`; tracking/update
`8/3`; health/speed/follow/XP `4/0.20000000298023224/16/0`; pumpkin
slot/bit/default `16/0x10/16`; ranged speed/interval/radius/projectile
speed/inaccuracy `1.25/20/10/1.6/12`; aim offsets
`1.100000023841858/0.20000000298023224`; damage Blaze/other `3/0`;
water/melt damage `1/1`; trail offsets `0.25`; shear drop offset `1.7`;
death Snowballs `0..15`; shadow `0.5`.

Server randomness includes target/goal scheduling, random stroll, projectile
inaccuracy, Shoot pitch and death loot. Player/shears sound pitch is fixed.
Client randomness is limited to generic event-particle projection.

**Side effects:**

Targets and navigation; projectile entity construction, damage and discard;
sound, particle and game-event publication; water/rain and heat damage;
Snow writes and neighbor updates; pumpkin metadata/NBT mutation; Shears
durability and Carved-Pumpkin item entities; structure clearing and entity
insertion; loot, criteria and client model/head projection.

**Gates:**

Logical side, Peaceful and NoAI; target class and sight; projectile
insertion/hit/damage admission; exact Blaze; water/rain and damage cooldown;
reloadable dimension/biome environment attribute; `mobGriefing`, Air and
Snow survival; exact Shears, pumpkin bit and life check at dispenser caller;
leash-first dispenser order; exact block pattern and entity factory; zero
biome/template/Trial selectors; loot context; resources.

**Branches and aborts:**

Client `aiStep` skips all subtype environment work. NoAI stops goals but
does not abort the server melt/trail path. A false melt attribute skips only
heat damage; false `mobGriefing` returns before every trail probe. Each probe
independently skips on non-Air or failed Snow survival, while a failed write
does not skip its game event. Non-Shears or no-pumpkin interaction returns
`PASS`; the client Shears branch returns `SUCCESS` without mutation. A
dispenser leash removal returns before shearing. Null construction factory
falls through without clearing, whereas failed insertion occurs after all
clears and cannot abort criteria or neighbor updates.

**Boundary cases and quirks:**

The target selector is broader than Snowball damage. NoAI disables combat
but not melting or trails. Water/rain damage precedes heat damage, and
lethal damage does not explicitly abort the remaining `aiStep`. Melting
precedes `mobGriefing`; trail placement does not test temperature. Failed
Snow writes still emit block-place game events. The four trail probes can
duplicate positions.

The public shear method trusts its caller. It clears state before loot and
does not damage the tool or emit the shear game event itself. A player
wrapper and dispenser wrapper supply those effects differently. The
dispenser's leash removal wins before shearing.

Construction bypasses finalization and ignores insertion failure after
destroying the pattern. The synchronized pumpkin bit is persisted only
through its Boolean NBT projection; other byte bits are preserved in memory
but not given subtype persistence.

**Failure semantics:**

A failed projectile insertion does not suppress Shoot. Rejected Snowball
damage does not suppress hit event/discard. Rejected water, heat or lethal
damage does not roll back already completed superclass work and is ignored
by the subtype. Failed trail writes do not suppress their game event.
Failed shear loot entity insertion does not restore the pumpkin. Failed
construction insertion does not restore blocks or suppress nearby criteria.

**Client/server authority split:**

The server owns goals, targeting, projectiles, damage, environmental
attribute evaluation, game-rule admission, Snow placement, pumpkin mutation,
shearing, durability, loot, construction, persistence and progression.
Clients consume slot `16`, entity event `3`, movement and resources. They
render the pumpkin block only from the synchronized flag and cannot commit
any trail, loot or state transition.

**Observability:**

Observe registration/attributes and slot `16`; NBT default/type behavior and
other-bit preservation; the exact goal/selector graph; every Enemy and
Blaze/non-Blaze hit; projectile aim, insertion and sound; water/rain/melt
ordering across dimensions and all biome attribute values; `mobGriefing`,
four coordinates, duplicate/write/event behavior; player and dispenser
shearing including leash precedence; pattern clearing/insertion/criterion
order; zero production rows; both loot tables, XP, Egg and migrations; all
five sounds and exact model/head/texture states.

**Persistence and reload:**

Generic entity/Mob state and Boolean `Pumpkin` persist. Goals, targets,
projectile intent, trail candidates and client render state do not.
Environment attributes, block/entity tags and loot reload server-side;
later ticks use the published snapshot without reconstructing the golem.
Language, models, block models and texture reload client-side.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.entity.EntityTypes`;
`net.minecraft.world.entity.ai.attributes.DefaultAttributes`;
`net.minecraft.world.entity.SpawnPlacements`;
`net.minecraft.world.entity.Entity`;
`net.minecraft.world.entity.LivingEntity`;
`net.minecraft.world.entity.Mob`;
`net.minecraft.world.entity.animal.golem.AbstractGolem`;
`net.minecraft.world.entity.animal.golem.SnowGolem`;
`net.minecraft.world.entity.ai.goal.RangedAttackGoal`;
`net.minecraft.world.entity.ai.goal.WaterAvoidingRandomStrollGoal`;
`net.minecraft.world.entity.ai.goal.target.NearestAttackableTargetGoal`;
`net.minecraft.world.entity.projectile.Snowball`;
`net.minecraft.world.entity.projectile.Projectile`;
`net.minecraft.world.level.block.CarvedPumpkinBlock`;
`net.minecraft.world.level.block.DispenserBlock`;
`net.minecraft.core.dispenser.ShearsDispenseItemBehavior`;
`net.minecraft.world.entity.Shearable`;
`net.minecraft.world.level.storage.loot.BuiltInLootTables`;
`net.minecraft.util.datafix.fixes.EntityIdFix`;
`net.minecraft.util.datafix.fixes.EntityTheRenameningFix`;
`net.minecraft.util.datafix.fixes.EntityUUIDFix`;
`net.minecraft.util.datafix.fixes.ItemStackSpawnEggFix`;
`net.minecraft.util.datafix.schemas.V705`; `V1460`; `V1510`;
`net.minecraft.client.renderer.entity.EntityRenderers`;
`net.minecraft.client.renderer.entity.SnowGolemRenderer`;
`net.minecraft.client.renderer.entity.layers.SnowGolemHeadLayer`;
`net.minecraft.client.renderer.entity.state.SnowGolemRenderState`;
`net.minecraft.client.model.animal.golem.SnowGolemModel`;
`net.minecraft.client.model.geom.ModelLayers`;
`reports/registries.json#minecraft:{entity_type,item,sound_event,
loot_table,worldgen/biome,advancement}`;
`reports/minecraft/components/item/snow_golem_spawn_egg.json`;
`data/minecraft/tags/block/snow_golem_immune_to.json`;
`data/minecraft/tags/entity_type/{fall_damage_immune,
freeze_immune_entity_types}.json`;
`data/minecraft/loot_table/{entities/snow_golem,
shearing/snow_golem}.json`;
`data/minecraft/worldgen/biome/*.json`;
`data/minecraft/dimension_type/the_nether.json`;
`data/minecraft/environment_attribute/gameplay/snow_golem_melts.json`;
`data/minecraft/structure/**/*.nbt`;
`assets/minecraft/textures/entity/snow_golem/snow_golem.png`;
`assets/minecraft/sounds.json`; `assets/minecraft/lang/en_us.json`;
`ITM-DISPENSER-001`; `BLK-CARVED-PUMPKIN-001`;
`BLK-SNOW-FAMILY-001`; `ENV-WEATHER-001`; `WGEN-DIMENSION-001`;
`CLI-006`; `CLI-EFFECT-001`.

**Test vectors:**

Run `EXP-ENT-037` across raw/constructed/loaded state and all pumpkin-bit
values; complete goals and every Enemy/Blaze hit branch; projectile
insertion, aim and sound; water/rain/heat/NoAI/Peaceful/mobGriefing
ordering; all four trail coordinates and duplicate/write outcomes; player
and dispenser shearing with leash precedence; construction/factory/write/
insertion/criterion failures; zero biome/Trial/template production; both
loot tables, tags/Egg/compatibility, all sounds and exact model/head/texture
projection.

**Limits:**

Generic lifecycle, metadata, goal scheduling, navigation, projectile
sweep, damage/death, invulnerability, block writes, game events, loot,
advancements, Spawn-Egg interaction, dispenser scheduling and renderer
submission retain their cited owners. This leaf owns Snow-Golem selectors,
overrides, constants and their exact composition.
