use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category("game_rule", 59, "333a8ac103f20d5e9d3eecb7ec1e57311389c7c6");
}
