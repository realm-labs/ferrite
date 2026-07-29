use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "sulfur_cube_archetype",
        12,
        "50df53120b294ecbe8769d681d12e4a7acb20363",
    );
}
