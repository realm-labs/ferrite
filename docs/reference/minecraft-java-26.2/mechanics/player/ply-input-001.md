# Player mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `PLY-INPUT-001` — Key state, posture and item-use modifiers shape movement intent before travel

**Parent:** `PLY-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — the transform from the seven sampled movement key states through local intent,
posture slowdown, sprint/flight transitions and input/action message cadence is complete below. OS
events, focus, screens and toggle-key policy that produce `KeyMapping.isDown` are owned by
`CLI-PREDICT-001`; obstacle detection that schedules `autoJumpTime` is owned by `PLY-AUTOJUMP-001`;
vehicle jump charging is owned by `ENT-VEHICLE-001`.

**Applies when:**

The loaded local player runs `aiStep` and later its enclosing client `tick`, or the server handles
the resulting input, sprint, fall-flying or ability message. The normative gameplay-input boundary
is the current `isDown` value of forward, backward, left, right, jump, shift and sprint mappings.

**Authoritative state:**

The client owns an immutable seven-boolean `Input`, a float `Vec2 moveVector`, `xxa/zza/jumping`,
local crouching and pose state, sprint state and timer, ability flags and double-jump timer,
auto-jump timer, item-use stack/components, current/retained input and sprint message baselines. The
server owns authoritative sprint, shift, ability and fall-flying state, and retains the most recent
client `Input`; ordinary position truth still follows `PLY-MOVE-VALIDATE-001`, not this intent
message.

**Transition and ordering:**

One loaded `LocalPlayer#tick` enters its superclass first. Before `aiStep`, entity `baseTick`
refreshes fluid interaction and dispatches `Player#updateSwimming`: ability flight forces the
swimming flag false; an existing swimming flag remains true only while sprinting, in water and not
riding; a false flag becomes true only while sprinting, underwater, not riding and the fluid state
at the current block position is water. This reads sprint/ability state from tick entry, before the
transitions below. During the subsequently dispatched `LocalPlayer#aiStep`:

1. Decrement positive `sprintTriggerTime`. Snapshot the **previous sample's** jump, shift and
   `hasForwardImpulse` before reading current keys. Compute the movement-slowdown `crouching` flag
   from that previous shift state: it is false while ability-flying, swimming or riding, or when the
   crouching dimensions do not fit; otherwise it is true when previous shift is held, or when not
   sleeping and the standing dimensions do not fit.
2. `KeyboardInput#tick` samples all seven mappings into a new immutable `Input`. For each opposing
   pair, equal booleans (both up or both down) produce `0.0f`; positive-only produces `+1.0f`;
   negative-only produces `-1.0f`. Construct `(strafe,forward)=(left-right,forward-backward)` and
   normalize it with Java float arithmetic. The only possible nonzero raw lengths are one and
   `sqrt(2)`; diagonal components therefore become the float result of `1.0f/sqrt(2.0f)`.
   `hasForwardImpulse` is strictly `moveVector.y > 1.0e-5f`.
3. If `autoJumpTime>0`, decrement it and replace only the sampled jump boolean with true; retain
   every other current boolean. This forced press is tagged as auto-jump for the ability-toggle
   gate. How the timer is scheduled is not implied here and is specified only by `PLY-AUTOJUMP-001`.
4. Derive sprint state. Reset `sprintTriggerTime` to zero when previous shift was true, when item
   use disallows sprint while riding, or when the **current raw** backward boolean is true
   (including simultaneous forward+backward). Sprint may start only when not already sprinting,
   current forward impulse is above `1.0e-5f`, sprinting is possible, item use does not disallow
   sprint, fall flying implies underwater, and moving slowly implies underwater. “Possible” means no
   blindness; for a rider, the vehicle both permits sprint and is locally authoritative, otherwise
   food level is strictly greater than six or `mayfly` is true; and, unless already ability-flying,
   the player is not in shallow water (`inWater && !underWater`). If the previous sample lacked
   forward impulse, a positive timer starts sprint immediately, otherwise the timer is loaded from
   the `sprintWindow` option (`0..10`, default `7`). Independently, a true current sprint boolean
   starts sprint. A sprinting swimmer then stops unless sprint remains possible, the player remains
   in water, and at least one of forward impulse, on-ground or shift is true. A non-swimming
   sprinter stops when sprint becomes impossible, forward impulse is lost, or a horizontal collision
   is not minor.
