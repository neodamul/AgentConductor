//! Embed the project-owned AC icon in Windows executables using the installed SDK.
use std::{env, fs, path::PathBuf, process::Command};
fn main() {
    println!("cargo:rerun-if-changed=src/ui/assets/ac.ico");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    if env::var("CARGO_CFG_TARGET_ENV").as_deref() != Ok("msvc") {
        panic!("Windows releases require the supported MSVC toolchain");
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let icon =
        PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("src/ui/assets/ac.ico");
    let source = out.join("ac.rc");
    let resource = out.join("ac.res");
    fs::write(
        &source,
        format!(
            "1 ICON \"{}\"\n",
            icon.display().to_string().replace('\\', "\\\\")
        ),
    )
    .unwrap();
    let mut compilers = vec![PathBuf::from("rc.exe")];
    if let Some(root) = env::var_os("ProgramFiles(x86)") {
        let bin = PathBuf::from(root).join("Windows Kits/10/bin");
        let mut versions: Vec<_> = fs::read_dir(bin)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .collect();
        versions.sort();
        versions.reverse();
        compilers.extend(versions.into_iter().map(|p| p.join("x64/rc.exe")));
    }
    let compiler = compilers
        .into_iter()
        .find(|p| Command::new(p).arg("/?").output().is_ok())
        .expect("Windows SDK resource compiler (rc.exe) is required");
    let status = Command::new(compiler)
        .arg("/nologo")
        .arg("/fo")
        .arg(&resource)
        .arg(&source)
        .status()
        .expect("compile AC resource");
    assert!(status.success(), "AC resource compilation failed");
    println!("cargo:rustc-link-arg-bins={}", resource.display());
}
