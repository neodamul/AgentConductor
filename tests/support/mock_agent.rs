use std::io::{Read, Write};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("--version") => println!("mock 0.1"),
        Some("models") => println!("local/one\nlocal/two"),
        Some("-p") => {
            use std::io::BufRead;
            let mut line=String::new();std::io::stdin().lock().read_line(&mut line).unwrap();
            assert!(line.contains("initialize"));assert!(!line.contains("prompt"));
            println!("{{\"type\":\"control_response\",\"response\":{{\"request_id\":\"catalog-init\",\"subtype\":\"success\",\"response\":{{\"models\":[{{\"value\":\"local/one\"}},{{\"value\":\"local/two\"}}]}}}}}}");
        }
        Some("app-server") => {
            use std::io::BufRead;
            for line in std::io::stdin().lock().lines() {
                let line=line.unwrap();assert!(!line.contains("turn/start"));
                if line.contains("model/list") {println!("{{\"id\":2,\"result\":{{\"data\":[{{\"model\":\"local/one\"}},{{\"model\":\"local/two\"}}],\"nextCursor\":null}}}}");break;}
                if line.contains("clientInfo") {println!("{{\"id\":1,\"result\":{{}}}}");}
                std::io::stdout().flush().unwrap();
            }
        }
        Some("echo") => {
            std::io::stderr().write_all(&vec![b'e'; 2 * 1024 * 1024]).unwrap();
            let mut prompt = Vec::new();
            std::io::stdin().read_to_end(&mut prompt).unwrap();
            std::io::stdout().write_all(&prompt).unwrap();
        }
        Some("stubborn") => {
            unsafe extern "C" { fn signal(sig: i32, handler: usize) -> usize; }
            unsafe { signal(2, 1); signal(15, 1); }
            std::fs::write("ready", "yes").unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
        Some("fail") => std::process::exit(7),
        Some("test") | Some("test-wait") => {
            if std::env::args().nth(1).as_deref()==Some("test-wait") {
                println!("integration test started");
                std::io::stdout().flush().unwrap();
                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
            for path in ["one.txt","two.txt","three.txt"] { assert!(std::path::Path::new(path).exists()); }
            println!("integration tests passed");
        }
        Some("plan") => {
            let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
            print!("{{\"type\":\"result\",\"payload\":{{\"tasks\":[");
            for (i,path) in ["one.txt","two.txt","three.txt"].iter().enumerate(){
                if i>0 {print!(",");}
                print!("{{\"id\":\"task-{i}\",\"agent\":\"mock\",\"prompt\":\"{path}\",\"paths\":[\"{path}\"],\"depends_on\":[],\"test_command\":[\"git\",\"diff\",\"--check\"]}}");
            }
            println!("]}}}}");
        }
        Some("task") | Some("quota") | Some("quota-once") | Some("quota-known") => {
            let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
            let path=prompt.lines().next().unwrap().trim();
            println!("{{\"type\":\"session\",\"session_id\":\"mock-{path}\"}}");
            std::io::stdout().flush().unwrap();
            let mode=std::env::args().nth(1).unwrap();
            let marker=std::env::var("MOCK_MARKERS").map(|dir|std::path::PathBuf::from(dir).join(format!("marker-{path}"))).ok();
            if mode=="quota" || mode=="quota-known" || (mode=="quota-once" && !marker.as_ref().unwrap().exists()) {
                if let Some(marker)=marker {std::fs::write(marker,"restricted").unwrap();}
                let reset=if mode=="quota-known" {",\"resetsAt\":200"} else {""};
                println!("{{\"type\":\"error\",\"error\":{{\"code\":\"usage_limit_reached\",\"message\":\"Usage limit reached\"{reset}}}}}");
                std::process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
            if path=="fail" {std::process::exit(7);}
            std::fs::write(path,"implemented\n").unwrap();
            println!("{{\"type\":\"result\",\"payload\":{{\"summary\":\"implemented\"}}}}");
        }
        Some("tree") => {
            let child = std::process::Command::new(std::env::current_exe().unwrap())
                .arg("sleep")
                .spawn()
                .unwrap();
            std::fs::write("grandchild.pid", child.id().to_string()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
        _ => std::thread::sleep(std::time::Duration::from_secs(60)),
    }
}
