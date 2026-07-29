use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "pos_rule_test",
        3,
        "a762dc72953478c9f958d1a2e2363772afb65d2f",
    );
}
