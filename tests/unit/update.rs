use super::{current_with_suffix, version, version_for};

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
    assert_eq!(
        version_for("0.2.0.acme-co", "acme-co"),
        version_for("0.2.0", "")
    );
    assert_eq!(
        version_for("0.2.0-Beta1.acme", "acme"),
        version_for("0.2.0-Beta1", "")
    );
    assert!(version("0.2.0-").is_none() && version("0.2.x").is_none() && version("0.2").is_none());
}

#[test]
fn a_fourth_patch_number_ranks_above_its_base_version() {
    // GitHub #1076: `0.2.17.1` and `0.2.17.1.suffix` are patch releases of 0.2.17.
    for (lower, higher) in [
        ("0.2.17", "0.2.17.1"),
        ("0.2.17.1", "0.2.17.2"),
        ("0.2.17.9", "0.2.17.10"),
        ("0.2.17.1", "0.2.18"),
        ("0.2.17.1-beta1", "0.2.17.1"),
    ] {
        assert!(version(lower) < version(higher), "{lower} < {higher}");
    }
    assert_eq!(version_for("v0.2.17.1.acme", "acme"), version("0.2.17.1"));
    assert!(
        version("0.2.17.1").is_some_and(|v| v.4),
        "a patch release is stable"
    );
}

#[test]
fn versions_compare_only_within_their_kind() {
    // GitHub #1076: a company build compares suffixed tags; a plain build compares plain tags.
    for suffix in ["acme", "7", "co.2"] {
        let key = |tag: &str| version_for(tag, suffix);
        assert_eq!(key(&format!("0.2.17.{suffix}")), version_for("0.2.17", ""));
        assert_eq!(
            key(&format!("0.2.17.1.{suffix}")),
            version_for("0.2.17.1", "")
        );
        assert!(
            key(&format!("0.2.17.{suffix}")) < key(&format!("0.2.17.1.{suffix}")),
            "{suffix}"
        );
        assert!(
            key(&format!("0.2.17.1.{suffix}")) < key(&format!("0.2.18.{suffix}")),
            "{suffix}"
        );
        assert!(
            key("0.2.18").is_none() && key("0.2.18.1").is_none(),
            "{suffix}: a plain tag"
        );
        assert!(
            version_for(&format!("0.2.18.1.{suffix}"), "").is_none(),
            "{suffix}: plain build"
        );
    }
    assert!(version_for("0.2.18.1.acme", "").is_none());
    assert!(version_for("0.2.18.1.acme", "other").is_none());
    assert!(version_for("0.2.17.1.2", "").is_none(), "five numbers");
}
