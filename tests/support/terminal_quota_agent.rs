use std::io::{BufRead,Write};
fn main(){
    if std::env::var("AC_QUOTA_CASE").as_deref()==Ok("trust"){
        println!("Permission Required: trust this project");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(60)); return;
    }
    println!("Claude Code vfixture\n$"); std::io::stdout().flush().unwrap();
    for line in std::io::stdin().lock().lines(){
        if line.unwrap().trim()=="/usage"{
            println!("Current session\n27% used\nCurrent week\n60% used");
            std::io::stdout().flush().unwrap();
        }else{panic!("Only usage is allowed");}
    }
}
