use super::*;

#[test]
fn only_an_empty_argument_list_opens_the_workspace_ui() {
    assert!(is_workspace_launch(&[]));
    assert!(!is_workspace_launch(&["--help".into()]));
    assert!(!is_workspace_launch(&["run".into(), "plan.json".into()]));
}

#[test]
fn windows_falls_back_to_profile_when_home_is_missing_or_empty() {
    let profile = std::env::temp_dir().join("desktop-profile");
    for home in [None, Some(OsString::new())] {
        assert_eq!(
            select_home(home, Some(profile.clone().into_os_string()), true).unwrap(),
            profile
        );
    }
}

#[test]
fn home_precedence_and_missing_value_errors_preserve_platform_contract() {
    let home = std::env::temp_dir().join("desktop-home");
    let profile = std::env::temp_dir().join("desktop-profile");
    for windows in [false, true] {
        assert_eq!(
            select_home(
                Some(home.clone().into_os_string()),
                Some(profile.clone().into_os_string()),
                windows
            )
            .unwrap(),
            home
        );
        assert!(
            select_home(None, None, windows)
                .unwrap_err()
                .contains("ui-new")
        );
    }
    assert!(select_home(None, Some(profile.into_os_string()), false).is_err());
    assert!(select_home(Some(OsString::new()), Some(OsString::new()), true).is_err());
}
