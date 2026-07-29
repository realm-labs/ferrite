use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category("ticket_type", 9, "e4d0dc82dd0e0e6a6942df16c6fc0d1dfec9bf9b");
}
