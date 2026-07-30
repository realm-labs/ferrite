# Prismarine Item Runtime

`G01-P6-S002` implements `ITM-PRISMARINE-MATERIAL-RUNTIME-001` for Prismarine Shards and
Prismarine Crystals.

## Runtime boundary

The shared item catalog now recognizes the two identities as Java raw IDs 1277 and 1278 while
retaining resource identity as the persistent lookup key. The partition verifier requires exactly
two members in `prismarine-material-runtime`, with common rarity, maximum stack 64 and no food or
consumable defaults.

`item::runtime::prismarine` owns the leaf-specific inputs:

- Guardian and Elder Guardian both use base shard count 0..2 and a per-Looting-level 0..1 bonus;
- their secondary Crystal entry has weight two, with total secondary weights five and six
  respectively;
- Buried Treasure uses 1..3 rolls, Crystal weight 5/15 and count 1..5;
- Sea Lantern selects itself for Silk Touch level at least one, otherwise Crystals with base 2..3,
  per-Fortune-level 0..1 bonus, clamp 1..5 and explosion decay;
- the four building recipes preserve exact kind, pattern, input counts and Shard-versus-Crystal
  unlock criteria.

## Cross-owner boundary

The profiles are deterministic semantic inputs. Generic weighted selection, count-provider
sampling, Looting/Fortune iteration, explosion survival, recipe grid allocation, advancement
publication, persistence and packet projection retain their audited shared owners. This partition
does not duplicate those engines.

## Validation

`crates/ferrite-gameplay/tests/slices/items/blk_002.rs` verifies the imported family and component
defaults, both Guardian profiles, Buried Treasure, both Sea Lantern alternatives and all four
recipe/unlock records. The existing BLK-001 item tests run alongside it to prevent catalog
extension from changing the previous partition.
