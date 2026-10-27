use super::*;

pub(super) fn execute(args: &[String]) -> Option<Result<(), String>> {
    match args {
        [command, dir, message] if command == "coordinator-request" => Some((|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let proposal =
                runtime.block_on(workflow::coordinator_request(Path::new(dir), 1800, message))?;
            println!(
                "{}",
                serde_json::to_string(&proposal).map_err(|e| e.to_string())?
            );
            Ok(())
        })()),
        [command, dir, coordinator] if command == "review" || command == "review-apply" => {
            Some((|| {
                let runtime = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build()
                    .map_err(|e| e.to_string())?;
                let proposal = runtime.block_on(workflow::review(
                    Path::new(dir),
                    coordinator,
                    command == "review-apply",
                ))?;
                println!(
                    "{}",
                    serde_json::json!({"proposal_sha256":workflow::digest(&serde_json::to_vec(&proposal).map_err(|e|e.to_string())?),"proposal":proposal,"round":workflow::load(Path::new(dir))?.review_round})
                );
                Ok(())
            })())
        }
        [command, dir, proposal] if command == "apply-review" => Some((|| {
            workflow::apply_review(Path::new(dir), proposal)?;
            print_report(&workflow::load(Path::new(dir))?)
        })()),
        [command, dir, task, feedback] if command == "revise" => Some((|| {
            workflow::revise(Path::new(dir), task, feedback)?;
            print_report(&workflow::load(Path::new(dir))?)
        })()),
        [command, dir] if command == "recover" => {
            Some((|| print_report(&workflow::recover(Path::new(dir))?))())
        }
        [command, dir, reason] if command == "stop" => Some((|| {
            workflow::stop(Path::new(dir), reason)?;
            println!("Stop request retained; inspect status until the run is stopped.");
            Ok(())
        })()),
        [command, dir] if command == "resume" => Some((|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .map_err(|e| e.to_string())?;
            let report = runtime.block_on(workflow::resume(Path::new(dir), 1800))?;
            print_report(&runtime.block_on(workflow::repair(Path::new(dir), 1800, report))?)
        })()),
        _ => None,
    }
}
