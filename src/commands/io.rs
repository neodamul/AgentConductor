//! `agentconductor io schema|validate`: the acio protocol's schemas and checker (GitHub #207).
use std::io::Read;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [io, command, kind] if io == "io" && command == "schema" => {
            Some(agentconductor::cli_io::schema(kind).map(|text| print!("{text}")))
        }
        [io, command, kind, source, rest @ ..] if io == "io" && command == "validate" => {
            Some(validate(kind, source, rest))
        }
        _ => None,
    }
}

fn validate(kind: &str, source: &str, flags: &[String]) -> Result<(), String> {
    let coordinator = match flags {
        [] => false,
        [flag] if flag == "--coordinator" => true,
        _ => return Err("usage: io validate <kind> <file|-> [--coordinator]".into()),
    };
    let mut text = String::new();
    let limit = 16 * 1024 * 1024;
    if source == "-" {
        std::io::stdin().take(limit).read_to_string(&mut text)
    } else {
        std::fs::File::open(source).and_then(|f| f.take(limit).read_to_string(&mut text))
    }
    .map_err(|e| format!("read {source}: {e}"))?;
    if matches!(kind, "events" | "controls") {
        agentconductor::cli_io::validate_stream(
            kind,
            &text,
            agentconductor::cli_io::Rules { coordinator },
        )?;
    } else {
        let document = serde_json::from_str(&text).map_err(|e| format!("not JSON: {e}"))?;
        agentconductor::cli_io::validate(kind, &document).map_err(|errors| errors.join("\n"))?;
    }
    println!("ok");
    Ok(())
}
