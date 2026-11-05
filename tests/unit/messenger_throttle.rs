use super::{Admit, admit};

fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("ac-throttle-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn ten_a_minute_then_fold_then_one_summary() {
    let d = dir("fold");
    for _ in 0..10 {
        assert_eq!(admit(&d, 1000, false), Admit::Send);
    }
    assert_eq!(admit(&d, 1010, false), Admit::Fold);
    assert_eq!(admit(&d, 1020, false), Admit::Fold);
    // The window slides: the first ten fell out, so the next notice goes out after a summary of two.
    assert_eq!(admit(&d, 1061, false), Admit::SendAfterSummary(2));
    assert_eq!(
        admit(&d, 1062, false),
        Admit::Send,
        "the summary is sent once"
    );
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn notices_with_a_code_are_never_counted_or_folded() {
    let d = dir("actionable");
    for _ in 0..10 {
        assert_eq!(admit(&d, 1000, false), Admit::Send);
    }
    for _ in 0..30 {
        assert_eq!(admit(&d, 1001, true), Admit::Send);
    }
    assert_eq!(
        admit(&d, 1002, false),
        Admit::Fold,
        "the limit still holds for the others"
    );
    std::fs::remove_dir_all(d).unwrap();
}

#[test]
fn an_unwritable_folder_never_blocks_a_notice() {
    let gone = std::env::temp_dir().join("ac-throttle-missing-folder/nested");
    assert_eq!(admit(&gone, 5, false), Admit::Send);
}
