use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "material_condition",
        11,
        "99dbf2961c296989eb7c64a9051a031730302c3e",
    );
}
