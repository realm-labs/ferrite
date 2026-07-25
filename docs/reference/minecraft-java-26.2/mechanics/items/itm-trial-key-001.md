# Items mechanics

[Back to the leaf-rule manual](../README.md).

## Leaf rule `ITM-TRIAL-KEY-001` — Trial keys are plain component-exact vault inputs with encounter-correlated acquisition

**Parent:** `PLY-005`, `PLY-006`, `PLY-INPUT-001`, `PLY-INTERACT-001`, `ITM-001`,
`ITM-003`, `ITM-004`, `ITM-005`, `ITM-006`, `ITM-007`, `ITM-USE-001`,
`ITM-CONTAINER-001`, `ITM-LOOT-001`, `ITM-ADVANCEMENT-001`, `BLK-VAULT-001`,
`BLK-TRIAL-SPAWNER-001`, `WGEN-JIGSAW-TRIAL-CHAMBERS-001`, `CLI-001`, `CLI-006`,
`CLI-EFFECT-001`

**FidelityClass:** `ExactObservableBehavior`

**EvidenceStatus:** `Confirmed`

**SourceConclusion:**

`SourceSpecified` — locked plain-item registration, components, vault and trial-spawner bytecode,
trial-spawner configurations, loot tables, advancements, trial-chamber structures and client assets
close both identities, their acquisition, component-exact consumption, progress and projection.

**Applies when:**

A trial key or ominous trial key is generated, stored, patched, used on a vault, consumed, tested by
an advancement, persisted, reloaded or rendered; or the owning trial-spawner/vault/loot data is
reloaded.

**Authoritative state:**

| item | raw item ID | rarity | maximum stack | configured vault |
|---|---:|---|---:|---|
| `trial_key` | `1533` | common | `64` | nonominous trial-chamber vault |
| `ominous_trial_key` | `1534` | common | `64` | ominous trial-chamber vault |

Both are ordinary `Item` instances, nondamageable, nonconsumable, unenchanted by default and in no
direct item tag. Each has only the generic common components shown by its report, with its own
item name and model. It has no in-air behavior, cooldown, durability, attribute, recipe remainder
or specialized tooltip provider. Its semantic role comes entirely from exact stacks stored in
vault configuration, loot entries and advancement predicates.

The locked normal and ominous trial-chamber vault configurations each require one default stack of
their matching key. Vault comparison tests both item identity and the complete component patch, then
count. A renamed, model-patched, lore-bearing or otherwise component-different key remains the same
registry item for predicates but is not the configured vault key.

**Transition and ordering:**

### Trial-spawner acquisition

All 14 locked normal trial-spawner configurations omit `loot_tables_to_eject` and therefore inherit
the default equal-weight list:

1. `minecraft:spawners/trial_chamber/consumables`;
2. `minecraft:spawners/trial_chamber/key`.

All 14 ominous configurations explicitly select
`minecraft:spawners/ominous/trial_chamber/key` with weight `3` and the matching consumables table
with weight `7`. Each key table has one pool, one roll and one unmodified count-one item entry, with
its own matching random sequence.

`BLK-TRIAL-SPAWNER-001` owns encounter timing and reward ejection. At the item join, the spawner
chooses its weighted ejection table once when the first registered player's reward becomes due and
reuses that table for every remaining registered UUID in that encounter. Consequently an ordinary
encounter chooses the trial-key table with probability `1/2`, and an ominous encounter chooses the
ominous-key table with probability `3/10`; conditional on that table choice, every admitted player
reward evaluation emits one default matching key. The outcomes are encounter-correlated rather
than an independent key draw per player. An empty/missing/replaced table can emit none under the
generic loot and data-reload rules.

### Structure-loot acquisition

Only the normal key has two additional locked sources:

- the trial-chamber entrance chest pool makes `2..3` independent rolls over total weight `36`;
  the count-one trial-key entry has weight `1`;
- the trial-chamber corridor decorated-pot pool makes one roll over total weight `351`; the
  count-one trial-key entry has weight `10`.

The entrance roll alternatives have weights `5/10/10/10` for sticks, wooden axe, honeycomb and
arrows. The pot alternatives account for the other `341` weight. Both key entries explicitly set
count one and emit default components. The ominous key appears in neither table. Structure/template
placement and container/pot loot materialization remain with
`WGEN-JIGSAW-TRIAL-CHAMBERS-001` and `ITM-LOOT-001`.

