use super::support::verify_category;

#[test]
fn locked_contract() {
    verify_category("potion", 46, "59ad098ece88a6636d88b42c6c059bf014ac41bd");
}
