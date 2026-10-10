//! R-36 / DD-42 / QAS-28: the README introduces a desktop product; the user manual is
//! the complete reference and documents every command the executable lists.
const README: &str = include_str!("../README.md");

fn manual() -> String {
    std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/user-manual.md"))
        .expect("docs/user-manual.md")
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
    let manual = manual();
    for command in commands {
        assert!(
            manual.contains(&format!("`agentconductor {command}")),
            "user manual misses `{command}`"
        );
    }
}

#[test]
fn user_manual_links_only_assets_the_readme_ships() {
    let manual = manual();
    for link in manual.split("](").skip(1) {
        let target = link.split(')').next().unwrap();
        if target.starts_with("assets/") {
            assert!(
                README.contains(&format!("](docs/{target})")),
                "manual asset {target} is not shipped with the README"
            );
        }
    }
}
