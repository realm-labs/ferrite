# Entities mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ENT-DEATH-001` — Death protection, death entry, drops and timed removal form one transaction

**Parent:** `ENT-007`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the locked server/client control flow fixes death-protection selection and
effects, ordinary and player death entry, killer callbacks, loot/equipment/XP contexts, every
death/drop override, death-event presentation and ordinary, Creaking and Ender Dragon removal
timelines. Loot-table pool evaluation remains independently owned by `ITM-LOOT-001`; this rule fixes
the context, call position and emitted stacks without copying those data tables.

**Applies when:**

The accepted server damage path in `ENT-DAMAGE-001` has committed health at or below zero and calls
the lethal branch. It also applies to an explicit subtype death entry or to subsequent
`LivingEntity` ticks while `health<=0 || dead`.

**Authoritative state:**

Health and `dead`; source/direct/causing entity and combat kill credit; main/off-hand stacks and
`death_protection`; active effects; sleep/use state; death/death-message timers and pose;
recent-player memory and last player; loot table/seed, equipment/drop chances, carried inventories
and XP reward; game rules, difficulty, spectator/team state and removal reason.

**Transition and ordering:**

Resolve the protection transaction first. If it returns false, resolve exactly one applicable
ordinary/player/subtype death entry, its synchronous score/drop/event effects, and then the
applicable tick-driven removal timeline as detailed below.

**Transition and ordering — protection:**

1. Before calling `die`, the damage body calls `checkTotemDeathProtection(source)`. A source in
   `bypasses_invulnerability` returns false immediately and does not inspect or consume either hand.
   In 26.2 that tag contains exactly `out_of_world` and `generic_kill`.
2. Otherwise inspect `MAIN_HAND` and then `OFF_HAND`. Select the first stack whose
   `death_protection` component is non-null, copy the entire pre-consumption stack, shrink the held
   stack by exactly one, and stop scanning. No component means false with no mutation. The locked
   item report has exactly one default item with this component: `totem_of_undying`, whose maximum
   stack size is one.
3. For a `ServerPlayer` only, award `Stats.ITEM_USED` for the copied stack's item, trigger
   `USED_TOTEM` with that copied stack, then cause `ITEM_INTERACT_FINISH` vibration through the
   copied stack. These precede health restoration and effects; nonplayers perform none of the three.
4. Set health to exactly float `1.0f`. Iterate component `death_effects` in stored order and ignore
   each effect's boolean result, then broadcast entity event byte `35` and return true. The 26.2
   totem first removes every active effect, then consumes exactly one victim `nextFloat` for an
   apply-effects probability of `1.0f` (the comparison still consumes RNG), and adds fresh copies in
   order: Regeneration amplifier 1 for 900 ticks, Absorption amplifier 1 for 100 ticks, and Fire
   Resistance amplifier 0 for 800 ticks. Effect merge/callback semantics are `ENT-EFFECT-001`.
5. A client receiving event `35` attaches a `totem_of_undying` tracking emitter for 30 ticks, plays
   `item.totem.use` locally at the entity with volume/pitch `1`, and, only when the entity is the
   local player, displays item activation. The displayed stack is the first current local hand with
   `death_protection`, main before off; if neither still has one, it displays a newly constructed
   ordinary totem stack. The event therefore does not transmit the copied server stack itself.

**Transition and ordering — ordinary living death:**

1. `LivingEntity#die` returns before any work when already removed or `dead==true`. Otherwise
   snapshot `causing=source.getEntity()` and `killCredit=getKillCredit()`. If kill credit exists,
   call its `awardKillScore(victim,source)` first. A server-player killer rejects self-kill credit,
   otherwise triggers the killed-player criterion when the victim is a server player, increments
   all-kill scoreboard objectives, player- or mob-kill stat/objectives, team-color kill objectives,
   and `PLAYER_KILLED_ENTITY`, in that order.
2. Stop sleeping when necessary, stop using the current item, and (server side only, if
   custom-named) write the death log. Call `handleKillingBlow`; the common implementation sets
   `dead=true`. Then recheck/close combat tracking.
