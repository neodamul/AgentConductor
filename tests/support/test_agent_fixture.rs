#[path = "test_agent_fixture/claude.rs"]
mod claude;
#[path = "test_agent_fixture/cooperative.rs"]
mod cooperative;
#[path = "test_agent_fixture/coordinator.rs"]
mod coordinator;
#[path = "test_agent_fixture/mcp.rs"]
mod mcp;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("mcp-server") => mcp::server(&args[1..]),
        Some("mcp-leak") => mcp::leak(&args[1..]),
        Some("mcp-coordinator") => coordinator::run(&args[1..]),
        Some("claude-api-worker") => claude::api_worker(),
        Some("cooperative-worker") => cooperative::run(&args[1..]),
        mode => panic!("unknown fixture mode: {mode:?}"),
    }
}
