//! DD-229: the event catalogue is the single source of event ids and texts; the texts are unchanged.
use agentconductor::messenger::{self, events};

const TEXTS: [(&str, &str, &str); 9] = [
    ("plan_ready", "plan ready for review", "계획 검토 준비 완료"),
    (
        "waiting_approval",
        "candidate ready for review",
        "후보 검토 대기",
    ),
    (
        "waiting_tool_approval",
        "tool request needs a decision",
        "도구 요청 결정 필요",
    ),
    (
        "waiting_availability",
        "waiting for account availability",
        "계정 사용 한도로 대기 중",
    ),
    ("completed", "run completed", "실행 완료"),
    ("failed", "run failed", "실행 실패"),
    ("stopped", "run stopped", "실행 중지"),
    ("blocked", "run blocked", "실행 차단"),
    ("test", "test message", "테스트 메시지"),
];

#[test]
fn every_event_keeps_its_english_and_korean_text() {
    for (id, en, ko) in TEXTS {
        assert_eq!(events::text(id, false), Some(en), "{id} in English");
        assert_eq!(events::text(id, true), Some(ko), "{id} in Korean");
    }
    assert_eq!(
        events::text("running", false),
        None,
        "an unlisted state is not a messenger event"
    );
}

#[test]
fn the_dialog_list_is_the_catalogue_and_only_tool_requests_are_actionable() {
    let ids: Vec<_> = events::CATALOG.iter().map(|d| d.id).collect();
    assert_eq!(ids, messenger::EVENTS);
    let actionable: Vec<_> = events::CATALOG
        .iter()
        .filter(|d| d.actionable)
        .map(|d| d.id)
        .collect();
    assert_eq!(actionable, ["waiting_tool_approval"]);
    for def in events::CATALOG {
        assert!(def.message.starts_with("messenger.event."), "{}", def.id);
        assert!(!def.source.is_empty());
    }
    assert_eq!(
        events::find("waiting_tool_approval").unwrap().detail,
        events::Detail::Tool
    );
}
