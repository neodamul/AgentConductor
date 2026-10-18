use eframe::egui::{self, RichText};

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

pub fn show(open: &mut bool, language: &str, ui: &mut egui::Ui) {
    if !*open {
        return;
    }
    let text = |id| crate::ui::messages::in_language(language, id);
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.heading(text("startup.agent_tools"));
            if ui.button(text("startup.close")).clicked() {
                *open = false;
            }
        });
        ui.label(text(
            "startup.optional_tools_that_make_claude_code_and_codex",
        ));
        ui.label(text("startup.install_uv_first_if_needed"));
        egui::ScrollArea::vertical()
            .max_height(320.0)
            .show(ui, |ui| {
                for command in COMMANDS {
                    ui.horizontal_wrapped(|ui| {
                        ui.add(
                            egui::Label::new(RichText::new(*command).monospace()).selectable(true),
                        );
                        let copy = text("native.copy");
                        if ui
                            .button(copy)
                            .accessible_name(format!("{copy} {command}"))
                            .clicked()
                        {
                            ui.ctx().copy_text((*command).to_owned());
                        }
                    });
                }
            });
    });
}
