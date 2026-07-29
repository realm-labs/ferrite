use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "dimension_type",
        4,
        "b0fb68dacb105af7c5f4a35d5bd67ceae1a9e296",
    );
}
