use std::io::{self,BufRead,Write};
fn main(){
 assert_eq!(std::env::args().nth(1).as_deref(),Some("app-server"));
 assert!(std::env::var("OPENAI_API_KEY").is_err());
 assert!(std::env::var("CODEX_API_KEY").is_err());
 let mut file=std::fs::File::create("requests.jsonl").unwrap();
 for line in io::stdin().lock().lines(){
  let line=line.unwrap();writeln!(file,"{line}").unwrap();
  if line.contains("\"method\":\"initialize\""){println!("{{\"id\":1,\"result\":{{}}}}");}
  else if line.contains("account/rateLimits/read"){
  if let Ok(path)=std::env::var("QUOTA_TEST_RELEASE") {
   let path=std::path::Path::new(&path);std::fs::write(path.with_extension("waiting"),b"waiting").unwrap();
   let deadline=std::time::Instant::now()+std::time::Duration::from_secs(10);
   while !path.exists(){assert!(std::time::Instant::now()<deadline,"quota fixture was not released");std::thread::sleep(std::time::Duration::from_millis(10));}
  }
  println!("{{\"id\":2,\"result\":{{\"rateLimits\":{{\"primary\":{{\"usedPercent\":27,\"windowDurationMins\":300}}}}}}}}");}
  else {assert!(line.contains("initialized"));}
 }
}
