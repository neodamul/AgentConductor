pub mod acio;
pub mod activity;
pub mod adapters;
pub mod agent_settings;
pub mod archive;
pub mod availability;
pub mod claude_control;
pub mod cli_io;
pub mod cooperative;
pub mod coordinator;
pub mod gates;
pub mod git;
pub mod guard;
pub mod icons;
pub mod knowledge;
pub mod leases;
pub mod macos_bundle;
pub mod manual;
pub mod memory;
pub mod messenger;
pub mod models;
pub mod native_guard;
#[cfg(feature = "native-ui-probe")]
pub mod native_ui_probe;
pub mod phases;
pub mod plan;
pub mod plan_rules;
pub mod planner;
pub mod process;
pub mod progress;
pub mod prompt_optimizer;
pub mod protocol;
pub mod quota;
pub mod release_archive;
pub mod release_notes;
pub mod release_package;
pub mod release_smoke;
pub mod roles;
pub mod slug_progress;
pub mod slugs;
pub mod state;
pub mod ui;
pub mod workflow;

pub mod briefing;
pub mod chat;

pub mod desktop;

pub mod executable;

pub mod mcp;

pub mod subprocess;

pub mod folder;
/// DD-135: release checks and self-update.
pub mod update;
pub mod update_source;
