use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "advancement",
        1688,
        "bbd362446325af20446e336a26dc75ecd7bb6752",
    );
}
