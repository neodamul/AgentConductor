use std::io::Read;
fn main() {
    let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
    let task=prompt.split_once("\n# Task\n").map_or(prompt.as_str(),|(_,task)|task).split("\n# Format\n").next().unwrap();
    let path=task.lines().find(|line|!line.trim().is_empty()).unwrap().trim();
    let now=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    println!("{{\"type\":\"system\",\"subtype\":\"init\",\"session_id\":\"quota-fixture-{path}\"}}");
    println!("{{\"type\":\"rate_limit_event\",\"session_id\":\"quota-fixture-{path}\",\"rate_limit_info\":{{\"status\":\"allowed\",\"rateLimitType\":\"seven_day_sonnet\",\"utilization\":0.27,\"resetsAt\":{}}}}}",now+3600);
    std::fs::write(path,"implemented\n").unwrap();
    println!("{{\"type\":\"result\",\"session_id\":\"quota-fixture-{path}\",\"is_error\":false}}");
}
