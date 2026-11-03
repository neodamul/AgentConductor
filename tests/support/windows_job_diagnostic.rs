pub fn host_job_breakaway_denied(screen: &str) -> bool {
    let screen = screen.to_ascii_lowercase();
    screen.contains("windows denied process breakaway")
        && screen.contains("access is denied. (os error 5)")
        && screen.contains("this app is inside a host job")
}

#[cfg(test)]
mod tests {
    use super::host_job_breakaway_denied;

    #[test]
    fn only_explicit_access_denied_host_job_diagnostics_are_skippable() {
        assert!(host_job_breakaway_denied(
            "Windows denied process breakaway (Access is denied. (os error 5)); this app is inside a host job"
        ));
        for unrelated in [
            "Windows denied process breakaway (Access is denied. (os error 5)); host-job membership could not be checked",
            "Windows denied process breakaway (Access is denied. (os error 5)); no host job detail",
            "quota query failed: host job unavailable",
        ] {
            assert!(!host_job_breakaway_denied(unrelated), "{unrelated}");
        }
    }
}
