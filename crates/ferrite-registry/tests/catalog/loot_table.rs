use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "loot_table",
        1355,
        "d080b4bb5b9c05c12dbe0e0b0b06d6f06b77f116",
    );
}
