//! #130: the update repository is a Setting, validated before saving. It may live on any
//! GitHub-compatible HTTPS host, not only github.com. HOME is set once in this test binary.
use agentconductor::update_source as source;

#[path = "support/env.rs"]
mod test_env;

#[test]
fn repository_is_a_slug_or_an_https_url_on_any_host() {
    for (input, saved) in [
        ("owner/name", "owner/name"),
        ("  owner/na.me_x  ", "owner/na.me_x"),
        ("https://github.com/owner/name", "owner/name"),
        ("https://github.com/owner/name.git", "owner/name"),
        ("https://github.com/owner/name/", "owner/name"),
        (
            "https://ghe.corp.example/team/tool",
            "https://ghe.corp.example/team/tool",
        ),
        (
            "https://git.internal:8443/team/tool.git/",
            "https://git.internal:8443/team/tool",
        ),
        ("", ""),
    ] {
        assert_eq!(
            source::normalize_repository(input).unwrap(),
            saved,
            "{input}"
        );
    }
    for bad in [
        "http://ghe.corp.example/team/tool",
        "ftp://ghe.corp.example/team/tool",
        "https://ghe.corp.example",
        "https://ghe.corp.example/team",
        "https://user:pw@ghe.corp.example/team/tool",
        "https://ghe corp/team/tool",
        "https://ghe.corp.example/team/tool/releases",
        "https://ghe.corp.example/team/tool?x=1",
        "https://github.com/owner/name#frag",
        "owner/name/extra",
        "owner",
        "../name",
        "owner/..",
        "-owner/name",
        "own er/name",
        "owner/na?me",
    ] {
        assert!(source::normalize_repository(bad).is_err(), "{bad}");
    }
}

#[test]
fn releases_are_read_from_the_saved_hosts_api() {
    let root = std::env::temp_dir().join(format!(
        "ac-update-source-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let mut env = test_env::lock();
    env.set("HOME", root.join("home"));
    assert_eq!(source::repository(), source::DEFAULT_REPOSITORY);
    assert_eq!(
        source::api_url(),
        "https://api.github.com/repos/neodamul/AgentConductor/releases/latest"
    );

    source::save("https://github.com/acme/tool.git").unwrap();
    assert_eq!(source::repository(), "acme/tool");
    assert_eq!(
        source::api_url(),
        "https://api.github.com/repos/acme/tool/releases/latest"
    );

    source::save("https://ghe.corp.example/team/tool").unwrap();
    assert_eq!(
        source::api_url(),
        "https://ghe.corp.example/api/v3/repos/team/tool/releases/latest"
    );
    let shown = source::settings();
    assert_eq!(shown["repository"], "https://ghe.corp.example/team/tool");
    assert_eq!(shown["default_repository"], source::DEFAULT_REPOSITORY);

    // Nothing is saved when the value is invalid.
    assert!(source::save("http://ghe.corp.example/team/tool").is_err());
    assert_eq!(source::repository(), "https://ghe.corp.example/team/tool");

    // The page reads and writes the same value through the request bridge.
    let exe = std::path::Path::new(env!("CARGO_BIN_EXE_agentconductor"));
    let room = agentconductor::ui::Room::startup(&root.join("desktop"), exe).unwrap();
    let call = |v: serde_json::Value| room.dispatch(v.to_string().as_bytes());
    let set =
        call(serde_json::json!({"kind":"set_update_settings","repository":"acme/tool"})).unwrap();
    assert_eq!(set["repository"], "acme/tool");
    let error = call(serde_json::json!({"kind":"set_update_settings","repository":"ftp://a/b/c"}))
        .unwrap_err();
    assert!(error.contains("ftp://a/b/c"), "{error}");
    assert_eq!(
        call(serde_json::json!({"kind":"update_settings"})).unwrap()["repository"],
        "acme/tool"
    );
    drop(room);

    source::save("").unwrap();
    assert_eq!(source::repository(), source::DEFAULT_REPOSITORY);
    std::fs::remove_dir_all(root).unwrap();
}
