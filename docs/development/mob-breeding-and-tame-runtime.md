# MOB-006 Breeding and Tame Runtime

`G01-P7-S010` implements `MOB-BREED-TAME-001` as protocol-neutral Region transitions. The owning
Region supplies entity clocks, item/tag admission, ordered mate observations, variant state and
explicit mob/level RNG draws.

## Responsibility split

`ferrite-gameplay::mob::runtime::mob_006` has four owners:

- `age` owns signed age ticks, forced growth, lock toggles, food interaction and love clocks;
- `breeding` owns compatibility, nearest mate, approach timing, generic child commit/failure and
  special producer ordering;
- `families` owns closed inheritance probabilities, equine attributes/parentability and
  child/egg/Brain producer facts;
- `tame` owns tame authority/events, trust admission, horse temper and owner teleport.

## Age, love and mate goal

Default baby age is `-24000`. Unlocked living-server baby/adult ages move one tick toward zero;
crossing zero updates synced baby state and invokes the species hook. Forced growth adds
`seconds*20`, caps through zero, accumulates the applied delta and installs that forced age as the
post-growth positive cooldown, with a forty-tick particle timer. Feeding growth is the audited ten
percent of whole seconds until adulthood.

Golden dandelion locking is baby/cooldown/tag gated, resets species baby age, consumes one and uses
forty particles; only locking sets stored persistence. Server food at age zero and no love consumes
then installs love 600/cause/event 18. An unlocked baby instead grows. Nonzero age or damage clears
love; otherwise it decrements and emits hearts on remaining multiples of ten.

Base mating rejects self/different runtime class and requires both love timers. The radius-eight
query chooses strictly nearest compatible non-panicking candidate, retaining encounter order on a
tie. The goal continues through partner life/love/nonpanic and `loveTime < 60`; navigation failure
is not a predicate. Each tick looks, navigates, increments, and attempts at adjusted tick 30 only
strictly inside squared distance nine.

## Child and inheritance commits

Null child construction changes nothing, allowing retry. A generic child becomes baby and snaps to
the actor before finalization. Actor then partner supplies criterion cause; criteria precede both
6000 parent cooldowns/love clears/event 18. Optional `1..7` XP is inserted before the child, and
child insertion failure rolls nothing back.

The family dispatch preserves ordinary same-type producers and parent selection, Mooshroom 1/1024
mutation, Rabbit 19/20 inheritance, Axolotl 1/1200 rare variant/persistence, Goat screaming 0.02,
Horse 4/9–4/9–1/9 coat and 2/5–2/5–1/5 markings, Llama strength and reflected equine attributes.
Equine parentability additionally requires tame/adult/full-health/in-love and neither vehicle nor
passenger. Fox and Allay use insertion-before-event special order; Turtle/Frog/Sniffer and Brain
Villager/Hoglin facts remain distinct from ordinary immediate-child timing.

## Tame, trust and owner teleport

Tame flag bit four, sitting bit one and owner assignment are authoritative; entity events only
present hearts/smoke. Taming does not generically set persistence. Wolf/Cat/Nautilus use one-in-
three admission, Parrot one-in-ten, with species-specific navigation/target/sit effects. Ocelot
trust requires a running tempt goal, untrusted state, food and strict squared distance below nine;
events 41/40 do not create an owner.

Horse taming checks its adjusted cadence, then succeeds only for positive maximum temper and a
strict draw below current temper. Failure adds five up to the maximum, with ejection/mad/event work
owned by the caller. Owner teleport begins at squared distance 144, is blocked by sitting,
passenger, leash and spectator owner, makes ten inclusive `[-3,3]/[-1,1]/[-3,3]` samples with a
horizontal delta at least two, rejects leaves for nonfliers and collision, then snaps to block
center and stops navigation.

## Validation

`crates/ferrite-gameplay/tests/slices/mobs/mob_006.rs` owns the source-specified slice. Its
seventeen tests lock age/forced/lock/love boundaries, mate tie/timing/range, null/generic/special
child ordering, key inheritance probabilities and producer families, tame/trust odds and authority,
horse temper equality and all owner-teleport gates/offsets. `G01-P7-B1` remains responsible for
composing these transitions with inventories, entity addition, criteria, persistence and protocol
projection.
