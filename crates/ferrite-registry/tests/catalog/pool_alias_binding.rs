use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "pool_alias_binding",
        3,
        "626dc7d831d52ef16223f1a9de5d042fc91ef005",
    );
}
