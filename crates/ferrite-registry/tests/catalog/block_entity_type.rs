use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "block_entity_type",
        49,
        "44833e6c7155dce89416ab85b64df3170bf32531",
    );
}
