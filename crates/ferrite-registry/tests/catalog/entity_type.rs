use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "entity_type",
        158,
        "89c65ced717838aeb3da47f8f72c43d87a37f6ac",
    );
}
