//! GitHub #82: reset times resolve for the IANA zones Claude prints, including daylight saving.
use agentconductor::quota;

fn reset(zone_line: &str, now: i64) -> Option<i64> {
    let text = format!("Current session\n10% used\nResets {zone_line}\n");
    quota::parse_claude_usage(&text, now).ok()?.windows[0].resets_at
}

#[test]
fn zones_with_daylight_saving_use_the_offset_in_force_at_the_reset() {
    // Los Angeles, PDT (UTC-7) in October.
    assert_eq!(
        reset("5pm (America/Los_Angeles)", 1792526400),
        Some(1792540800)
    );
    // New York, EST (UTC-5) in December.
    assert_eq!(
        reset("5pm (America/New_York)", 1796860800 + 19 * 3600),
        Some(1796940000)
    );
    // Berlin, CEST (UTC+2) in July.
    assert_eq!(
        reset("9am (Europe/Berlin)", 1782864000 + 4 * 3600),
        Some(1782889200)
    );
    // Sydney, AEDT (UTC+11) in January.
    assert_eq!(
        reset("9am (Australia/Sydney)", 1799884800 + 20 * 3600),
        Some(1799964000)
    );
}

#[test]
fn fixed_offset_zones_and_the_existing_ones_still_resolve() {
    // Kolkata is UTC+5:30 all year: 11am IST is 05:30 UTC. The session window is five hours, so now is early.
    assert_eq!(
        reset("11am (Asia/Kolkata)", 1793491200 + 3600),
        Some(1793491200 + 5 * 3600 + 1800)
    );
    assert!(reset("11am (Asia/Seoul)", 1793491200 + 3600).is_some());
    assert_eq!(reset("5pm (Nowhere/Land)", 1793491200), None);
}

