use std::{fs, path::Path};

fn python_sources() -> Vec<std::path::PathBuf> {
    ["tests", "scripts"]
        .into_iter()
        .filter_map(|directory| fs::read_dir(directory).ok())
        .flat_map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .collect::<Vec<_>>()
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "py"))
        .collect()
}

#[test]
fn existing_python_text_io_declares_utf8() {
    let mut offenders = Vec::new();
    for path in python_sources() {
        let text = fs::read_to_string(&path).unwrap();
        for (line_number, line) in text.lines().enumerate() {
            let text_io = [".read_text(", ".write_text(", "open(", ".open("]
                .iter()
                .any(|pattern| line.contains(pattern));
            let binary = [
                "'rb'", "\"rb\"", "'wb'", "\"wb\"", "zipfile", "ZipFile", "tarfile",
            ]
            .iter()
            .any(|pattern| line.contains(pattern));
            if text_io && !binary && !line.contains("encoding") && !line.contains("re.search") {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    line_number + 1,
                    line.trim()
                ));
            }
        }
    }
    assert!(offenders.is_empty(), "{}", offenders.join("\n"));
    assert!(!Path::new("tests/test_utf8_sources.py").exists());
}

#[test]
fn ui_assets_are_utf8_and_html_declares_it() {
    for entry in fs::read_dir("src/ui").unwrap().flatten() {
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| ["html", "js", "css"].contains(&extension.to_str().unwrap()))
        {
            String::from_utf8(fs::read(path).unwrap()).unwrap();
        }
    }
    assert!(include_str!("../src/ui/room.html").contains("<meta charset=\"utf-8\">"));
}