3. On `ServerLevel`, invoke `causing.killedEntity(level,victim,source)` when causing is non-null.
   The default returns true. A player first awards the type-specific `ENTITY_KILLED` stat and
   returns true. A powered creeper, only while its own loot gate passes and it has not previously
   produced a skull, evaluates `loot_table/entities/charged_creeper` against the victim and sets
   `droppedSkulls=true` after each emitted stack, then returns its superclass result.
4. A zombie callback first retains the default true result. Only for a villager victim on Normal or
   Hard does conversion participate: Normal consumes one zombie `nextBoolean` and skips conversion
   when true; Hard always attempts it. Successful villager-to-zombie-villager conversion changes the
   callback result to false. A false callback suppresses the victim's `ENTITY_DIE` game event, all
   death loot/XP/equipment and wither rose, but does not suppress the following event byte `3` or
   `DYING` pose.
5. When `causing` is null or its callback returned true, emit `ENTITY_DIE`, execute the complete
   drop transaction below, then attempt a wither rose from `killCredit`. A Wither kill credit tries
   the victim's block position: when `mob_griefing` is true, the current state is air and the
   default wither rose can survive, call `setBlock(...,3)` and treat the rose as placed without
   consulting that call's boolean return. Otherwise spawn one wither-rose item exactly at victim
   `(x,y,z)` without the common pickup-delay helper. Non-Wither kill credit does nothing.
6. Regardless of the callback result, broadcast byte `3`; then set pose `DYING`. On clients, byte
   `3` plays the entity death sound at normal volume and pitch `1+(nextFloat-nextFloat)*0.2`. For
   nonplayer client entities it also sets health to zero and locally enters `die(generic)`; player
   entities only play the sound in this handler.

**Transition and ordering — loot, equipment and XP:**

1. At entry compute `killedByPlayer = lastHurtByPlayerMemoryTime>0`. The common loot gate is adult
   and `mob_drops`; `Monster` removes the adult test, so baby monsters may use their loot table.
   When the gate passes, run the entity loot table and then `dropCustomDeathLoot`; regardless of
   that gate, run `dropEquipment` and then `dropExperience`.
2. Entity-table evaluation uses `THIS_ENTITY=victim`, `ORIGIN=victim.position`,
   `DAMAGE_SOURCE=source`, optional `ATTACKING_ENTITY=source.entity` and optional
   `DIRECT_ATTACKING_ENTITY=source.directEntity`. Only when `killedByPlayer` and the remembered
   player reference is non-null does it also supply `LAST_DAMAGE_PLAYER` and that player's float
   luck. It builds the `ENTITY` parameter set, evaluates the selected locked table with
   `getLootTableSeed()` (common seed `0`), and passes every resulting stack to `spawnAtLocation` in
   generated order.
3. Common `spawnAtLocation(stack)` rejects an empty stack; otherwise it constructs an item entity at
   exact victim `(x,y,z)`, sets pickup delay `10`, and adds it. Construction consumes two
   item-entity `nextFloat` calls for bob/yaw followed by two item-entity `nextDouble` calls for
   velocity `((d0*0.2)-0.1, 0.2, (d1*0.2)-0.1)`. It consumes no victim RNG. The wither rose's direct
   constructor has the same four construction draws but pickup delay remains zero.
4. `Mob#dropCustomDeathLoot` traverses slots
   `MAINHAND, OFFHAND, FEET, LEGS, CHEST, HEAD, BODY, SADDLE`. Missing slot chances default to float
   `0.085f`; chance exactly zero skips the slot, and a chance strictly greater than `1.0f` is
   “preserved” (`withGuaranteedDrop` writes `2.0f`). When the causing entity is living, the locked
   Looting effect adds `0.01*level` only when its holder/attacker is a player; no other 26.2
   enchantment changes equipment-drop chance. An eligible nonempty stack is rejected by
   `prevent_equipment_drop`, or when the kill was not player-attributed and the slot is not
   preserved. Otherwise consume one mob `nextFloat` and drop only when it is strictly less than the
   adjusted chance. A dropped non-preserved damageable stack then consumes inner and outer `nextInt`
   calls and sets damage to `maxDamage-nextInt(1+nextInt(max(maxDamage-3,1)))`; preserved or
   nondamageable stacks consume neither. Spawn the same stack, then clear its slot.
