//! #429: records the actual planner argv, then delegates to the deterministic CLI fixture.
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::fs::write(std::env::var_os("AC_CAPTURE_ARGS").unwrap(), args.join("\n")).unwrap();
    let status = std::process::Command::new(std::env::var_os("AC_DELEGATE").unwrap())
        .args(&args)
        .status()
        .unwrap();
    std::process::exit(status.code().unwrap_or(1));
}