5. Process ability flight. A `mayfly` spectator is forced into flight when not already flying and
   sends an ability update. Otherwise a non-auto-jump rising jump edge starts the double-tap window
   by storing `jumpTriggerTime=7` when it is zero. A second rising edge while it remains nonzero
   toggles flight only when not swimming and either not riding or controlling a jumpable vehicle;
   enabling flight while grounded also calls the ordinary ground jump, then sends abilities and
   clears the timer. `Player#aiStep`, called later in this same method, decrements a positive
   `jumpTriggerTime`, so a newly stored seven is six at tick end. If current jump is true, no
   ability transition occurred, previous jump was false, the player is not climbable and local
   `tryToStartFallFlying` succeeds, send `START_FALL_FLYING`; auto-jump is not separately excluded
   from this fall-flying attempt. The server independently attempts fall flying and stops it when
   that attempt fails.
6. Before superclass `aiStep`, current shift in water calls `goDownInWater` when fluids affect the
   player. While ability-flying and controlling the camera, compute vertical direction as
   `(jump?1:0)-(shift?1:0)` and add `direction * flyingSpeed * 3.0f` to velocity Y; simultaneous
   jump+shift cancels. Vehicle jump charge/release is delegated to `ENT-VEHICLE-001`. After
   superclass movement, being grounded while still flying disables flight and sends abilities unless
   spectator.
7. When superclass movement calls `applyInput`, a camera-controlled local player first transforms
   the sampled `Vec2`. A zero vector is returned unchanged. Otherwise scale it by `0.98f`; if using
   an item and not riding, multiply by that stack's `minecraft:use_effects.speed_multiplier`; if
   `crouching || isVisuallyCrawling`, multiply by the current `minecraft:sneaking_speed` attribute
   converted from double to float. Let the resulting length be `L>0`, direction `D=v/L`, and
   `Q=sqrt(1+(min(abs(D.x),abs(D.y))/max(abs(D.x),abs(D.y)))^2)` in float operations. The final
   vector is `D * min(L*Q,1.0f)`. Store its X as `xxa`, Y as `zza`, and current jump as `jumping`;
   `yya` is untouched. This square remap preserves equal per-axis slowdown: with no item/sneak
   modifier, cardinal magnitude is `0.98f` while a diagonal is capped to unit magnitude; with total
   modifier `M` below the cap, each active diagonal axis has the same magnitude as a cardinal axis
   scaled by `0.98f*M`. A local player that is not the camera entity does not sample into
   `xxa/zza/jumping` here; inherited `applyInput` only multiplies its existing `xxa` and `zza` by
   `0.98f`.
8. At the tail of `Player#tick`, pose selection uses the now-current shift state and the swimming
   flag selected at pre-`aiStep` base tick. If even swimming dimensions do not fit, retain the old
   pose. Otherwise desired pose priority is sleeping, swimming, fall flying, spin attack, crouching
   when shift is down and ability flight is off, then standing. Spectators and passengers accept the
   desired pose without its fit test. Other players use desired when its dimensions fit, otherwise
   crouching when that fits, otherwise swimming. Every fit test uses the pose dimensions at current
   position, deflated by `1.0e-7`, against block and entity collision. A swimming pose outside water
   is “visually crawling” and therefore receives the sneaking-speed multiplier above.

