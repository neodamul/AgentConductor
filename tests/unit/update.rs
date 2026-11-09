use super::{current_with_suffix, version};

#[test]
fn company_suffix_is_added_only_for_a_valid_local_label() {
    assert_eq!(
        current_with_suffix("0.1.25", "company_name"),
        "0.1.25.company_name"
    );
    assert_eq!(current_with_suffix("0.1.25", "\n"), "0.1.25");
    assert_eq!(current_with_suffix("0.1.25", "../private"), "0.1.25");
}

#[test]
fn a_release_outranks_its_betas_and_betas_rank_by_number() {
    let order = [
        "0.1.61",
        "0.2.0-Beta1",
        "0.2.0-Beta2",
        "0.2.0-Beta10",
        "0.2.0",
        "0.2.1-beta1",
    ];
    for pair in order.windows(2) {
        assert!(
            version(pair[0]) < version(pair[1]),
            "{} < {}",
            pair[0],
            pair[1]
        );
    }
    assert_eq!(version("v0.2.0-Beta1"), version("0.2.0-beta1"));
    // A company suffix on the running build changes neither the core nor the prerelease.
    assert_eq!(version("0.2.0.acme-co"), version("0.2.0"));
    assert_eq!(version("0.2.0-Beta1.acme"), version("0.2.0-Beta1"));
    assert!(version("0.2.0-").is_none() && version("0.2.x").is_none() && version("0.2").is_none());
}
