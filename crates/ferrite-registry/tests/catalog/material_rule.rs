use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "material_rule",
        4,
        "b4989ab92e5c03719fd1ebb4901251bdae044fea",
    );
}