After the superclass tick returns, compare the entire current immutable `Input` with
`lastSentInput`. On inequality send one `ServerboundPlayerInputPacket`, then retain that exact
immutable value; unchanged intent sends nothing. A non-passenger next synchronizes sprint before its
coordinate/status message, so order in that client tick is input delta, sprint command if changed,
then movement message. A passenger sends input delta, then an unconditional rotation message; if its
distinct root vehicle is locally authoritative it next sends vehicle movement and only then
synchronizes sprint. Ability and fall-flying messages created during `aiStep` precede this later
input-delta send.

On the server thread, `handlePlayerInput` always replaces `lastClientInput` first. Only a client
already marked loaded additionally resets last-action time and copies its shift boolean to
authoritative shared shift state. `getLastClientMoveIntent` cancels opposing left/right and
forward/backward exactly as above, builds `(strafe,0,forward)`, normalizes only as required by
`Entity#getInputVector` at scale one, and rotates it by authoritative yaw; it is a consumer of
retained intent, not the ordinary player position integrator. Sprint commands are ignored before
client-loaded and otherwise set authoritative sprint true/false without repeating the client
hunger/collision gates. An ability message sets server `flying = packet.flying && mayfly`; other
ability fields are not client-owned.

**Branches and aborts:**

Client-not-loaded tick return; every opposing-key combination; zero/cardinal/diagonal vector; camera
ownership; prior/current shift and jump edges; forced auto-jump; configured sprint window; raw
sprint key; blindness, food/mayfly, shallow/underwater, item-use, slow-pose and vehicle-authority
gates; swim/run sprint stop; spectator and double-tap flight; fall-flying attempt; vertical-flight
cancellation; pose fit/fallback; unchanged input/sprint baselines; passenger/root-vehicle paths;
server client-loaded and mayfly gates. Focus, screens, toggle/hold key policy, the auto-jump
detector and vehicle jump charging remain explicitly delegated rather than silently defaulted.

**Constants and randomness:**

`hasForwardImpulse` uses strict `>1.0e-5f`; vector normalization returns zero below length `1.0e-4f`
although key-derived inputs only have lengths zero, one or `sqrt(2)`; base intent scale is `0.98f`;
square-remap cap is `1.0f`; default syncable `sneaking_speed` is `0.3` with registered range
`[0,1]`; default `UseEffects` is `(canSprint=false, interactVibrations=true, speedMultiplier=0.2f)`
and the codec constrains the multiplier to `[0,1]`; food must be strictly `>6`; sprint-window
range/default is `0..10`/`7`; flight double-tap stores `7`; vertical flight scale is `3.0f`; pose
boxes deflate by `1.0e-7`. All transforms use Java float operations at the stated points. No RNG is
consumed.

**Side effects:**

Current/retained input, local `xxa/zza/jumping`, crouching/pose/swimming/sprint/flight/fall-flying
state, velocity Y, ordinary jump side effects, tutorial input callback,
player-input/sprint/fall-flying/ability/movement messages, server last-action time and shared shift
state, sprint attribute mutation through `setSprinting`, and ability synchronization. Camera bob
fields move halfway toward current rotations after controlled input; this presentation-only
interpolation does not alter gameplay intent.

**Gates:**

Loaded connection, current key mappings, camera entity, collision clearance, pose and medium,
abilities/game mode, blindness, food, vehicle capability/authority, item-use `UseEffects`,
sneaking-speed attribute, and retained message baselines. Difficulty and game rules do not directly
gate this transform.

**Boundary cases and quirks:**

Local movement slowdown deliberately uses the previous input sample's shift-derived `crouching`
flag, while pose selection later uses current shift; an ordinary press/release can therefore lag the
movement multiplier by one client movement tick. Swimming refresh occurs still earlier and sees
tick-entry sprint/ability state, so an `aiStep` sprint/flight transition does not retroactively
change that tick's selected swimming flag. Raw backward cancels the sprint window even when forward
is also held and net forward is zero. Cardinal input remains `0.98f` before travel while an
unmodified diagonal is remapped back to unit length. Default item use both disallows sprint and
multiplies movement by `0.2f`, but an item's data component can independently change both. Server
input retention happens even before client-loaded, while shift/action-time side effects do not.
Input, sprint, ability and coordinate messages have separate change detectors and cadence.

