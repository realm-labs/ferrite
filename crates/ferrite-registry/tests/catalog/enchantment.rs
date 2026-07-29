use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "enchantment",
        43,
        "928360743b0d160a0b1ad8acf9589567ac16be96",
    );
}
