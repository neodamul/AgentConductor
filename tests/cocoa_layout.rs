//! #472: keep Cocoa native modules within the review budget.
#[test]
fn cocoa_modules_stay_within_three_hundred_lines() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let facade = root.join("src/ui/native/cocoa.rs");
    let children = std::fs::read_dir(root.join("src/ui/native/cocoa")).unwrap();
    for path in std::iter::once(facade).chain(children.map(|entry| entry.unwrap().path())) {
        if path.extension().is_some_and(|extension| extension == "rs") {
            let lines = std::fs::read_to_string(&path).unwrap().lines().count();
            assert!(
                lines <= 300,
                "{}: {lines} lines exceeds 300",
                path.display()
            );
        }
    }
}
