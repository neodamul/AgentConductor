use super::current_with_suffix;

#[test]
fn company_suffix_is_added_only_for_a_valid_local_label() {
    assert_eq!(
        current_with_suffix("0.1.25", "company_name"),
        "0.1.25.company_name"
    );
    assert_eq!(current_with_suffix("0.1.25", "\n"), "0.1.25");
    assert_eq!(current_with_suffix("0.1.25", "../private"), "0.1.25");
}
