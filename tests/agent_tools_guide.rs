use serde_json::Value;
use std::path::Path;

const HTML: &str = include_str!("../src/ui/startup.html");
const SCRIPT: &str = include_str!("../src/ui/startup.js");
const MANUAL: &str = include_str!("../docs/user-manual.md");

const COMMANDS: &[&str] = &[
    "npx -y @agentmemory/agentmemory@latest",
    "/plugin marketplace add rohitg00/agentmemory",
    "codex plugin add agentmemory@agentmemory",
    "/plugin install ponytail@ponytail",
    "codex plugin add ponytail@ponytail",
    "brew install rtk",
    "winget install rtk-ai.rtk",
    "rtk init -g --codex",
    "uv tool install graphifyy",
    "graphify install --platform codex",
    "uv tool install --upgrade 'cocoindex-code[full]'",
    "codex mcp add cocoindex-code -- ccc mcp",
];

fn dialog(id: &str) -> &'static str {
    let start = HTML.find(&format!("<dialog id=\"{id}\"")).unwrap();
    let end = start + HTML[start..].find("</dialog>").unwrap();
    &HTML[start..end]
}

fn translated_message(message: &str) -> bool {
    let en: Value = serde_json::from_str(include_str!("../src/ui/messages/en.json")).unwrap();
    let ko: Value = serde_json::from_str(include_str!("../src/ui/messages/ko.json")).unwrap();
    en.as_object().unwrap().iter().any(|(key, value)| {
        let Some(english) = value.as_str() else {
            return false;
        };
        (english == message || (message.ends_with(' ') && english.starts_with(message)))
            && ko[key] != *value
    })
}

#[test]
fn guide_and_manual_open_from_settings_not_the_workspace_menu() {
    let settings = dialog("start-settings");
    assert!(settings.contains("id=\"start-tools-guide\""));
    assert!(settings.contains("id=\"start-manual-open\""));
    assert!(SCRIPT.contains(
        "q('tools-guide').onclick=()=>{q('settings').close('cancel');q('tools').showModal();}"
    ));
    assert!(SCRIPT.contains(
        "q('manual-open').onclick=()=>{q('settings').close('cancel');AC_MANUAL.open(request);}"
    ));
    for removed in ["startup.agent_tools_guide", "startup.user_manual"] {
        assert!(
            !SCRIPT.contains(removed),
            "{removed} is no longer a workspace menu item"
        );
    }
}

#[test]
fn guide_lists_exact_commands_as_untranslated_code() {
    let guide = dialog("start-tools");
    for command in COMMANDS {
        assert!(
            guide.contains(&format!("<code>{command}</code>")),
            "{command}"
        );
    }
}

#[test]
fn guide_is_static_and_runs_nothing() {
    let guide = dialog("start-tools");
    for forbidden in ["<script", "<a ", "href=", "onclick", "request("] {
        assert!(!guide.contains(forbidden), "{forbidden}");
    }
}

#[test]
fn guide_text_is_translated_and_manual_matches() {
    for message in ["Agent tools guide…", "Agent tools", "Close"] {
        assert!(translated_message(message), "{message}");
    }
    for command in COMMANDS {
        assert!(MANUAL.contains(command), "{command}");
    }
}

#[test]
fn python_verification_module_is_removed() {
    assert!(!Path::new("tests/test_agent_tools_guide.py").exists());
}
