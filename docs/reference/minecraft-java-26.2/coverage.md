# Locked coverage report

Generated/verified on 2026-07-19 from server SHA-1 `823e2250d24b3ddac457a60c92a6a941943fcd6a` and client SHA-1 `2dc72797acbc1b63fc16a11c4ac393605f453754`.

## Documentation

- Stable parent rules: 65/65, each referenced by at least one leaf rule.
- Implementation-level leaf rules: 86, spread across all ten subsystems.
- Directed experiment definitions: 47; all currently `planned`, so none is incorrectly counted as confirming evidence.
- Source locators: 1,175 across 463 classes, verified by `javap -p -s` against locked jars and the locked fastutil dependency used by prediction reconciliation.
- English is the sole normative language; there is no translation mirror to drift.

## Content catalog

| Kind | IDs | Sorted-ID SHA-1 |
|---|---:|---|
| block | 1,196 | `8b11cf08c57a1d88e979fe0c695b23a9a29a5579` |
| block_entity_type | 49 | `44833e6c7155dce89416ab85b64df3170bf32531` |
| fluid | 5 | `f574dae348e4df9d2b91cff85970ade239431645` |
| ticket_type | 9 | `e4d0dc82dd0e0e6a6942df16c6fc0d1dfec9bf9b` |
| game_rule | 59 | `333a8ac103f20d5e9d3eecb7ec1e57311389c7c6` |
| item | 1,537 | `a3974d51eb37878f2e5227bf37febe44a4246468` |
| entity_type | 158 | `89c65ced717838aeb3da47f8f72c43d87a37f6ac` |
| mob_effect | 40 | `fe57e113459ca51f5ced3d853c75027a30342f22` |
| menu | 25 | `19b8c933cb322f0d3235a63d0d9e6fc7018cbcd6` |
| recipe_serializer | 21 | `7632b57a44d894fe4bff43613e948c29fabc226d` |
| potion | 46 | `59ad098ece88a6636d88b42c6c059bf014ac41bd` |
| recipe | 1,585 | `1c63ef263ed69d97012bdc7dedb4230b616f4da0` |
| loot_table | 1,355 | `d080b4bb5b9c05c12dbe0e0b0b06d6f06b77f116` |
| advancement | 1,688 | `bbd362446325af20446e336a26dc75ecd7bb6752` |
| worldgen | 963 | `cc19230cb9179c06f12e5d99973b4934e4a2733d` |
| worldgen/feature | 63 | `da0961440046464b11527a98ec4a8e6d53ddafdf` |
| pool_alias_binding | 3 | `626dc7d831d52ef16223f1a9de5d042fc91ef005` |
| structure_pool_element | 5 | `ee1323cfdecbeecb98262591e3d7ca8b6f9ba77e` |
| structure_processor | 11 | `95c99296898e18847252ab71f15fc6951d1b432e` |
| rule_test | 6 | `2dd70628ed51c6de583e72454bfc1779069e66a0` |
| pos_rule_test | 3 | `a762dc72953478c9f958d1a2e2363772afb65d2f` |
| rule_block_entity_modifier | 4 | `c6a014b637b0f43358d1e517408ab5fdcc75f825` |
| structure_type | 16 | `33281e4ca75391bed9e335eecb722f5fb7dd3b04` |
| density_function_type | 34 | `0b1d8cacbf57a265a1556cc4e05738fd14158c81` |
| material_condition | 11 | `99dbf2961c296989eb7c64a9051a031730302c3e` |
| material_rule | 4 | `b4989ab92e5c03719fd1ebb4901251bdae044fea` |
| damage_type | 51 | `a87189dae025e2e5c910528d96f3cc763111f281` |
| sulfur_cube_archetype | 12 | `50df53120b294ecbe8769d681d12e4a7acb20363` |
| enchantment | 43 | `928360743b0d160a0b1ad8acf9589567ac16be96` |
| dimension_type | 4 | `b0fb68dacb105af7c5f4a35d5bd67ceae1a9e296` |
| environment_attribute | 48 | `c9ad03701e2953d886eb80ba1a8616db0abca632` |
| **Total** | **9,054** | all IDs classified exactly once |

This is structural catalog coverage, not a claim that all 9,054 entries are behaviorally audited. `DataOnly` entries get their values from the locked query result; behavior-family entries inherit a leaf state machine; special entries identify explicit dispatch; `Unreviewed` entries remain readiness blockers. The current verified backlog is 788 explicitly `Unreviewed` IDs.

## Reproduce

```sh
cargo run -p mc-reference --bin mc-ref -- coverage
cargo run -p mc-reference --bin mc-ref -- experiment verify
cargo run -p mc-reference --bin mc-ref -- verify --offline
```

The report is valid only while the commands reproduce all counts and hashes. A later release must use a sibling directory and a new report.
