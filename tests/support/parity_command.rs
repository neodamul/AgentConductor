//! #420: captures the actual parity subprocess request without launching a browser.
use std::{env,fs::OpenOptions,io::Write};
fn main(){
 let mut log=OpenOptions::new().create(true).append(true).open(env::var_os("AC_PARITY_CAPTURE").unwrap()).unwrap();
 writeln!(log,"{}|{}|{}",env::var("PARITY_THEME").unwrap_or_default(),env::var("PARITY_WIDTH").unwrap_or_default(),env::args().last().unwrap_or_default()).unwrap();
 if let Ok(code)=env::var("AC_PARITY_EXIT"){std::process::exit(code.parse().unwrap());}
}
