use super::*;

pub(super) struct Session {
    pub(super) job: Job,
    pub(super) process: OwnedHandle,
    pub(super) console: Option<Console>,
    pub(super) input: File,
    pub(super) receiver: Option<mpsc::Receiver<Vec<u8>>>,
    pub(super) reader: Option<std::thread::JoinHandle<()>>,
    pub(super) overflow: Arc<AtomicBool>,
    pub(super) pid: u32,
}
impl Session {
    pub(super) fn open(p: &AgentProfile, cwd: &Path, dir: &Path) -> Result<Self, String> {
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
    pub(super) fn finish(&mut self) -> Option<i32> {
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
