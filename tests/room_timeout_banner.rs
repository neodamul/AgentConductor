const ROOM: &str = include_str!("../src/ui/room.js");

#[test]
fn blocked_report_error_reaches_the_existing_run_banner() {
    assert!(
        ROOM.contains("failure=r.error||r.tasks.find(t=>t.status==='failed'&&t.error)?.error||''")
    );
    assert!(
        ROOM.contains("failure?acMessage(\"room.last_error\")+String(failure).slice(0,300):''")
    );
    assert!(ROOM.contains(
        "AC_I18N.bind($('run-banner-error'),()=>(why));$('run-banner-error').hidden=!why"
    ));
}
