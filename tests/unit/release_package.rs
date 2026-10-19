use super::validate_company_suffix;

#[test]
fn only_a_clean_numeric_build_can_be_packaged() {
    assert!(validate_company_suffix("\n", "0.1.25", "0.1.25").is_ok());
    assert!(validate_company_suffix("company_name", "0.1.25.company_name", "0.1.25").is_err());
    assert!(validate_company_suffix("", "0.1.25.company_name", "0.1.25").is_err());
}
