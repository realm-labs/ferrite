use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "environment_attribute",
        48,
        "c9ad03701e2953d886eb80ba1a8616db0abca632",
    );
}