5. Unconditional equipment overrides run after the preceding gate. Fox first drops and clears a
   nonempty main hand before invoking the entire base transaction, ignoring `mob_drops` and
   `prevent_equipment_drop`. Allay removes and spawns every internal-inventory stack, then
   drops/clears a remaining main hand unless it has `prevent_equipment_drop`. Abstract horses spawn
   every nonempty inventory slot lacking that component; chested horses then spawn one chest item
   and clear `hasChest`. Copper golems call `dropPreservedEquipment`, which traverses all eight
   slots and drops/clears only nonempty slots marked preserved. These paths use pickup delay `10`.
6. Gated custom overrides execute inside the loot gate. Piglins remove and spawn all
   internal-inventory stacks. Endermen with a carried block create a diamond axe with exactly Silk
   Touch I from `enchantment_provider/enderman_loot_drop`, evaluate that block's drops with
   `ORIGIN`, `TOOL` and optional `THIS_ENTITY`, and spawn each result. Withers spawn one nether star
   and, if entity creation succeeded, set its item age to `-6000`, extending the normal 6000-tick
   lifetime by another 6000 ticks.
7. XP is suppressed when `skipDropExperience()` was called. A player is always an XP dropper; every
   other living entity requires recent-player memory, `shouldDropExperience()` and `mob_drops`. The
   common predicate rejects babies; Monsters always accept and Tadpoles always reject. Player base
   reward is zero under `keep_inventory` or spectator, otherwise `min(experienceLevel*7,100)`. A Mob
   with positive `xpReward` starts there, then for every non-saddle slot in the eight-slot order
   whose stack is nonempty and drop chance is `<=1`, consumes `nextInt(3)` and adds `1+result`;
   Hoglin and Piglin override this to the unmodified `xpReward`. No locked 26.2 enchantment defines
   `mob_experience`, so the final enchantment pass leaves these values unchanged.
8. `ExperienceOrb.award` splits a positive reward greedily with thresholds
   `2477,1237,617,307,149,73,37,17,7,3,1`. For each piece it consumes one level `nextInt(40)`,
   searches a `1x1x1` AABB centered at the death position, and merges into the first nonremoved
   equal-value orb whose `(orbId-randomOffset)%40==0`, incrementing count and resetting age to zero.
   Otherwise it spawns one orb at the exact position with zero requested direction. Zero/negative
   reward emits nothing.

**Player and subtype overrides:**

- `ServerPlayer#die` is the authoritative player branch and does not call ordinary
  `LivingEntity#die`. It emits `ENTITY_DIE`; sends the victim a combat-kill packet containing the
  combat message when `show_death_messages`, otherwise an empty component; and broadcasts the real
  message according to team visibility (`ALWAYS`, own team only, other teams only, or none). It then
  tries to respawn left and right shoulder entities only when
  `timeEntitySatOnShoulder+20 < gameTime`, clearing each shoulder tag after the attempt. If
  `forgive_dead_players`, every nonspectator `NeutralMob` in the block-position AABB inflated by
  `(32,10,32)` receives `playerDied`.
- Next, a nonspectator server player runs the drop transaction. Its equipment override first
  delegates to the empty common method, then, unless `keep_inventory`, destroys every
  inventory/equipment stack with `prevent_equipment_drop` by container index. It next traverses the
  ordinary item list by increasing index and then equipment values in `EquipmentSlot` enum order;
  every remaining nonempty stack is dropped and its owner slot/list entry cleared. Each drop starts
  at `(x,eyeY-0.30000001192092896,z)` with pickup delay `40`, no thrower, victim RNG
  `f=nextFloat*0.5`, `angle=nextFloat*6.2831855`, and velocity
  `(-sin(angle)*f,0.20000000298023224,cos(angle)*f)`. Spectators skip items and XP entirely.
