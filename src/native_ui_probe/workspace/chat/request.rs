pub fn request(snapshot: &serde_json::Value, question: &str) -> Result<Vec<u8>, String> {
    let revision = snapshot["revision"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Workspace revision is unavailable".to_owned())?;
    let language = if snapshot["language"] == "ko" {
        "Korean"
    } else {
        "English"
    };
    crate::chat::build_prompt(
        &[crate::chat::Turn {
            role: "operator".into(),
            text: question.into(),
        }],
        language,
    )?;
    let packet = serde_json::to_vec(&serde_json::json!({
        "kind":"chat", "revision":revision, "question":question
    }))
    .map_err(|error| error.to_string())?;
    if packet.len() > 128 * 1024 {
        return Err(crate::ui::messages::text("native.native_request_exceeds_128_kib").into());
    }
    Ok(packet)
}
