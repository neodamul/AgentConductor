//! Local reset and observation times shown on quota badges and room labels.

use super::msg;

pub(super) fn local_time(timestamp: i64, now: i64) -> String {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    let Ok(value) = time::OffsetDateTime::from_unix_timestamp(timestamp) else {
        return timestamp.to_string();
    };
    let value = value.to_offset(offset);
    if let Ok(today) = time::OffsetDateTime::from_unix_timestamp(now) {
        let today = today.to_offset(offset);
        if value.year() == today.year()
            && value.month() == today.month()
            && value.day() == today.day()
        {
            return format!("{:02}:{:02}", value.hour(), value.minute());
        }
    }
    format!(
        "{}/{:02} {:02}:{:02}",
        u8::from(value.month()),
        value.day(),
        value.hour(),
        value.minute()
    )
}

pub(super) fn local_datetime(timestamp: i64) -> String {
    let offset = time::UtcOffset::current_local_offset().unwrap_or(time::UtcOffset::UTC);
    time::OffsetDateTime::from_unix_timestamp(timestamp)
        .map(|value| {
            let value = value.to_offset(offset);
            format!(
                "{}-{:02}-{:02} {:02}:{:02}:{:02}",
                value.year(),
                u8::from(value.month()),
                value.day(),
                value.hour(),
                value.minute(),
                value.second()
            )
        })
        .unwrap_or_else(|_| timestamp.to_string())
}

pub(super) fn room_reset_label(timestamp: i64, language: &str) -> String {
    format!(
        "{}{}",
        msg(language, "room.resets"),
        local_datetime(timestamp)
    )
}