There is no locked recipe, trade or other direct loot entry for either key. Normal/ominous vault
reward tables consume the keys as configured inputs; they do not directly return the matching key.

### Vault use

Generic block interaction offers a nonsecondary-use held stack to the block before the plain item.
The item itself returns `PASS` when reached. Secondary use with anything in either hand suppresses
the vault's item interaction, so a held plain key does not unlock while that suppression applies.
`BLK-VAULT-001` owns the full active-state, block-entity, rewarded-player, reward-loot and timing
transaction.

At this join, any nonempty use on an active vault returns `SUCCESS_SERVER` from the vault on both
logical sides. The server then validates a live vault block entity, active state, exact configured
key item and components, count at least one, unrewarded player and nonempty generated reward. Only
that admitted path awards the key's item-used statistic, consumes exactly one key for a
non-infinite-material player, records the UUID and starts unlocking. Infinite-material players
retain the stack. Wrong identity/components/count, rewarded player, missing block entity or empty
reward commits no key statistic or consumption; the vault's throttled failure sound and silent
aborts remain as specified by its owner.

A normal key never opens the locked ominous configuration and vice versa. Custom vault data can
replace the exact configured stack; identity alone is not privileged by the runtime.

### Advancement observation

`under_lock_and_key` uses `item_used_on_block` and requires the pre-use stack's item to be
`trial_key` plus a vault whose `ominous` state is false. `revaulting`, a goal-frame child of that
advancement, analogously requires `ominous_trial_key` and `ominous=true`.

`ServerPlayerGameMode` copies the held stack before block use, calls the vault, and triggers
`item_used_on_block` whenever the returned result consumes action. Because an active vault returns
`SUCCESS_SERVER` independently of its later server validation, advancement and vault consumption
have deliberately different predicates:

- a component-patched matching item can satisfy the advancement while exact component comparison
  rejects the key;
- an already rewarded player, empty reward result or active vault missing its block entity can
  still satisfy the advancement while consuming nothing;
- nonactive state, secondary-use suppression, wrong item identity or wrong ominous block state
  does not satisfy the scoped criterion.

The trigger observes the pre-use copy, so successful survival consumption cannot erase the tested
identity. The normal advancement is the parent of Revaulting; the criteria themselves remain one
requirement each.

### Persistence and client projection

Stacks persist item identity, count and arbitrary component patch through the generic item-stack
codec. There is no key-owned progress state. Existing vaults persist their configured exact key;
existing trial spawners persist encounter/ejection state according to their block-entity owners.
Loot/config/advancement data reload changes future evaluations without rewriting already emitted
keys or already persisted vault configuration.

Both items use direct `generated` item models with one matching texture layer and no component-based
model selection. They add no specialized tooltip line. Ingredients orders trial key immediately
after experience bottle, then ominous trial key, then the generated enchanted-book entries.
Captured/custom components can change generic name/lore/model presentation but not this default tab
entry or the advancement's item-identity match.

**Branches and aborts:**

Normal/ominous; default/component-patched stack; 0..64 and infinite materials; normal/ominous/custom
vault configuration; inactive/active and secondary-use state; absent/live block entity; exact/wrong
components; unrewarded/rewarded; empty/nonempty reward; normal/ominous encounter and selected table;
one/many registered players; chest/pot roll; advancement item/block-state combinations; saved/live
data and resource snapshots.

**Constants and randomness:**

Raw IDs `1533/1534`; maximum stack `64`; vault key count `1`; 14 normal and 14 ominous locked
spawner configs; normal table weights `1/1`; ominous weights `3/7`; key-table roll/count `1/1`;
entrance rolls `2..3`, key weight/total `1/36`; corridor-pot rolls `1`, key weight/total `10/351`.
Trial-spawner table selection occurs once per encounter reward sequence; chest/pot selections occur
once per admitted pool roll.

**Side effects:**

Loot-produced item stacks/entities or container contents; held count and item-used statistic;
vault reward/player/state transaction; item-used-on-block advancement progress; durable stack and
block-entity data; tooltip/model/tab projection.

**Gates:**

