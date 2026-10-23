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
mod session;
use session::Session;

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
    // SAFETY: both out-pointers are valid, writable HANDLEs and the default security attributes are null.
    if unsafe { CreatePipe(&mut read, &mut write, null(), 0) } == 0 {
        return Err("Windows quota pipe creation failed".into());
    }
    // SAFETY: CreatePipe succeeded, so `read` and `write` are open handles that nothing else owns.
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
        // SAFETY: `self.0` is the pseudoconsole created by `Session::open`, closed once by its owner.
        unsafe { ClosePseudoConsole(self.0) }
    }
}
struct Attributes(Vec<usize>, bool);
impl Attributes {
    fn new(console: HPCON) -> Result<Self, String> {
        let mut size = 0;
        // SAFETY: a null list only asks for the required size, which is written to `size`.
        unsafe { InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut size) };
        if size == 0 {
            return Err("Windows quota attributes unavailable".into());
        }
        let mut a = Self(vec![0; size.div_ceil(std::mem::size_of::<usize>())], false);
        // SAFETY: the buffer holds at least `size` bytes, aligned as usize, and is initialized exactly once.
        if unsafe { InitializeProcThreadAttributeList(a.0.as_mut_ptr().cast(), 1, 0, &mut size) }
            == 0
        {
            return Err("Windows quota attributes failed".into());
        }
        a.1 = true;
        // SAFETY: the list was initialized above and outlives the attribute; the pseudoconsole attribute takes the HPCON value itself with its size.
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
            // SAFETY: `self.1` is set only after the list was initialized, so it is deleted exactly once.
            unsafe { DeleteProcThreadAttributeList(self.0.as_mut_ptr().cast()) }
        }
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
