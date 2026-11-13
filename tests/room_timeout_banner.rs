const RECOVERY: &str = include_str!("../src/ui/room_recovery.js");
#[test]
fn blocked_report_error_reaches_the_task_card_without_a_run_banner() {
    assert!(!include_str!("../src/ui/room.html").contains("id=\"run-banner\""));
    assert!(
        RECOVERY
            .contains("failure=r.error||r.tasks.find(t=>t.status==='failed'&&t.error)?.error||''")
    );
    assert!(RECOVERY.contains("AC_I18N.bind($('run-error'),()=>why);$('run-error').hidden=!why"));
}
