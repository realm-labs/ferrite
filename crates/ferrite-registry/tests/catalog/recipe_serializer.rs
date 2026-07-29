use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "recipe_serializer",
        21,
        "7632b57a44d894fe4bff43613e948c29fabc226d",
    );
}
