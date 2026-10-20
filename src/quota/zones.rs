//! UTC offsets for the IANA zones Claude prints, including daylight saving (GitHub #82).
//! A built-in table keeps this identical on macOS and Windows, which has no tz database.
//! shortcut: common zones only, current rules only; add a zone here when one is missing.

#[derive(Clone, Copy)]
enum Rule {
    None,
    Us,
    Eu,
    AuSouth,
    Nz,
}

const H: i64 = 3600;
const ZONES: &[(&str, i64, Rule)] = &[
    ("America/New_York", -5 * H, Rule::Us),
    ("America/Toronto", -5 * H, Rule::Us),
    ("America/Chicago", -6 * H, Rule::Us),
    ("America/Denver", -7 * H, Rule::Us),
    ("America/Los_Angeles", -8 * H, Rule::Us),
    ("America/Vancouver", -8 * H, Rule::Us),
    ("America/Anchorage", -9 * H, Rule::Us),
    ("America/Halifax", -4 * H, Rule::Us),
    ("America/Phoenix", -7 * H, Rule::None),
    ("Pacific/Honolulu", -10 * H, Rule::None),
    ("America/Mexico_City", -6 * H, Rule::None),
    ("America/Bogota", -5 * H, Rule::None),
    ("America/Lima", -5 * H, Rule::None),
    ("America/Sao_Paulo", -3 * H, Rule::None),
    ("America/Argentina/Buenos_Aires", -3 * H, Rule::None),
    ("Europe/London", 0, Rule::Eu),
    ("Europe/Dublin", 0, Rule::Eu),
    ("Europe/Lisbon", 0, Rule::Eu),
    ("Europe/Paris", H, Rule::Eu),
    ("Europe/Berlin", H, Rule::Eu),
    ("Europe/Madrid", H, Rule::Eu),
    ("Europe/Rome", H, Rule::Eu),
    ("Europe/Amsterdam", H, Rule::Eu),
    ("Europe/Brussels", H, Rule::Eu),
    ("Europe/Vienna", H, Rule::Eu),
    ("Europe/Zurich", H, Rule::Eu),
    ("Europe/Stockholm", H, Rule::Eu),
    ("Europe/Oslo", H, Rule::Eu),
    ("Europe/Copenhagen", H, Rule::Eu),
    ("Europe/Warsaw", H, Rule::Eu),
    ("Europe/Prague", H, Rule::Eu),
    ("Europe/Budapest", H, Rule::Eu),
    ("Europe/Athens", 2 * H, Rule::Eu),
    ("Europe/Helsinki", 2 * H, Rule::Eu),
    ("Europe/Kyiv", 2 * H, Rule::Eu),
    ("Europe/Kiev", 2 * H, Rule::Eu),
    ("Europe/Bucharest", 2 * H, Rule::Eu),
    ("Europe/Istanbul", 3 * H, Rule::None),
    ("Europe/Moscow", 3 * H, Rule::None),
    ("Africa/Lagos", H, Rule::None),
    ("Africa/Johannesburg", 2 * H, Rule::None),
    ("Africa/Nairobi", 3 * H, Rule::None),
    ("Asia/Riyadh", 3 * H, Rule::None),
    ("Asia/Tehran", 3 * H + 1800, Rule::None),
    ("Asia/Dubai", 4 * H, Rule::None),
    ("Asia/Karachi", 5 * H, Rule::None),
    ("Asia/Kolkata", 5 * H + 1800, Rule::None),
    ("Asia/Calcutta", 5 * H + 1800, Rule::None),
    ("Asia/Kathmandu", 5 * H + 2700, Rule::None),
    ("Asia/Dhaka", 6 * H, Rule::None),
    ("Asia/Bangkok", 7 * H, Rule::None),
    ("Asia/Jakarta", 7 * H, Rule::None),
    ("Asia/Ho_Chi_Minh", 7 * H, Rule::None),
    ("Asia/Shanghai", 8 * H, Rule::None),
    ("Asia/Hong_Kong", 8 * H, Rule::None),
    ("Asia/Singapore", 8 * H, Rule::None),
    ("Asia/Taipei", 8 * H, Rule::None),
    ("Asia/Manila", 8 * H, Rule::None),
    ("Australia/Perth", 8 * H, Rule::None),
    ("Asia/Seoul", 9 * H, Rule::None),
    ("Asia/Tokyo", 9 * H, Rule::None),
    ("Australia/Adelaide", 9 * H + 1800, Rule::AuSouth),
    ("Australia/Brisbane", 10 * H, Rule::None),
    ("Australia/Sydney", 10 * H, Rule::AuSouth),
    ("Australia/Melbourne", 10 * H, Rule::AuSouth),
    ("Australia/Hobart", 10 * H, Rule::AuSouth),
    ("Pacific/Auckland", 12 * H, Rule::Nz),
];

/// Days since 1970-01-01 of a civil date.
fn days(year: i64, month: i64, day: i64) -> i64 {
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    era * 146_097 + yoe * 365 + yoe / 4 - yoe / 100 + doy - 719_468
}

/// The `n`th Sunday (1-based) of a month, or the last one for `n == 0`; as epoch seconds at 00:00 UTC.
fn sunday(year: i64, month: i64, n: i64) -> i64 {
    let weekday = |d: i64| (d + 4).rem_euclid(7); // 0 = Sunday
    let day = if n == 0 {
        let last = days(year + i64::from(month == 12), month % 12 + 1, 1) - 1;
        last - weekday(last)
    } else {
        let first = days(year, month, 1);
        first + (7 - weekday(first)) % 7 + (n - 1) * 7
    };
    day * 86_400
}

fn year_of(utc: i64) -> i64 {
    let mut year = 1970 + utc.div_euclid(31_556_952);
    while days(year, 1, 1) * 86_400 > utc {
        year -= 1;
    }
    while days(year + 1, 1, 1) * 86_400 <= utc {
        year += 1;
    }
    year
}

fn dst(rule: Rule, std: i64, utc: i64) -> bool {
    let year = year_of(utc + std);
    match rule {
        Rule::None => false,
        Rule::Us => {
            utc >= sunday(year, 3, 2) + 2 * H - std && utc < sunday(year, 11, 1) + 2 * H - std - H
        }
        Rule::Eu => utc >= sunday(year, 3, 0) + H && utc < sunday(year, 10, 0) + H,
        Rule::AuSouth => {
            utc >= sunday(year, 10, 1) + 2 * H - std || utc < sunday(year, 4, 1) + 2 * H - std
        }
        Rule::Nz => {
            utc >= sunday(year, 9, 0) + 2 * H - std || utc < sunday(year, 4, 1) + 2 * H - std
        }
    }
}

/// The zone's UTC offset in seconds at the UTC instant `utc`; None for an unknown zone.
pub(super) fn offset_at(zone: &str, utc: i64) -> Option<i64> {
    let &(_, std, rule) = ZONES.iter().find(|(name, ..)| *name == zone)?;
    Some(std + if dst(rule, std, utc) { H } else { 0 })
}

/// The zone's standard-time offset, used to find the instant a local time refers to.
pub(super) fn standard(zone: &str) -> Option<i64> {
    ZONES.iter().find(|(name, ..)| *name == zone).map(|z| z.1)
}
