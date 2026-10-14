//! A short-lived Windows pseudoconsole. No model turn, server or permission approval.
use crate::{plan::AgentProfile, process::windows_job::Job, quota::Snapshot};
use std::{
    fs::File,
    io::{Read, Write},
    os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle},
    path::{Path, PathBuf},
    ptr::{null, null_mut},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows_sys::Win32::{
    Foundation::HANDLE,
    System::{
        Console::{COORD, ClosePseudoConsole, CreatePseudoConsole, HPCON},
        Pipes::CreatePipe,
        Threading::{
            CREATE_NEW_PROCESS_GROUP, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, CreateProcessW,
            DeleteProcThreadAttributeList, EXTENDED_STARTUPINFO_PRESENT, GetExitCodeProcess,
            InitializeProcThreadAttributeList, PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE,
            PROCESS_INFORMATION, ResumeThread, STARTF_USESTDHANDLES, STARTUPINFOEXW,
            UpdateProcThreadAttribute, WaitForSingleObject,
        },
    },
};
fn wide(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(Some(0)).collect()
}
fn pipe() -> Result<(OwnedHandle, OwnedHandle), String> {
    let (mut read, mut write): (HANDLE, HANDLE) = (null_mut(), null_mut());
    if unsafe { CreatePipe(&mut read, &mut write, null(), 0) } == 0 {
        return Err("Windows quota pipe creation failed".into());
    }
    Ok(unsafe {
        (
            OwnedHandle::from_raw_handle(read),
            OwnedHandle::from_raw_handle(write),
        )
    })
}
struct Console(HPCON);
impl Drop for Console {
    fn drop(&mut self) {
        unsafe { ClosePseudoConsole(self.0) }
    }
}
struct Attributes(Vec<usize>, bool);
impl Attributes {
    fn new(console: HPCON) -> Result<Self, String> {
        let mut size = 0;
        unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size) };
        if size == 0 {
            return Err("Windows quota attributes unavailable".into());
        }
        let mut a = Self(vec![0; size.div_ceil(std::mem::size_of::<usize>())], false);
        if unsafe { InitializeProcThreadAttributeList(a.0.as_mut_ptr().cast(), 1, 0, &mut size) }
            == 0
        {
            return Err("Windows quota attributes failed".into());
        }
        a.1 = true;
        if unsafe {
            UpdateProcThreadAttribute(
                a.0.as_mut_ptr().cast(),
                0,
                PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE as usize,
                console as *const _,
                std::mem::size_of::<HPCON>(),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err("Windows pseudoconsole binding failed".into());
        }
        Ok(a)
    }
}
impl Drop for Attributes {
    fn drop(&mut self) {
        if self.1 {
            unsafe { DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast()) }
        }
    }
}
struct Session {
    job: Job,
    process: OwnedHandle,
    console: Option<Console>,
    input: File,
    receiver: Option<mpsc::Receiver<Vec<u8>>>,
    reader: Option<std::thread::JoinHandle<()>>,
    overflow: Arc<AtomicBool>,
    pid: u32,
}
impl Session {
    fn open(p: &AgentProfile, cwd: &Path, dir: &Path) -> Result<Self, String> {
        let (ir, iw) = pipe()?;
        let (or, ow) = pipe()?;
        let mut hpc = 0;
        let hr = unsafe {
            CreatePseudoConsole(
                COORD { X: 160, Y: 50 },
                ir.as_raw_handle(),
                ow.as_raw_handle(),
                0,
                &mut hpc,
            )
        };
        if hr < 0 {
            return Err(format!("Windows ConPTY unavailable: {hr:#x}"));
        }
        let console = Console(hpc);
        drop(ir);
        drop(ow);
        let executable = crate::executable::resolve(&p.executable)
            .unwrap_or_else(|| PathBuf::from(&p.executable));
        let settings = dir.join("quota-safety.json");
        std::fs::write(&settings, br#"{"disableAllHooks":true}"#).map_err(|e| e.to_string())?;
        let settings = settings
            .to_str()
            .ok_or("quota settings path is not UTF-8")?;
        if settings.contains(['\r', '\n', '\0', '"', '%', '!', '^', '&', '|', '<', '>']) {
            return Err("unsupported quota settings path".into());
        }
        let flags = format!(
            r#"--ax-screen-reader --tools "" --strict-mcp-config --setting-sources "" --settings "{settings}""#
        );
        let name = executable
            .to_str()
            .ok_or("quota executable path is not UTF-8")?
            .to_owned();
        if name.contains(['\r', '\n', '\0', '"']) {
            return Err("invalid quota executable path".into());
        }
        let batch = executable
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("cmd") || s.eq_ignore_ascii_case("bat"));
        let (application, command) = if batch {
            if name.contains(['%', '!', '^', '&', '|', '<', '>']) {
                return Err(
                    "unsupported batch launcher path; select the native CLI executable".into(),
                );
            }
            let app =
                PathBuf::from(std::env::var_os("SystemRoot").ok_or("SystemRoot unavailable")?)
                    .join("System32")
                    .join("cmd.exe");
            (
                app.clone(),
                format!("cmd.exe /d /s /c \"\"{name}\" {flags}\""),
            )
        } else {
            (executable, format!("\"{name}\" {flags}"))
        };
        let mut env: std::collections::BTreeMap<String, (String, String)> = std::env::vars()
            .map(|(k, v)| (k.to_ascii_lowercase(), (k, v)))
            .collect();
        for (k, v) in &p.environment {
            env.insert(k.to_ascii_lowercase(), (k.clone(), v.clone()));
        }
        env.remove("anthropic_api_key");
        let mut block = Vec::<u16>::new();
        for (_, (k, v)) in env {
            if k.contains('\0') || v.contains('\0') {
                return Err("invalid quota environment".into());
            }
            block.extend(format!("{k}={v}").encode_utf16());
            block.push(0);
        }
        block.push(0);
        let mut attrs = Attributes::new(hpc)?;
        let mut si: STARTUPINFOEXW = unsafe { std::mem::zeroed() };
        si.StartupInfo.cb = std::mem::size_of::<STARTUPINFOEXW>() as u32;
        // Null standard handles force ConPTY bindings instead of inheriting redirected
        // coordinator handles (Microsoft terminal discussion #15814).
        si.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        si.lpAttributeList = attrs.0.as_mut_ptr().cast();
        let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
        let app = wide(application.as_os_str());
        let mut cmd = wide(std::ffi::OsStr::new(&command));
        // DD-94: cmd.exe rejects \\?\ working directories and falls back to System32.
        let dir = wide(&crate::git::git_path(cwd.as_os_str()));
        let job = Job::new()?;
        if unsafe {
            CreateProcessW(
                app.as_ptr(),
                cmd.as_mut_ptr(),
                null(),
                null(),
                0,
                EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_UNICODE_ENVIRONMENT
                    | CREATE_SUSPENDED
                    | CREATE_NEW_PROCESS_GROUP,
                block.as_ptr().cast(),
                dir.as_ptr(),
                &si.StartupInfo,
                &mut pi,
            )
        } == 0
        {
            return Err(format!(
                "Windows quota CLI could not start: {}",
                std::io::Error::last_os_error()
            ));
        }
        let process = unsafe { OwnedHandle::from_raw_handle(pi.hProcess) };
        let thread = unsafe { OwnedHandle::from_raw_handle(pi.hThread) };
        // Adopt before resume; a failed assignment must never leave the suspended child alive.
        if let Err(e) = job.assign(process.as_raw_handle()) {
            unsafe {
                windows_sys::Win32::System::Threading::TerminateProcess(process.as_raw_handle(), 1)
            };
            return Err(e);
        }
        if unsafe { ResumeThread(thread.as_raw_handle()) } == u32::MAX {
            return Err("Windows quota process resume failed".into());
        }
        let (tx, rx) = mpsc::sync_channel(8);
        let overflow = Arc::new(AtomicBool::new(false));
        let full = overflow.clone();
        let mut output = File::from(or);
        let reader = std::thread::spawn(move || {
            let mut buf = [0; 8192];
            while let Ok(n) = output.read(&mut buf) {
                if n == 0 {
                    break;
                }
                if let Err(mpsc::TrySendError::Full(_)) = tx.try_send(buf[..n].to_vec()) {
                    full.store(true, Ordering::Release);
                }
            }
        });
        Ok(Self {
            job,
            process,
            console: Some(console),
            input: File::from(iw),
            receiver: Some(rx),
            reader: Some(reader),
            overflow,
            pid: pi.dwProcessId,
        })
    }
    fn finish(&mut self) -> Option<i32> {
        self.job.terminate();
        unsafe { WaitForSingleObject(self.process.as_raw_handle(), 3000) };
        let mut code = 0;
        let ok = unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) };
        drop(self.console.take());
        drop(self.receiver.take());
        if let Some(t) = self.reader.take() {
            let _ = t.join();
        }
        if ok == 0 { None } else { Some(code as i32) }
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.finish();
    }
}
fn collect(p: AgentProfile, dir: PathBuf, cwd: PathBuf) -> Result<Snapshot, String> {
    std::fs::create_dir(&dir).map_err(|e| e.to_string())?;
    let mut s = Session::open(&p, &cwd, &dir)?;
    let mut owner = crate::process::Ownership {
        coordinator_pid: std::process::id(),
        pid: Some(s.pid),
        process_group: Some(s.pid as i32),
        phase: "running".into(),
        exit_code: None,
        termination_signal: None,
        stopped: false,
        timed_out: false,
    };
    crate::process::record_owner(&dir, &owner)?;
    let mut spool = File::create(dir.join("stdout.terminal")).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    let mut sent = false;
    let mut found = None;
    // Output before the last trust answer is not read again.
    let (mut start, mut answers) = (0, 0);
    let end = Instant::now() + Duration::from_secs(20);
    let result = (|| {
        loop {
            if Instant::now() >= end {
                owner.timed_out = true;
                return Err(super::quota_timeout(sent).into());
            }
            if s.overflow.load(Ordering::Acquire) {
                return Err("Claude quota output queue exceeded its bound".into());
            }
            match s
                .receiver
                .as_ref()
                .unwrap()
                .recv_timeout(Duration::from_millis(1000))
            {
                Ok(chunk) => {
                    if data.len() + chunk.len() > 1024 * 1024 {
                        return Err("Claude quota output exceeds 1 MiB".into());
                    }
                    spool.write_all(&chunk).map_err(|e| e.to_string())?;
                    data.extend(chunk);
                    let text = super::plain(&String::from_utf8_lossy(&data[start..]));
                    if super::claude_trust_prompt(&text) {
                        if answers >= super::TRUST_ANSWERS {
                            return Err(super::TRUST_REFUSED.into());
                        }
                        s.input
                            .write_all(super::claude_trust_answer(&text))
                            .map_err(|e| e.to_string())?;
                        s.input.flush().map_err(|e| e.to_string())?;
                        answers += 1;
                        start = data.len();
                        continue;
                    }
                    if let Some(blocker) = super::claude_blocker(&text) {
                        return Err(blocker.into());
                    }
                    if !sent && super::claude_usage_ready(&text) {
                        s.input.write_all(b"/usage\r").map_err(|e| e.to_string())?;
                        s.input.flush().map_err(|e| e.to_string())?;
                        sent = true;
                    }
                    if sent {
                        let now = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map_err(|e| e.to_string())?
                            .as_secs() as i64;
                        if let Ok(snapshot) = super::parse_claude_usage(&text, now) {
                            found = Some(snapshot);
                        }
                    }
                }
                Err(mpsc::RecvTimeoutError::Timeout) if found.is_some() => {
                    return Ok(found.take().unwrap());
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    return found.ok_or("Claude closed before reporting quota".into());
                }
                Err(_) => {}
            }
        }
    })();
    owner.exit_code = s.finish();
    owner.phase = "terminal".into();
    crate::process::record_owner(&dir, &owner)?;
    result
}
pub async fn query_claude(p: &AgentProfile, dir: &Path, cwd: &Path) -> Result<Snapshot, String> {
    // An empty lpCurrentDirectory is not NULL: CreateProcessW fails with os error 123.
    super::require_project(cwd)?;
    let (p, dir, cwd) = (p.clone(), dir.to_owned(), cwd.to_owned());
    tokio::task::spawn_blocking(move || collect(p, dir, cwd))
        .await
        .map_err(|_| "Windows quota collector failed".to_string())?
}