Current trial-spawner config and selected/evaluated loot table; structure loot materialization;
generic block interaction and secondary-use suppression; vault state/config/block entity,
item/components/count, rewarded set, reward result and player ability; advancement listener,
pre-use item and ominous state; current data/resource snapshots.

**State read/written:**

Reads current item/count/components, player mode/secondary-use/progress, vault state/config/server
data/reward table, trial-spawner config/cohort/fixed ejection table, structure loot context and
client resources. Writes emitted/default key stacks, player inventory/statistic, vault pending/
rewarded/state data, advancement completion and durable item/block-entity state.

**Failure behavior:**

Plain use in air or after block fallthrough passes. Secondary-use suppression prevents vault item
use. A nonactive vault does not consume action through this item path. Active-vault server rejection
can still return consuming success and trigger its scoped advancement; it does not consume the key
or award its item-used statistic. Missing/empty loot emits no key. Invalid recipes/trades do not
exist for this scope.

**Persistence boundary:**

The stack codec owns identity/count/components. Trial-spawner and vault codecs own their fixed
ejection/config/rewarded/pending state. Neither a chest/pot/spawner loot roll nor an in-flight vault
use resumes after completion. Data reload replaces future loot/config/advancement interpretation;
resource reload replaces translations/models/textures; neither rewrites emitted or persisted
stacks.

**Boundary cases and quirks:**

The keys are behaviorally plain items. Vault validity is component-exact, while both advancements
match only registry identity. An active-vault consuming result therefore can progress a rejected
patched key. Advancement inspects the pre-use copy. Trial-spawner key outcomes are correlated for
all players because the encounter fixes one reward table, not independently selected at every
ejection. Only the normal key has entrance-chest and corridor-pot acquisition.

**Evidence:**

`OFF-SERVER-001`; `OFF-CLIENT-001`; `OFF-DATA-001`; `OFF-REPORT-001`;
`net.minecraft.world.item.Items`; `net.minecraft.world.item.Item`;
`net.minecraft.server.level.ServerPlayerGameMode#useItemOn`;
`net.minecraft.world.level.block.VaultBlock#useItemOn`;
`net.minecraft.world.level.block.entity.vault.VaultBlockEntity$Server#tryInsertKey`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfig`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerConfig$Builder`;
`net.minecraft.world.level.block.entity.trialspawner.TrialSpawner`;
`net.minecraft.world.item.CreativeModeTabs`;
`reports/registries.json#minecraft:item`;
`reports/minecraft/components/item/{trial_key,ominous_trial_key}.json`;
`data/minecraft/trial_spawner/trial_chamber/**/{normal,ominous}.json`;
`data/minecraft/loot_table/spawners/{trial_chamber/key,ominous/trial_chamber/key}.json`;
`data/minecraft/loot_table/{chests/trial_chambers/entrance,pots/trial_chambers/corridor}.json`;
`data/minecraft/advancement/adventure/{under_lock_and_key,revaulting}.json`;
`data/minecraft/structure/trial_chambers/**/*.nbt`;
`assets/minecraft/{items,models/item,textures/item}/{trial_key,ominous_trial_key}.*`;
`PLY-INTERACT-001`; `ITM-USE-001`; `ITM-LOOT-001`; `ITM-ADVANCEMENT-001`;
`BLK-VAULT-001`; `BLK-TRIAL-SPAWNER-001`; `WGEN-JIGSAW-TRIAL-CHAMBERS-001`;
`CLI-EFFECT-001`; `EXP-ITM-029`.

**Test vectors:**

Generate every normal/ominous spawner reward with zero/one/many registered players and both fixed
table choices; assert encounter correlation and one default key per key-table evaluation. Exhaust
entrance/pot weights and counts. Use default and every component-patched key on normal/ominous,
inactive/active, missing-BE, rewarded and empty/nonempty-reward vaults in both ability modes with
secondary use on/off; separately assert statistic, consumption and both advancement predicates.
Persist/reload all states, reload data/resources, and inspect tooltip/model/Ingredients projection.

**Limits:**

This leaf does not duplicate generic loot selection, trial-spawner encounter timing, vault
state/reward ejection, block-interaction packets, advancement listener, item-stack codec or
resource-pack algorithms. Those remain with the cited owners; this rule fixes the two plain item
identities and their exact acquisition/consumption/progress joins.
