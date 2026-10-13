//! Exact reset dates from the supported Claude screen-reader timezone forms.
use crate::availability::parse_time;
fn date(at: i64) -> Option<(i64, i64, i64)> {
    if !(0..=253402300799).contains(&at) {
        return None;
    }
    let mut days = at / 86400;
    let mut year = 1970;
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year += 1;
    }
    let lengths = [
        31,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 0;
    while days >= lengths[month] {
        days -= lengths[month];
        month += 1;
    }
    Some((year, month as i64 + 1, days + 1))
}
fn leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}
fn offset(zone: &str) -> Option<i64> {
    match zone {
        "Asia/Seoul" | "Asia/Tokyo" => Some(9 * 3600),
        "UTC" | "GMT" | "Etc/UTC" => Some(0),
        _ => {
            let zone = zone
                .strip_prefix("UTC")
                .or_else(|| zone.strip_prefix("GMT"))?;
            let sign = if zone.starts_with('+') {
                1
            } else if zone.starts_with('-') {
                -1
            } else {
                return None;
            };
            let (h, m) = zone[1..].split_once(':')?;
            let (h, m) = (h.parse::<i64>().ok()?, m.parse::<i64>().ok()?);
            ((0..=23).contains(&h) && (0..=59).contains(&m)).then_some(sign * (h * 3600 + m * 60))
        }
    }
}
fn clock(value: &str) -> Option<(i64, i64)> {
    let lower = value.to_ascii_lowercase();
    let (value, ampm) = if let Some(v) = lower.strip_suffix("am") {
        (v, Some(false))
    } else if let Some(v) = lower.strip_suffix("pm") {
        (v, Some(true))
    } else {
        (lower.as_str(), None)
    };
    let (h, m) = value.split_once(':').unwrap_or((value, "0"));
    let (mut h, m) = (h.parse::<i64>().ok()?, m.parse::<i64>().ok()?);
    if let Some(pm) = ampm {
        if !(1..=12).contains(&h) {
            return None;
        }
        h = h % 12 + if pm { 12 } else { 0 };
    }
    if !(0..=23).contains(&h) || !(0..=59).contains(&m) {
        return None;
    }
    Some((h, m))
}
pub(super) fn parse(line: &str, now: i64, minutes: u64) -> Option<i64> {
    let text = line.strip_prefix("Resets ")?.trim();
    if let Ok(at) = parse_time(text) {
        return (at > now && at - now <= minutes as i64 * 60).then_some(at);
    }
    let (text, zone) = text.rsplit_once(" (")?;
    let zone = zone.strip_suffix(')')?;
    let offset = offset(zone)?;
    let local = now.checked_add(offset)?;
    let (mut year, month, day) = date(local)?;
    let text = text.replace(',', " ").replace(" at ", " ");
    let tokens: Vec<_> = text.split_whitespace().collect();
    let (month, day, time, explicit) = if tokens.len() == 1 {
        (month, day, tokens[0], false)
    } else {
        let month_name = tokens.first()?.to_ascii_lowercase();
        let month = [
            "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
        ]
        .iter()
        .position(|m| month_name.starts_with(*m))
        .map(|m| m as i64 + 1)?;
        let day = tokens.get(1)?.parse::<i64>().ok()?;
        if tokens.len() == 4 {
            year = tokens[2].parse().ok()?;
        }
        (month, day, *tokens.last()?, true)
    };
    let (hour, minute) = clock(time)?;
    let stamp = |y| {
        parse_time(&format!(
            "{y:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:00Z"
        ))
        .ok()?
        .checked_sub(offset)
    };
    let mut at = stamp(year)?;
    if at <= now {
        if !explicit {
            at = at.checked_add(86400)?;
        } else if tokens.len() != 4 {
            at = stamp(year + 1)?;
        } else {
            return None;
        }
    }
    (at > now && at - now <= minutes as i64 * 60).then_some(at)
}
