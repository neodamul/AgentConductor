//! GitHub #290 (R-200 / DD-212): the user-facing docs of a release are checked by tests, not by a
//! checklist. The README and the manuals name the package version, and every screenshot they
//! show exists and every captured screen is used. (Screenshot freshness needs git history and is
//! checked by `scripts/check_screenshots.py` when a release is published.)
use std::{collections::BTreeSet, fs, path::Path};

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

#[test]
fn readme_and_manual_sentences_name_the_package_version() {
    let version = env!("CARGO_PKG_VERSION");
    for (path, needle) in [
        ("README.md", format!("**Version:** {version} ")),
        (
            "docs/user-manual.md",
            format!("The app version is beside the title. Version {version} "),
        ),
        (
            "docs/user-manual.ko.md",
            format!("앱 제목 옆의 숫자가 실행 중인 버전입니다. {version}에서는"),
        ),
    ] {
        assert!(
            read(path).contains(&needle),
            "{path} must say {needle:?}: update it with the version bump"
        );
    }
}

/// Targets of Markdown images (`![alt](target)`) that point into an assets folder.
fn images(text: &str) -> BTreeSet<String> {
    text.split("](")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .filter(|target| target.contains("assets/") && target.ends_with(".png"))
        .map(|target| target.rsplit('/').next().unwrap().to_owned())
        .collect()
}

#[test]
fn every_shown_screenshot_exists_and_every_captured_screen_is_shown() {
    let shown: BTreeSet<String> = ["README.md", "docs/user-manual.md", "docs/user-manual.ko.md"]
        .iter()
        .flat_map(|path| images(&read(path)))
        .collect();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/assets");
    for name in &shown {
        assert!(
            root.join(name).is_file(),
            "docs/assets/{name} is shown but missing"
        );
    }
    for entry in fs::read_dir(&root).unwrap() {
        let name = entry.unwrap().file_name().to_string_lossy().into_owned();
        if name.starts_with("ui-") && name.ends_with("-current.png") {
            assert!(
                shown.contains(&name),
                "docs/assets/{name} is not shown in the README or a manual"
            );
        }
    }
}

#[test]
fn image_targets_are_read_from_markdown() {
    let text = "![a](docs/assets/ui-a-current.png) ![b](assets/ui-b-current.png) ![c](https://x/y.png) [d](docs/x.md)";
    assert_eq!(
        images(text).into_iter().collect::<Vec<_>>(),
        ["ui-a-current.png", "ui-b-current.png"]
    );
}

#[test]
fn publishing_waits_for_the_screenshot_check_with_full_history() {
    let workflow = read(".github/workflows/release.yml");
    assert!(workflow.contains("python3 scripts/check_screenshots.py"));
    assert!(
        workflow.contains("fetch-depth: 0"),
        "the check needs the full history"
    );
    assert!(
        workflow.contains("needs: [verify, screenshots]"),
        "a release is published only after the screenshot check"
    );
}
