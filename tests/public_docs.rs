//! R-36 / DD-42 / QAS-28: the README introduces a desktop product; the user manual is
//! the complete reference and documents every command the executable lists.
const README: &str = include_str!("../README.md");

#[test]
fn readme_is_a_short_desktop_first_introduction_without_command_reference() {
    assert!(
        README.lines().count() < 120,
        "README is a reference manual again"
    );
    for fence in ["```sh", "```bash", "```shell", "```json"] {
        assert!(
            !README.contains(fence),
            "README must not contain {fence} blocks"
        );
    }
    let screenshots = README.matches("](docs/assets/").count();
    assert!(screenshots >= 2, "README needs real screenshots");
    assert!(README.contains("](docs/user-manual.md)"));
    assert!(README.contains("](docs/user-manual.ko.md)"));
    for section in [
        "## What it does",
        "## How orchestration works",
        "## How roles are assigned",
        "## When an account hits its usage limit",
        "## What you need",
        "## Getting started",
    ] {
        assert!(README.contains(section), "README misses {section}");
    }
    assert!(README.contains("agentmemory") && README.contains("optional"));
}

#[test]
fn public_agent_room_captions_do_not_claim_unverified_rust_provenance() {
    let readme = README;
    let manual = include_str!("../docs/user-manual.md");
    let korean = include_str!("../docs/user-manual.ko.md");
    for document in [readme, manual, korean] {
        assert!(
            !document.contains("Native Rust Agent Room")
                && !document.contains("Rust 네이티브 Agent Room"),
            "a filename does not establish Rust screenshot provenance"
        );
    }
}

#[test]
fn public_manuals_do_not_embed_the_unverified_legacy_status_capture() {
    for document in [
        README,
        include_str!("../docs/user-manual.md"),
        include_str!("../docs/user-manual.ko.md"),
    ] {
        assert!(
            !document.contains("ui-status-lights-native.jpg"),
            "status screenshot provenance was not verified against the current Rust UI"
        );
    }
}

#[test]
fn user_manual_documents_every_command_listed_by_the_executable() {
    let help = std::process::Command::new(env!("CARGO_BIN_EXE_agentconductor"))
        .arg("--help")
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&help.stdout).to_string() + &String::from_utf8_lossy(&help.stderr);
    let commands: Vec<&str> = text
        .lines()
        .skip_while(|l| !l.starts_with("Commands:"))
        .skip(1)
        .take_while(|l| l.starts_with("  "))
        .filter_map(|l| l.split_whitespace().next())
        .collect();
    assert!(commands.len() > 30, "unexpected help output: {text}");
    for path in ["docs/user-manual.md", "docs/user-manual.ko.md"] {
        let manual =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
                .expect(path);
        for command in &commands {
            assert!(
                manual.contains(&format!("`agentconductor {command}")),
                "{path} misses `{command}`"
            );
        }
    }
}

#[test]
fn user_manual_links_only_assets_the_readme_ships() {
    for path in ["docs/user-manual.md", "docs/user-manual.ko.md"] {
        let manual =
            std::fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
                .unwrap();
        for link in manual.split("](").skip(1) {
            let target = link.split(')').next().unwrap();
            if target.starts_with("assets/") {
                assert!(
                    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("docs")
                        .join(target)
                        .is_file(),
                    "missing asset {target}"
                );
                assert!(
                    README.contains(&format!("](docs/{target})")),
                    "manual asset {target} is not shipped with the README"
                );
            }
        }
    }
}