- After drops the server player increments all death-count objectives; if kill credit exists, awards
  `ENTITY_KILLED_BY`, calls the killer's `awardKillScore`, and attempts a wither rose. Then it
  broadcasts byte `3`, awards `DEATHS`, resets `TIME_SINCE_DEATH` and `TIME_SINCE_REST`, clears
  fire, frozen ticks and the on-fire flag, rechecks combat, records the current dimension/block as
  last death location, and marks the client unloaded-after-death. It does not call causing-entity
  `killedEntity`, set `dead`, set `DYING`, stop using/sleeping, or unconditionally attempt a wither
  rose when kill credit is null.
- The non-server `Player#die` wrapper calls ordinary death first, reapplies position, runs drops
  only for a nonspectator `ServerLevel` player, then sets death velocity. With a non-null source it
  uses `angle=(hurtDir+yaw)*0.017453292f` and
  `(-cos(angle)*0.1,0.10000000149011612,-sin(angle)*0.1)`; with null source it uses `(0,0.1,0)`. It
  then applies the base player death stats/fire/location updates. This wrapper is not a second
  server-player transaction.
- A nonsitting Ender Dragon's killing-blow hook restores health to `1.0f` and changes phase to
  `DYING` instead of setting `dead`; a sitting dragon leaves health lethal. Its death tick updates
  the fight, increments `dragonDeathTime`, consumes three dragon `nextFloat` calls and adds one
  explosion-emitter particle on every tick 180 through 200 inclusive, moves itself and every part
  upward by `0.10000000149011612`, and emits global level event `1028` on server death tick 1 unless
  silent. Reward is 12,000 before the first recorded dragon kill and 500 later: when `mob_drops`,
  each tick greater than 150 divisible by 5 awards `floor(reward*0.08)` and tick 200 awards
  `floor(reward*0.2)`. At tick 200 it notifies the fight, removes as `KILLED`, then emits
  `ENTITY_DIE`.
- A heart-bound Creaking that is tearing down increments `deathTime` without calling the common
  removal; after the increment exceeds 45 on the server it sends 100 pale-oak-wood and 10
  awake-creaking-heart block-crumble particles over 30% of each bounding-box dimension at speed
  zero, plays its death sound, and removes as `DISCARDED`. A heart-bound Creaking not tearing down
  and every unbound Creaking use the common timer.

**Branches and aborts:**

Bypass protection; main/off/no component; player versus nonplayer effects; removed/dead reentry;
causing entity absent/default/creeper/zombie-conversion false; player/server-player/ordinary die;
spectator and all listed game rules; adult/baby/Monster/Tadpole; player memory present but player
reference absent; loot table absent; slot empty/chance zero/preserved/prevented/player
attribution/chance miss; fox/allay/horse/chested-horse/copper-golem/piglin/enderman/wither; XP
consumed/always/conditional/zero; common/Creaking/dragon timer.

**Constants and randomness:**

All constants and draw sites are stated above. Loot-table RNG and conditions are locked data owned
by `ITM-LOOT-001`; this rule adds no implicit draws around them. Event `3` pitch consumes two
client-entity floats; event `35`'s emitter owns its client particle RNG. Common removal's byte `60`
produces exactly 20 POOF particles; each particle consumes three entity gaussians scaled by `0.02`
plus the draws used by `getRandomX/Y/Z`.

**Side effects:**

Hand/inventory/equipment mutation; effect removal/addition; stats, criteria, scoreboard and team
objectives; game events/vibrations; death/chat packets; shoulder/neutral-mob state; loot/item/XP
entities; wither-rose block/item; pose, velocity, fire/frozen/combat/death-location/client-loaded
state; sounds/particles; fight/phase state and removal.

**Gates:**

