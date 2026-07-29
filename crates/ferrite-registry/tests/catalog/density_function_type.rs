use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "density_function_type",
        34,
        "0b1d8cacbf57a265a1556cc4e05738fd14158c81",
    );
}
