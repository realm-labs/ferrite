use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category(
        "worldgen/feature",
        63,
        "da0961440046464b11527a98ec4a8e6d53ddafdf",
    );
}
