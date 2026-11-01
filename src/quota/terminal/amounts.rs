pub(super) fn thousands(amount: &str) -> String {
    let (whole, fraction) = amount.split_once('.').unwrap_or((amount, ""));
    let mut groups = whole.split(',');
    let grouped = groups
        .next()
        .is_some_and(|group| (1..=3).contains(&group.len()))
        && groups.all(|group| group.len() == 3 && group.bytes().all(|byte| byte.is_ascii_digit()));
    if !whole.contains(',') || !grouped {
        return amount.to_owned();
    }
    let digits = whole.replace(',', "");
    if fraction.is_empty() && !amount.contains('.') {
        digits
    } else {
        format!("{digits}.{fraction}")
    }
}