**Evidence:**

`OFF-CLIENT-001`, `OFF-SERVER-001`. Anchors: `net.minecraft.client.player.KeyboardInput#tick()`,
`net.minecraft.client.player.ClientInput#hasForwardImpulse()`,
`net.minecraft.client.player.ClientInput#makeJump()`,
`net.minecraft.client.player.LocalPlayer#tick()`,
`net.minecraft.client.player.LocalPlayer#aiStep()`,
`net.minecraft.client.player.LocalPlayer#applyInput()`,
`net.minecraft.client.player.LocalPlayer#modifyInput(net.minecraft.world.phys.Vec2)`,
`net.minecraft.client.player.LocalPlayer#modifyInputSpeedForSquareMovement(net.minecraft.world.phys.Vec2)`,
`net.minecraft.client.player.LocalPlayer#distanceToUnitSquare(net.minecraft.world.phys.Vec2)`,
`net.minecraft.client.player.LocalPlayer#canStartSprinting()`,
`net.minecraft.client.player.LocalPlayer#isSprintingPossible(boolean)`,
`net.minecraft.client.player.LocalPlayer#shouldStopRunSprinting()`,
`net.minecraft.client.player.LocalPlayer#shouldStopSwimSprinting()`,
`net.minecraft.client.player.LocalPlayer#onUpdateAbilities()`,
`net.minecraft.world.entity.LivingEntity#applyInput()`,
`net.minecraft.world.entity.Entity#updateSwimming()`,
`net.minecraft.world.entity.Entity#isVisuallyCrawling()`,
`net.minecraft.world.entity.player.Player#updatePlayerPose()`,
`net.minecraft.world.entity.player.Player#canPlayerFitWithinBlocksAndEntitiesWhen(net.minecraft.world.entity.Pose)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerInput(net.minecraft.network.protocol.game.ServerboundPlayerInputPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerCommand(net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket)`,
`net.minecraft.server.network.ServerGamePacketListenerImpl#handlePlayerAbilities(net.minecraft.network.protocol.game.ServerboundPlayerAbilitiesPacket)`,
and `net.minecraft.server.level.ServerPlayer#getLastClientMoveIntent()`. Data anchors:
`reports/registries.json#minecraft:attribute/minecraft:sneaking_speed` and
`reports/minecraft/components/item/<id>.json#minecraft:use_effects`.

**Test vectors:**

(1) Enumerate all opposing-pair combinations and cardinal/diagonal directions; assert immutable
booleans, exact float vector, forward threshold and square-remapped `xxa/zza`. (2) Combine
default/custom item-use multipliers with sneaking speeds `0`, `0.3`, `1`, camera/non-camera and
rider/non-rider; assert every multiply/cap and untouched `yya/jumping` branch. (3) Press/release
shift under standing/crouching/swimming clearance and record previous-sample slowdown versus
current-sample tail pose. (4) Exercise sprint window values `0`, `1`, `7`, `10`, raw sprint, both
forward+back, food `6/7`, blindness, mayfly, shallow/underwater, item `can_sprint`, major/minor
collision and vehicle authority. (5) Replay jump edges at double-tap timer boundaries, spectator
forcing, grounded enable/landing disable, jump+shift flight cancellation and fall-flying command
success/failure. (6) Change one boolean at a time and hold it unchanged while
passenger/non-passenger; assert exact input/sprint/ability/fall-flying/movement message order and
suppression. (7) Deliver input before/after client-loaded and ability true without/with `mayfly`;
assert retained input, shift/action-time, sprint, rotated move intent and flying state.
`EXP-PLY-006` is the executable regression vector; focus/event-to-KeyMapping behavior is tested by
`CLI-PREDICT-001`, not guessed here.
