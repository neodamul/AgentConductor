use super::sync_directory;
#[cfg(windows)]
use super::windows_job;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Ownership {
    pub coordinator_pid: u32,
    pub pid: Option<u32>,
    pub process_group: Option<i32>,
    pub phase: String,
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_signal: Option<i32>,
    pub stopped: bool,
    pub timed_out: bool,
}

impl Ownership {
    #[cfg(unix)]
    pub fn ensure_quiescent(&self) -> Result<(), String> {
        if self.phase == "not_spawned" && self.pid.is_none() && self.process_group.is_none() {
            return Ok(());
        }
        if !matches!(self.phase.as_str(), "running" | "terminal") {
            return Err("ambiguous process ownership; manual investigation required".into());
        }
        let pid = self
            .pid
            .filter(|id| *id > 0 && *id <= i32::MAX as u32)
            .ok_or("invalid or missing owned PID")? as i32;
        if self.process_group != Some(pid) {
            return Err("invalid or missing owned process group".into());
        }
        for target in [pid, -pid] {
            // SAFETY: kill with signal 0 only probes for existence; `target` is a validated PID or its negation and no memory is passed.
            if unsafe { libc::kill(target, 0) } == 0 {
                return Err("owned process or group is still present; recovery refused".into());
            }
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
                return Err("cannot prove owned process absence; recovery refused".into());
            }
        }
        Ok(())
    }

    #[cfg(windows)]
    pub fn ensure_quiescent(&self) -> Result<(), String> {
        if self.phase == "not_spawned" && self.pid.is_none() && self.process_group.is_none() {
            return Ok(());
        }
        if !matches!(self.phase.as_str(), "running" | "terminal") {
            return Err("ambiguous process ownership; manual investigation required".into());
        }
        let pid = self
            .pid
            .filter(|id| *id > 0 && *id <= i32::MAX as u32)
            .ok_or("invalid or missing owned PID")?;
        if self.process_group != Some(pid as i32) {
            return Err("invalid or missing owned process group".into());
        }
        // Job members die with the job handle, so the root's absence proves quiescence.
        if !windows_job::process_absent(pid)? {
            return Err("owned process or group is still present; recovery refused".into());
        }
        Ok(())
    }
}

pub(crate) fn record_owner(dir: &std::path::Path, owner: &Ownership) -> Result<(), String> {
    use std::io::Write;
    let pending = dir.join("owner.pending");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| format!("create process ownership record: {e}"))?;
    file.write_all(&serde_json::to_vec(owner).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    std::fs::rename(pending, dir.join("owner.json")).map_err(|e| e.to_string())?;
    sync_directory(dir)
}
