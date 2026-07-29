use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "decorated_pot_pattern",
        24,
        "982ccb083b866e9058a6f59d9b9e5f27179dc852",
    );
}
