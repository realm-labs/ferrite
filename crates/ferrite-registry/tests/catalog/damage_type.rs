use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "damage_type",
        51,
        "a87189dae025e2e5c910528d96f3cc763111f281",
    );
}