Damage tags and item components; health/dead/removed; server side and subtype; difficulty; loot
table/data and recent player memory; `mob_drops`, `mob_griefing`, `keep_inventory`,
`show_death_messages`, `forgive_dead_players`, and `ender_pearls_vanish_on_death`; spectator/team
state; equipment enchantments/drop chances; fight history and silence.

**Boundary cases and quirks:**

A bypass source leaves a totem untouched. The copied full pre-shrink stack drives protection
stats/effects, while local activation independently rescans possibly already-synchronized hands. A
zombie conversion false result suppresses drops and the first death game event but not byte
`3`/pose. Fox-held and player-inventory drops ignore `mob_drops`; player XP also ignores it but
becomes zero under `keep_inventory`. A remembered-player timer without a resolvable player still
enables player-attributed loot functions but supplies neither player nor luck. Common `setBlock`
success is ignored when placing a wither rose. Unload/removal without `die` never runs this
transaction. A dead player's in-flight ender pearl is not discarded by `die` itself: on its next
tick it discards before ordinary projectile motion only when the owner is a dead non-winning server
player and `ender_pearls_vanish_on_death` is true.

**Evidence:**

`OFF-SERVER-001`, `OFF-CLIENT-001`, `OFF-DATA-001`;
`net.minecraft.world.entity.LivingEntity#checkTotemDeathProtection(net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.item.component.DeathProtection#applyEffects(net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.world.item.consume_effects.ClearAllStatusEffectsConsumeEffect#apply(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect#apply(net.minecraft.world.level.Level,net.minecraft.world.item.ItemStack,net.minecraft.world.entity.LivingEntity)`,
`net.minecraft.client.multiplayer.ClientPacketListener#handleEntityEvent(net.minecraft.network.protocol.game.ClientboundEntityEventPacket)`,
`net.minecraft.world.entity.LivingEntity#die(net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.LivingEntity#dropAllDeathLoot(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.LivingEntity#dropFromLootTable(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,boolean,net.minecraft.resources.ResourceKey,java.util.function.Consumer)`,
`net.minecraft.world.entity.Mob#dropCustomDeathLoot(net.minecraft.server.level.ServerLevel,net.minecraft.world.damagesource.DamageSource,boolean)`,
`net.minecraft.world.entity.LivingEntity#dropExperience(net.minecraft.server.level.ServerLevel,net.minecraft.world.entity.Entity)`,
`net.minecraft.world.entity.ExperienceOrb#award(net.minecraft.server.level.ServerLevel,net.minecraft.world.phys.Vec3,int)`,
`net.minecraft.server.level.ServerPlayer#die(net.minecraft.world.damagesource.DamageSource)`,
`net.minecraft.world.entity.player.Player#dropEquipment(net.minecraft.server.level.ServerLevel)`,
`net.minecraft.world.entity.boss.enderdragon.EnderDragon#tickDeath()`,
`net.minecraft.world.entity.monster.creaking.Creaking#tickDeath()`, and every override named above.
Data: `reports/minecraft/components/item/totem_of_undying.json`,
`data/minecraft/tags/damage_type/bypasses_invulnerability.json`,
`data/minecraft/enchantment/{looting,vanishing_curse}.json`,
`data/minecraft/enchantment_provider/enderman_loot_drop.json`, entity loot tables, and
game-rule/registry reports.

**Test vectors:**

`EXP-ENT-002`: bypass/main/off/no protection and two protected hands; exact effect/RNG/event order;
common removed/dead reentry; null/player/charged-creeper/zombie-villager causing entities; every
loot/slot/player-memory/gamerule boundary; item spawn position/delay/velocity and RNG cursors; all
custom equipment/inventory/block/nether-star paths; XP predicates, rewards, split/merge;
server-player team messages, shoulders, neutral forgiveness, inventory/XP/score ordering; common
ticks 19/20, Creaking 45/46 and dragon 1/150/155/180/200. Assert game events, packets, stack
ownership, effects, stats, entities, pose/velocity, fight callbacks and removal reason in exact
order.
