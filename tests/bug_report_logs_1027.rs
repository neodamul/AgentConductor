//! #1027: Report a bug zips masked log tails into Downloads and the draft says to drag it in.
use agentconductor::bug_report::{self, logs};
use serde_json::json;
use std::{fs, io::Read, path::Path};
#[path = "support/env.rs"]
mod test_env;

fn temp(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("ac-1027-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

fn entries(zip: &Path) -> Vec<(String, String)> {
    let mut archive = zip::ZipArchive::new(fs::File::open(zip).unwrap()).unwrap();
    (0..archive.len())
        .map(|i| {
            let mut file = archive.by_index(i).unwrap();
            let mut text = String::new();
            file.read_to_string(&mut text).unwrap();
            (file.name().unwrap().into_owned(), text)
        })
        .collect()
}

#[test]
fn secrets_are_masked_and_plain_text_is_kept() {
    for (input, expected) in [
        (
            "post https://hooks.slack.com/services/T0/B0/xyz ok",
            "post [masked] ok",
        ),
        (
            "Authorization: Bearer abc.def.ghi",
            "Authorization: Bearer [masked]",
        ),
        (
            r#"{"token": "s3cr3t", "id": 4}"#,
            r#"{"token": "[masked]", "id": 4}"#,
        ),
        ("api_key=XYZ next", "api_key=[masked] next"),
        ("key ghp_abcdefghijklmnopqrstuv end", "key [masked] end"),
        ("sk-ant-api03-abcdefghijkl used", "[masked] used"),
        (
            "request chat took 1200 ms at https://example.com/a",
            "request chat took 1200 ms at https://example.com/a",
        ),
    ] {
        assert_eq!(logs::mask(input), expected, "{input}");
    }
}

#[test]
fn the_zip_holds_only_chosen_masked_tails_plus_environment() {
    let app = temp("zip");
    let run = app.join("desktop/coordinator-ui-1");
    fs::create_dir_all(run.join("process")).unwrap();
    let mut requests = "x".repeat(logs::TAIL as usize + 4096);
    requests.push_str("\nwebhook https://hooks.slack.com/services/T/B/secret\n");
    fs::write(app.join("request-diagnostics.log"), &requests).unwrap();
    fs::write(run.join("process/stderr.log"), "boom\n").unwrap();
    fs::write(app.join("desktop/coordinator-ui-1.messenger.log"), "sent\n").unwrap();

    let listed = logs::files(&app);
    let ids: Vec<_> = listed["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| (f["id"].as_str().unwrap(), f["present"] == true))
        .collect();
    assert_eq!(
        ids,
        [
            ("environment", true),
            ("requests", true),
            ("messenger", true),
            ("run", true)
        ]
    );
    assert_eq!(listed["files"][1]["bytes"], logs::TAIL);

    let zip = logs::write_zip(&app, &["requests".into()], &app, "report.zip").unwrap();
    let entries = entries(&zip);
    let names: Vec<_> = entries.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ["environment.txt", "request-diagnostics.log"]);
    assert!(entries[0].1.contains(&agentconductor::update::current()));
    let log = &entries[1].1;
    assert!(log.len() as u64 <= logs::TAIL);
    assert!(log.contains("webhook [masked]") && !log.contains("hooks.slack.com"));
    fs::remove_dir_all(app).unwrap();
}

#[test]
fn reporting_with_logs_writes_to_downloads_and_names_the_zip_in_the_draft() {
    let home = temp("home");
    fs::create_dir_all(home.join(".agentconductor")).unwrap();
    fs::write(
        home.join(".agentconductor/request-diagnostics.log"),
        "slow\n",
    )
    .unwrap();
    let mut env = test_env::lock();
    env.set("HOME", &home);
    env.set("USERPROFILE", &home);
    let report =
        bug_report::handle(json!({"kind":"report_bug","open":false,"logs":["requests"]})).unwrap();
    let zip = Path::new(report["zip"].as_str().unwrap());
    assert_eq!(zip.parent().unwrap(), home.join("Downloads"));
    let name = zip.file_name().unwrap().to_string_lossy().into_owned();
    let body = report["body"].as_str().unwrap();
    assert!(body.contains(&format!("## Logs\nDrag {name} from your Downloads folder")));
    assert_eq!(entries(zip).len(), 2);

    let plain = bug_report::handle(json!({"kind":"report_bug","open":false})).unwrap();
    assert!(plain.get("zip").is_none() && !plain["body"].as_str().unwrap().contains("## Logs"));
    let files = bug_report::handle(json!({"kind":"bug_report_files"})).unwrap();
    assert_eq!(files["files"][1]["present"], true);
    assert_eq!(files["files"][2]["present"], false);
    drop(env);
    fs::remove_dir_all(home).unwrap();
}
