use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "structure_processor",
        11,
        "95c99296898e18847252ab71f15fc6951d1b432e",
    );
}
