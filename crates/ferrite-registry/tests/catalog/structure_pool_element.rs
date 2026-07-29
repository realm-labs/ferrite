use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "structure_pool_element",
        5,
        "ee1323cfdecbeecb98262591e3d7ca8b6f9ba77e",
    );
}
