use agentconductor::ui::{
    messages,
    update_guard::{Blocker, job_label, message},
};

const IDS: [&str; 15] = [
    "native.startup_settings_poisoned",
    "native.update_blocked_active",
    "native.update_blocked_advice",
    "native.update_blocker_line",
    "native.update_check_advice",
    "native.update_check_failed",
    "native.update_check_line",
    "native.update_elapsed",
    "native.update_job_catalog",
    "native.update_job_chat",
    "native.update_job_other",
    "native.update_job_plan",
    "native.update_job_quota",
    "native.update_job_run",
    "native.update_job_start",
];

#[test]
fn update_blocker_messages_exist_in_native_english_and_korean_catalogs() {
    for language in ["en", "ko"] {
        for id in IDS {
            assert_ne!(
                messages::in_language(language, id),
                id,
                "missing {language} native message: {id}"
            );
        }
    }
}

#[test]
fn refusal_keeps_active_work_separate_from_failed_checks_in_both_locales() {
    for language in ["en", "ko"] {
        let text = message(
            language,
            &[
                Blocker::Active {
                    workspace: "Operations".into(),
                    what: job_label(language, "quota:claude"),
                    elapsed_ms: Some(65_000),
                },
                Blocker::Unknown {
                    what: "workspace list".into(),
                    error: "access denied".into(),
                },
            ],
        );
        assert!(!text.contains("native."), "{language}: {text}");
        assert!(
            text.contains("Operations") && text.contains("1:05"),
            "{text}"
        );
        assert!(text.contains("access denied"), "{text}");
    }
}
