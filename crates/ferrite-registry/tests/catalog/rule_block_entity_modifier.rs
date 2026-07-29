use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "rule_block_entity_modifier",
        4,
        "c6a014b637b0f43358d1e517408ab5fdcc75f825",
    );
}
