use super::*;
use eframe::egui;

#[test]
fn elapsed_uses_minutes_and_seconds_without_truncating_minutes() {
    assert_eq!(format_elapsed(Duration::from_secs(65)), "1:05");
    assert_eq!(format_elapsed(Duration::from_secs(7_265)), "121:05");
}

#[test]
fn persisted_start_restores_elapsed_without_producing_negative_time() {
    let now = Instant::now();
    let restored = WaitClock::restore(Some(1_000), 121_000, now);
    assert_eq!(restored.elapsed_at(now), Duration::from_secs(120));
    assert_eq!(
        WaitClock::restore(Some(9_000), 1_000, now).elapsed_at(now),
        Duration::ZERO
    );
}

#[test]
fn snapshot_observation_resets_only_the_silence_age() {
    let now = Instant::now();
    let mut clock = WaitClock::start(now);
    assert_eq!(
        clock.silent_for_at(now + Duration::from_millis(4_999)),
        None
    );
    assert_eq!(
        clock.silent_for_at(now + Duration::from_secs(5)),
        Some(Duration::from_secs(5))
    );
    clock.observe(now + Duration::from_secs(5));
    assert_eq!(
        clock.elapsed_at(now + Duration::from_secs(8)),
        Duration::from_secs(8)
    );
    assert_eq!(clock.silent_for_at(now + Duration::from_secs(8)), None);
}

#[test]
fn localized_label_shows_elapsed_and_stale_state() {
    let now = Instant::now();
    let clock = WaitClock::start(now);
    assert_eq!(
        label(clock, "en", now + Duration::from_secs(4)),
        "Elapsed 0:04"
    );
    assert_eq!(
        label(clock, "en", now + Duration::from_secs(65)),
        "Elapsed 1:05 · No new state for 1:05"
    );
    assert_eq!(
        label(clock, "ko", now + Duration::from_secs(70)),
        "경과 1:10 · 새 상태 없음 1:10"
    );
}

#[test]
fn active_snapshot_restores_job_time_and_state_events_reset_silence() {
    let now = Instant::now();
    let mut active = ActiveWork::default();
    let running = serde_json::json!({"revision":"r1","job":{"kind":"chat","id":"c1","state":"running","started_ms":1_000}});
    active.observe_snapshot(&running, now, 121_000);
    assert_eq!(active.label("chat", "en", now), Some("Elapsed 2:00".into()));
    assert_eq!(
        active.label("chat", "en", now + Duration::from_secs(7)),
        Some("Elapsed 2:07 · No new state for 0:07".into())
    );
    active.observe_snapshot(&running, now + Duration::from_secs(7), 128_000);
    assert_eq!(
        active.label("chat", "en", now + Duration::from_secs(7)),
        Some("Elapsed 2:07".into())
    );
}

#[test]
fn terminal_or_unrelated_jobs_clear_the_active_progress_label() {
    let now = Instant::now();
    let mut active = ActiveWork::default();
    let snapshot = |kind: &str, state: &str| serde_json::json!({"revision":"r1","job":{"kind":kind,"id":"job-1","state":state,"started_ms":1}});
    active.observe_snapshot(&snapshot("optimize", "running"), now, 1_001);
    assert!(active.is_active());
    active.observe_snapshot(&snapshot("plan", "running"), now, 1_001);
    assert!(!active.is_active());
    active.observe_snapshot(&snapshot("mcp", "running"), now, 1_001);
    active.observe_snapshot(
        &snapshot("mcp", "completed"),
        now + Duration::from_secs(1),
        2_001,
    );
    assert!(!active.is_active());
}

#[test]
fn chat_and_mcp_surfaces_render_the_active_wait_label() {
    let mut chat = crate::native_ui_probe::workspace::chat::State::default();
    chat.open();
    chat.started("chat-1".into());
    let context = egui::Context::default();
    let input = progress_input();
    let mut output = context.run_ui(input.clone(), |ui| {
        chat.show_with_progress("en", Some("Elapsed 2:00 · No new state for 0:08"), ui);
    });
    assert!(painted_text(&output).contains("Elapsed 2:00 · No new state for 0:08"));
    clear_textures(&mut output);

    let snapshot = serde_json::json!({"revision":"r1","config":{"goal":"Implement jira:EMM-52599","coordinator":"claude","profiles":[{"id":"claude"}],"mcp":{"company":{"type":"local","command":["company-mcp"]}}}});
    let mut editor = crate::native_ui_probe::workspace::mcp::State::default();
    editor.open(Some(&snapshot));
    let context = egui::Context::default();
    let mut warmup = context.run_ui(input.clone(), |ui| {
        editor.show_with_progress(Some(&snapshot), "en", true, Some("Elapsed 1:15"), ui);
    });
    clear_textures(&mut warmup);
    let mut output = context.run_ui(input.clone(), |ui| {
        editor.show_with_progress(Some(&snapshot), "en", true, Some("Elapsed 1:15"), ui);
    });
    assert!(painted_text(&output).contains("Elapsed 1:15"));
    clear_textures(&mut output);
    let mut source = crate::native_ui_probe::workspace::mcp::SourceImport::default();
    let mut goal = snapshot["config"]["goal"].as_str().unwrap().to_owned();
    let mut output = context.run_ui(input, |ui| {
        source.show_with_progress(
            &mut goal,
            Some(&snapshot),
            "en",
            true,
            Some("Elapsed 1:15"),
            ui,
        );
    });
    assert!(painted_text(&output).contains("Elapsed 1:15"));
    clear_textures(&mut output);
}

fn clear_textures(output: &mut egui::FullOutput) {
    output.textures_delta.set.clear();
    output.textures_delta.free.clear();
}

fn painted_text(output: &egui::FullOutput) -> String {
    fn append(shape: &egui::Shape, output: &mut Vec<String>) {
        match shape {
            egui::Shape::Text(text) => output.push(text.galley.job.text.clone()),
            egui::Shape::Vec(shapes) => shapes.iter().for_each(|shape| append(shape, output)),
            _ => {}
        }
    }
    let mut text = Vec::new();
    output
        .shapes
        .iter()
        .for_each(|shape| append(&shape.shape, &mut text));
    text.join("\n")
}

fn progress_input() -> egui::RawInput {
    egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 600.0),
        )),
        ..Default::default()
    }
}
