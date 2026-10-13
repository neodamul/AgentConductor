use std::io::{BufRead,Write};
fn main(){
    if std::env::var("AC_QUOTA_CASE").as_deref()==Ok("trust_stuck"){
        // Asks again after every answer.
        loop{
            println!("Permission Required: trust this project");
            std::io::stdout().flush().unwrap();
            let mut line=String::new();
            if std::io::stdin().lock().read_line(&mut line).unwrap()==0{return;}
        }
    }
    if std::env::var("AC_QUOTA_CASE").as_deref()==Ok("trust_yn"){
        // Screen-reader form seen on Windows (2026-10-13): Enter alone is not an answer.
        println!("Permission Required: Accessing workspace:\ny. Yes, I trust this folder\nn. No, exit\nEnter y/n:");
        loop{
            std::io::stdout().flush().unwrap();
            let mut line=String::new();
            if std::io::stdin().lock().read_line(&mut line).unwrap()==0{return;}
            match line.trim(){"y"=>break,"n"=>return,_=>println!("Please answer y or n.\nEnter y/n:")}
        }
    }
    if std::env::var("AC_QUOTA_CASE").as_deref()==Ok("trust"){
        // Enter accepts the default "Yes, proceed"; anything else is refused.
        println!("Do you trust the files in this folder?\nPermission Required: trust this project");
        std::io::stdout().flush().unwrap();
        let mut line=String::new();
        std::io::stdin().lock().read_line(&mut line).unwrap();
        if !line.trim().is_empty(){panic!("only Enter accepts trust");}
    }
    if std::env::var("AC_QUOTA_CASE").as_deref()==Ok("onboarding"){
        // DD-94: first-run screen reported from a real Windows machine (Claude Code 2.1.218).
        println!("Welcome to Claude Code v2.1.218\nLet's get started.\nChoose the text style that looks best with your terminal To change this later, run /theme\nEnter selection [1-7], or Escape to cancel:");
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(60)); return;
    }
    if let Some(expected)=std::env::var_os("AC_EXPECT_CWD"){
        let same=std::env::current_dir().ok().and_then(|d|d.canonicalize().ok())==std::path::Path::new(&expected).canonicalize().ok();
        if !same {println!("wrong cwd {:?}",std::env::current_dir()); std::io::stdout().flush().unwrap(); return;}
    }
    println!("Claude Code vfixture\n$"); std::io::stdout().flush().unwrap();
    for line in std::io::stdin().lock().lines(){
        if line.unwrap().trim()=="/usage"{
            println!("Current session\n27% used\nCurrent week\n60% used");
            std::io::stdout().flush().unwrap();
        }else{panic!("Only usage is allowed");}
    }
}
