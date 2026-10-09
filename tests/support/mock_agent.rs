use std::io::{Read, Write};

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("control") | Some("control-malformed") => {
            use std::io::BufRead;
            let mut stdin = std::io::stdin().lock();
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            assert!(line.contains("initialize"));
            std::io::stderr().write_all(&vec![b'e'; 2 * 1024 * 1024]).unwrap();
            println!("{{\"type\":\"control_response\",\"response\":{{\"subtype\":\"success\",\"request_id\":\"initialize\"}}}}");
            std::io::stdout().flush().unwrap();
            line.clear();stdin.read_line(&mut line).unwrap();assert!(line.contains("user"));
            if std::env::args().nth(1).as_deref()==Some("control-malformed") {println!("invalid control JSON");}
            else {println!("{{\"type\":\"control_request\",\"request_id\":\"permission\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_name\":\"Bash\",\"input\":{{\"command\":\"touch forbidden\"}}}}}}");}
            std::io::stdout().flush().unwrap();
            line.clear();stdin.read_line(&mut line).unwrap();
            if line.contains("allow") {std::fs::write("forbidden","command ran").unwrap();}
            if line.is_empty() {std::thread::sleep(std::time::Duration::from_secs(60));return;}
            println!("{{\"type\":\"result\",\"is_error\":false}}");
            std::io::stdout().flush().unwrap();
            // A result is not proof of idle; a late permission request still needs stdin.
            println!("{{\"type\":\"control_request\",\"request_id\":\"late-permission\",\"request\":{{\"subtype\":\"can_use_tool\",\"tool_name\":\"Bash\",\"input\":{{\"command\":\"touch forbidden\"}}}}}}");
            std::io::stdout().flush().unwrap();
            line.clear();stdin.read_line(&mut line).unwrap();assert!(line.contains("late-permission"));
            println!("{{\"type\":\"system\",\"subtype\":\"session_state_changed\",\"state\":\"idle\"}}");std::io::stdout().flush().unwrap();
            line.clear();assert_eq!(stdin.read_line(&mut line).unwrap(),0);
        }
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
        Some("dependency-test") => {
            let one=std::fs::read_to_string("one.txt").unwrap();
            let two=std::fs::read_to_string("two.txt").unwrap();
            let three=std::fs::read_to_string("three.txt").unwrap();
            assert_eq!(two,format!("observed {one}"));
            assert_eq!(three,format!("documented {two}"));
            println!("dependency consistency passed");
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
        Some("review") | Some("review-unknown") | Some("review-slow") | Some("review-write") | Some("review-accept") => {
            let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
            assert!(prompt.contains("agent:one.txt"));assert!(prompt.contains("untrusted data"));
            if std::env::args().nth(1).as_deref()==Some("review-accept") {
                println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"accept\",\"summary\":\"Evidence verified\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[]}}}}");return;
            }
            if std::env::args().nth(1).as_deref()==Some("review-slow") {
                println!("{{\"type\":\"session\",\"session_id\":\"review-{}\"}}",std::process::id());std::io::stdout().flush().unwrap();std::thread::sleep(std::time::Duration::from_secs(3));
            }
            if std::env::args().nth(1).as_deref()==Some("review-write") {std::fs::write("one.txt","invalid coordinator edit\n").unwrap();}
            let task=if std::env::args().nth(1).as_deref()==Some("review-unknown") {"invented"}else{"one.txt"};
            println!("{{\"type\":\"result\",\"payload\":{{\"verdict\":\"revise\",\"summary\":\"Repair failed implementation\",\"evidence\":[\"agent:one.txt\"],\"revisions\":[{{\"task\":\"{task}\",\"feedback\":\"Repair failed task\"}}]}}}}");
        }
        Some("task") | Some("task-slow") | Some("dependency") | Some("quota") | Some("quota-once") | Some("quota-known") => {
            let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
            let original=prompt.lines().next().unwrap().trim();
            let path=if original=="fail" && prompt.contains("Repair failed task") {
                prompt.lines().find_map(|line|line.strip_prefix("Owned paths: ")).unwrap().trim()
            } else {original};
            use std::hash::{Hash, Hasher};
            let mut identity=std::collections::hash_map::DefaultHasher::new();
            std::env::current_dir().unwrap().hash(&mut identity);
            println!("{{\"type\":\"session\",\"session_id\":\"mock-{:x}-{path}\"}}",identity.finish());
            std::io::stdout().flush().unwrap();
            let mode=std::env::args().nth(1).unwrap();
            let marker=std::env::var("MOCK_MARKERS").map(|dir|std::path::PathBuf::from(dir).join(format!("marker-{path}"))).ok();
            if mode=="quota" || mode=="quota-known" || (mode=="quota-once" && !marker.as_ref().unwrap().exists()) {
                if let Some(marker)=marker {std::fs::write(marker,"restricted").unwrap();}
                let reset=if mode=="quota-known" {",\"resetsAt\":200"} else {""};
                println!("{{\"type\":\"error\",\"error\":{{\"code\":\"usage_limit_reached\",\"message\":\"Usage limit reached\"{reset}}}}}");
                std::process::exit(1);
            }
            std::thread::sleep(std::time::Duration::from_millis(if mode=="task-slow" {3000} else {500}));
            if path=="fail" {std::process::exit(7);}
            let content=if mode=="dependency" {
                if path!="one.txt" && prompt.contains("\"attempt\": 2") { assert!(std::path::Path::new(path).exists(),"previous owned partial file was discarded"); }
                match path {
                    "one.txt" => if prompt.contains("Use version two") {"v2\n".into()} else {"v1\n".into()},
                    "two.txt" => format!("observed {}",std::fs::read_to_string("one.txt").unwrap()),
                    "three.txt" => format!("documented {}",std::fs::read_to_string("two.txt").unwrap()),
                    _ => panic!("unknown owned path"),
                }
            } else {"implemented\n".into()};
            std::fs::write(path,content).unwrap();
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
