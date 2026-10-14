use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Restriction {
    pub available_at: Option<i64>,
    pub observed_at: i64,
    pub reason: String,
    pub source: String,
}

/// Deliberately accepts explicit dates with seconds and a UTC offset only.
/// Localized dates and phrases such as "tomorrow at 2pm" need human scheduling.
pub fn parse_time(text: &str) -> Result<i64, String> {
    let bytes = text.as_bytes();
    if !text.is_ascii()
        || !matches!(bytes.len(), 20 | 25)
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return Err("use YYYY-MM-DDTHH:MM:SSZ or YYYY-MM-DDTHH:MM:SS+HH:MM".into());
    }
    let number = |a: usize, b: usize| -> Result<i64, String> {
        if !bytes[a..b].iter().all(u8::is_ascii_digit) {
            return Err("invalid timestamp".into());
        }
        text[a..b].parse().map_err(|_| "invalid timestamp".into())
    };
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = [
        31,
        if leap { 29 } else { 28 },
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
    if !(1970..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day < 1
        || day > days[(month - 1) as usize]
        || hour > 23
        || minute > 59
        || second > 59
    {
        return Err("invalid timestamp calendar value".into());
    }
    let offset = if bytes.len() == 20 {
        if bytes[19] != b'Z' {
            return Err("explicit timezone required".into());
        }
        0
    } else {
        if !matches!(bytes[19], b'+' | b'-') || bytes[22] != b':' {
            return Err("invalid timezone".into());
        }
        let h = number(20, 22)?;
        let m = number(23, 25)?;
        if h > 23 || m > 59 {
            return Err("invalid timezone".into());
        }
        (h * 3600 + m * 60) * if bytes[19] == b'+' { 1 } else { -1 }
    };
    let mut elapsed_days = 0;
    for y in 1970..year {
        elapsed_days += if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) {
            366
        } else {
            365
        };
    }
    elapsed_days += days[..(month - 1) as usize].iter().sum::<i64>() + day - 1;
    let timestamp = elapsed_days * 86400 + hour * 3600 + minute * 60 + second - offset;
    if timestamp < 0 {
        return Err("timestamp predates Unix epoch".into());
    }
    Ok(timestamp)
}

/// Seconds before a run retries a model that reported it is at capacity.
pub const TRANSIENT_RETRY_SECONDS: i64 = 60;

/// "Selected model is at capacity. Please try a different model" and similar replies are
/// momentary (user decision 2026-10-14): retried shortly, at most three times.
pub fn transient_capacity(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    [
        "at capacity",
        "try a different model",
        "model is overloaded",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
}

pub fn detect(payload: &Value, message: &str, observed_at: i64) -> Option<Restriction> {
    let evidence = format!("{payload} {message}");
    let lower = evidence.to_ascii_lowercase();
    if transient_capacity(&evidence) {
        return Some(Restriction {
            available_at: Some(observed_at + TRANSIENT_RETRY_SECONDS),
            observed_at,
            reason: message.chars().take(2048).collect(),
            source: "transient_capacity".into(),
        });
    }
    let limited = [
        "rate_limit",
        "rate limit",
        "usage_limit",
        "usage limit",
        "hit your limit",
        "quota exceeded",
        "overloaded",
        "temporarily unavailable",
        "service unavailable",
        "too many requests",
        "out of extra usage",
        "weekly limit",
        "daily limit",
    ]
    .iter()
    .any(|phrase| lower.contains(phrase))
        || temporary_status(payload);
    if !limited {
        return None;
    }
    fn collect(value: &Value, now: i64, times: &mut Vec<i64>) {
        match value {
            Value::Object(fields) => {
                if fields
                    .get("usedPercent")
                    .and_then(Value::as_f64)
                    .is_some_and(|n| n < 100.0)
                {
                    return;
                }
                for (key, value) in fields {
                    if matches!(
                        key.as_str(),
                        "resetsAt" | "resets_at" | "reset_at" | "available_at" | "retry_at"
                    ) {
                        if let Some(t) = value
                            .as_i64()
                            .filter(|t| *t > 0)
                            .or_else(|| value.as_str().and_then(|s| parse_time(s).ok()))
                        {
                            times.push(t);
                        }
                    } else if matches!(key.as_str(), "retry_after" | "retryAfterSeconds") {
                        if let Some(t) = value
                            .as_i64()
                            .filter(|t| *t >= 0)
                            .and_then(|t| now.checked_add(t))
                        {
                            times.push(t);
                        }
                    } else {
                        collect(value, now, times);
                    }
                }
            }
            Value::Array(values) => {
                for value in values {
                    collect(value, now, times);
                }
            }
            _ => {}
        }
    }
    let mut times = vec![];
    collect(payload, observed_at, &mut times);
    let mut source = "structured_reset";
    if times.is_empty() {
        source = "explicit_message_time";
        for token in evidence.split_whitespace() {
            if let Ok(time) = parse_time(token.trim_matches(|c: char| {
                matches!(
                    c,
                    '"' | ',' | '.' | ';' | '(' | ')' | '\\' | '{' | '}' | '[' | ']'
                )
            })) {
                times.push(time);
            }
        }
    }
    let available_at = times.into_iter().max();
    Some(Restriction {
        available_at,
        observed_at,
        reason: if payload["api_error"] == "usage_limit_reached" {
            "room.account_quota_exhausted".into()
        } else {
            message.chars().take(2048).collect()
        },
        source: if available_at.is_some() {
            source
        } else {
            "manual_required"
        }
        .into(),
    })
}

fn temporary_status(value: &Value) -> bool {
    match value {
        Value::Object(fields) => fields.iter().any(|(key, value)| {
            (matches!(
                key.as_str(),
                "status" | "status_code" | "api_error_status" | "code"
            ) && matches!(value.as_i64(), Some(429 | 503)))
                || temporary_status(value)
        }),
        Value::Array(values) => values.iter().any(temporary_status),
        _ => false,
    }
}
