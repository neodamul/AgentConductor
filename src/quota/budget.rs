use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Budget {
    pub bucket: String,
    pub scope: String,
    pub remaining: Option<String>,
    pub allocated: Option<String>,
    pub used: Option<String>,
    pub remaining_percent: Option<f64>,
    pub unlimited: bool,
    pub resets_at: Option<i64>,
}
impl Budget {
    pub fn available(&self) -> bool {
        self.remaining.is_some() || self.remaining_percent.is_some() || self.unlimited
    }
}
fn amount(value: &Value) -> Result<String, String> {
    let text = value.as_str().ok_or("invalid budget amount")?;
    decimal(text).ok_or("invalid budget amount")?;
    Ok(text.to_owned())
}
// Exact, bounded decimal quantities; no floating-point financial subtraction.
fn decimal(text: &str) -> Option<(i128, u32)> {
    if text.len() > 64 {
        return None;
    }
    let mut parts = text.split('.');
    let whole = parts.next()?;
    let fraction = parts.next().unwrap_or("");
    if parts.next().is_some()
        || whole.is_empty()
        || whole.len() > 18
        || fraction.len() > 18
        || !whole.bytes().all(|b| b.is_ascii_digit())
        || !fraction.bytes().all(|b| b.is_ascii_digit())
    {
        return None;
    }
    let scale = fraction.len() as u32;
    let units = whole
        .parse::<i128>()
        .ok()?
        .checked_mul(10i128.pow(scale))?
        .checked_add(if fraction.is_empty() {
            0
        } else {
            fraction.parse::<i128>().ok()?
        })?;
    Some((units, scale))
}
fn remaining(limit: &str, used: &str) -> Option<String> {
    let (a, sa) = decimal(limit)?;
    let (b, sb) = decimal(used)?;
    let scale = sa.max(sb);
    let value = a
        .checked_mul(10i128.pow(scale - sa))?
        .checked_sub(b.checked_mul(10i128.pow(scale - sb))?)?
        .max(0);
    if scale == 0 {
        return Some(value.to_string());
    }
    let divisor = 10i128.pow(scale);
    let text = format!(
        "{}.{:0width$}",
        value / divisor,
        value % divisor,
        width = scale as usize
    );
    Some(text.trim_end_matches('0').trim_end_matches('.').to_owned())
}
pub fn parse(bucket: &str, value: &Value) -> Result<Vec<Budget>, String> {
    let mut out = Vec::new();
    if let Some(v) = value.get("individualLimit").filter(|v| !v.is_null()) {
        let allocated = amount(&v["limit"])?;
        let used = amount(&v["used"])?;
        let percent = v["remainingPercent"]
            .as_f64()
            .filter(|p| p.is_finite() && (0.0..=100.0).contains(p))
            .ok_or("invalid budget remaining percentage")?;
        let resets_at = v["resetsAt"]
            .as_i64()
            .filter(|t| *t > 0)
            .ok_or("invalid budget reset timestamp")?;
        out.push(Budget {
            bucket: bucket.into(),
            scope: "individual".into(),
            remaining: remaining(&allocated, &used),
            allocated: Some(allocated),
            used: Some(used),
            remaining_percent: Some(percent),
            unlimited: false,
            resets_at: Some(resets_at),
        });
    }
    if let Some(v) = value.get("credits").filter(|v| !v.is_null()) {
        let unlimited = v["unlimited"]
            .as_bool()
            .ok_or("invalid credit limit flag")?;
        v["hasCredits"]
            .as_bool()
            .ok_or("invalid credit availability flag")?;
        let balance = v
            .get("balance")
            .filter(|v| !v.is_null())
            .map(amount)
            .transpose()?;
        out.push(Budget {
            bucket: bucket.into(),
            scope: "credits".into(),
            remaining: balance,
            allocated: None,
            used: None,
            remaining_percent: None,
            unlimited,
            resets_at: None,
        });
    }
    Ok(out)
}
