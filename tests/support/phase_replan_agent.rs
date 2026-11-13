use std::io::Read;
fn main(){
 if std::env::args().nth(1).as_deref()==Some("check"){assert!(std::path::Path::new(&std::env::args().nth(2).unwrap()).is_file());return;}
 let mut prompt=String::new();std::io::stdin().read_to_string(&mut prompt).unwrap();
 assert!(prompt.contains("completed_phases")&&prompt.contains("diff")&&prompt.contains("tests")&&prompt.contains("final_reports"));
 if let Ok(marker)=std::env::var("AC_REPLAN_MARKER"){std::fs::write(marker,"called").unwrap();}
 if std::env::var("AC_REPLAN_WAIT").is_ok(){std::thread::sleep(std::time::Duration::from_secs(30));}
 println!("{}",std::fs::read_to_string(std::env::var(if prompt.contains("Re-plan ONLY Phase p3 "){"AC_REPLAN_REPLY_NEXT"}else{"AC_REPLAN_REPLY"}).unwrap()).unwrap());
}
