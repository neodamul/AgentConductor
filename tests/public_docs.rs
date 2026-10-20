//! R-36 / DD-42 / QAS-28: the README introduces a desktop product; the user manual is
//! the complete reference and documents every command the executable lists.
const README: &str = include_str!("../README.md");

#[test]
fn manuals_explain_timeout_auto_retry_and_stop() {
    let english = include_str!("../docs/user-manual.md");
    let english_html = include_str!("../docs/user-manual.html");
    let korean = include_str!("../docs/user-manual.ko.md");
    let korean_html = include_str!("../docs/user-manual.ko.html");
    let qas = include_str!("../docs/05-qas.md");
    for manual in [english, english_html] {
        assert!(manual.contains("after a bounded backoff"));
        assert!(manual.contains("Stop cancels a pending retry"));
    }
    for manual in [korean, korean_html] {
        assert!(manual.contains("제한된 대기 후 자동으로 다시 실행"));
        assert!(manual.contains("재시도를 취소합니다"));
    }
    assert!(qas.contains("Wall-timeouts retry automatically with bounded backoff"));
}

#[test]
fn issue_nine_completion_audit_distinguishes_code_baseline_and_historical_verification() {
    let audit = include_str!("../docs/09-completion-audit.md");
    let wiki = include_str!("../knowledge/wiki/native-rust-ui-migration.md");
    assert!(audit.contains("4a114ec6a76141c48ebe8138694398572cae1acd"));
    assert!(audit.contains("a8be85fc2392821fab345d3e1b879a6aa9bc6714"));
    assert!(audit.contains("7289562e5de394555b196da75b0ecf3904f3c53c"));
    assert!(audit.contains("3/25 (12%)"));
    assert!(audit.contains("issue9-a8be85f-workspace-ui.json"));
    assert!(audit.contains("4a114ec6a76141c48ebe8138694398572cae1acd"));
    assert!(audit.contains("issue9-4a114ec-plan-gate.json"));
    assert!(audit.contains("issue9-4a114ec-settings-batch.json"));
    assert!(audit.contains("122 tests across 23 suites"));
    assert!(wiki.contains("4a114ec6a76141c48ebe8138694398572cae1acd"));
    assert!(wiki.contains("3/25"));
    assert!(audit.contains("At tested code head `27e307e`"));
    assert!(audit.contains("768 tests / 14 ignored across 260 suites"));
    assert!(audit.contains("Actions run `39424348558` completed with test failures"));
    assert!(audit.contains("363 passed / 14 ignored across 120 suites"));
    assert!(audit.contains("22 packaged workflows"));
    let plan = include_str!("../docs/07-implementation-plan.md");
    assert!(
        plan.contains("Batch-fix plan gating and stabilize the related native UI journey tests")
    );
    assert!(plan.contains(
        "packaged workflows; this grouped control/test correction adds no end-to-end workflow"
    ));
    assert!(!audit.contains("remains draft at head `bb0ff3950fcf5b28ee61205987131784d1ae107a`"));
}

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
