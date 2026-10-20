//! GitHub #81: only an explicit command is a decision; prose around it is not.
use agentconductor::messenger::inbound;

#[test]
fn negated_or_conversational_replies_are_not_decisions() {
    for text in [
        "don't allow 3FA9C21B",
        "I would not approve 3FA9C21B",
        "deny? no, allow 3FA9C21B",
        "please allow 3FA9C21B",
        "never deny 3FA9C21B",
    ] {
        assert_eq!(inbound::parse(text), None, "{text}");
    }
}

#[test]
fn explicit_commands_still_parse() {
    for (text, allow) in [
        ("allow 3fa9c21b", true),
        ("<@U9> Deny `3FA9C21B`.", false),
        ("@bot approve 3FA9C21B", true),
        ("  reject 3FA9C21B  ", false),
        ("*allow* 3FA9C21B", true),
    ] {
        assert_eq!(
            inbound::parse(text),
            Some((allow, "3FA9C21B".into())),
            "{text}"
        );
    }
}
