//! #420: exercise the actual Rust parity driver with an observable, dependency-free subprocess.
use std::{fs, path::Path, process::Command};
#[test]
fn parity_driver_dispatches_all_four_platform_matrix_cells() {
    let root = std::env::temp_dir().join(format!(
        "ac-parity-matrix-{}",
        agentconductor::native_guard::session_id().unwrap()
    ));
    fs::create_dir_all(root.join("bin")).unwrap();
    let suffix = if cfg!(windows) { ".exe" } else { "" };
    let driver = root.join(format!("driver{suffix}"));
    let node = root.join(format!("bin/node{suffix}"));
    for (file, out, test) in [
        ("tests/design_parity.rs", &driver, true),
        ("tests/support/parity_command.rs", &node, false),
    ] {
        let mut command = Command::new("rustc");
        command.arg(file).args(["-o"]).arg(out);
        if test {
            command.arg("--test");
        }
        assert!(command.status().unwrap().success());
    }
    let mut paths = vec![root.join("bin")];
    paths.extend(std::env::split_paths(
        &std::env::var_os("PATH").unwrap_or_default(),
    ));
    let captured = root.join("calls");
    let out = root.join("evidence");
    let result = Command::new(&driver)
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("PARITY_OUT", &out)
        .env("AC_PARITY_CAPTURE", &captured)
        .env_remove("PARITY_WIDTH")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let actual = fs::read_to_string(&captured).unwrap();
    let mut cells = actual
        .lines()
        .map(|line| {
            let p: Vec<_> = line.split('|').collect();
            assert_eq!(p.len(), 3);
            assert!(Path::new(p[2]).starts_with(&out));
            (p[0].to_owned(), p[1].to_owned(), p[2].to_owned())
        })
        .collect::<Vec<_>>();
    cells.sort();
    let mut expected = vec![];
    for theme in ["dark", "light"] {
        for width in [840, 1280] {
            expected.push((
                theme.to_owned(),
                width.to_string(),
                out.join(format!("{theme}-{width}"))
                    .to_string_lossy()
                    .into_owned(),
            ));
        }
    }
    expected.sort();
    assert_eq!(
        cells, expected,
        "every theme/width must run and keep independent captures"
    );
    let refused =
        Command::new(&driver)
            .env(
                "PATH",
                std::env::join_paths(std::iter::once(root.join("bin")).chain(
                    std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()),
                ))
                .unwrap(),
            )
            .env("PARITY_OUT", &out)
            .env("AC_PARITY_CAPTURE", &captured)
            .env("CI", "true")
            .env("AC_PARITY_EXIT", "77")
            .output()
            .unwrap();
    assert!(
        !refused.status.success(),
        "CI must fail when the browser check cannot run"
    );
    fs::remove_dir_all(root).unwrap();
}
