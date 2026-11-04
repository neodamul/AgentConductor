//! #472: keep process ownership and supervision independently reviewable.
#[test]
fn process_modules_stay_within_the_three_hundred_line_budget() {
    for entry in std::iter::once(std::path::PathBuf::from("src/process.rs"))
        .chain(
            std::fs::read_dir("src/process")
                .unwrap()
                .map(|entry| entry.unwrap().path()),
        )
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
    {
        let lines = std::fs::read_to_string(&entry).unwrap().lines().count();
        assert!(
            lines <= 300,
            "{}: {lines} lines exceeds 300",
            entry.display()
        );
    }
}
