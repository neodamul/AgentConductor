#[path = "support/release_package.rs"]
mod support;

use agentconductor::release_package::assemble;
use serde_json::json;
use std::fs;
use support::{fixture, temp};

#[test]
fn accepts_permissive_license_choices_used_by_native_dependencies() {
    let root = temp();
    let target = "aarch64-apple-darwin";
    let mut metadata = fixture(&root, target);
    let template = metadata["packages"][0].clone();
    let expressions = [
        "0BSD OR MIT OR Apache-2.0",
        "Apache-2.0 OR GPL-2.0-only",
        "Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT",
        "BSD-2-Clause OR Apache-2.0 OR MIT",
        "BSD-3-Clause OR Apache-2.0",
        "BSD-3-Clause OR MIT OR Apache-2.0",
        "MIT / Apache-2.0",
        "MIT OR Apache-2.0 OR LGPL-2.1-or-later",
        "MIT OR Zlib OR Apache-2.0",
        "Unlicense/MIT",
        "Zlib OR Apache-2.0 OR MIT",
    ];
    for (index, expression) in expressions.iter().enumerate() {
        let id = format!("license-policy-{index}");
        let mut package = template.clone();
        package["id"] = json!(id);
        package["name"] = json!(format!("dependency-{index}"));
        package["license"] = json!(expression);
        metadata["packages"].as_array_mut().unwrap().push(package);
        metadata["resolve"]["nodes"]
            .as_array_mut()
            .unwrap()
            .push(json!({"id":id}));
    }
    for name in [
        "LICENSE-APACHE",
        "LICENSE-BSD",
        "LICENSE-0BSD",
        "LICENSE-ZLIB",
    ] {
        fs::write(
            root.join("dependency").join(name),
            "reviewed permissive license\n",
        )
        .unwrap();
    }
    let destination = root.join("dist/package");
    let result = assemble(&root, &destination, &metadata, target, &"e".repeat(40));
    assert!(result.is_ok(), "{}", result.unwrap_err());
    fs::remove_dir_all(root).unwrap();
}
