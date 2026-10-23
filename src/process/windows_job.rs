use std::{ffi::c_void, ptr::null};
use windows_sys::Win32::{
    Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError, HANDLE, WAIT_OBJECT_0},
    System::{
        Console::{CTRL_BREAK_EVENT, GenerateConsoleCtrlEvent},
        JobObjects::{
            AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JobObjectExtendedLimitInformation,
            SetInformationJobObject, TerminateJobObject,
        },
        Threading::{
            CREATE_NEW_PROCESS_GROUP, CREATE_NO_WINDOW, CREATE_SUSPENDED, OpenProcess,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, WaitForSingleObject,
        },
    },
};

pub const CREATE_FLAGS: u32 = CREATE_SUSPENDED | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW;

#[link(name = "ntdll")]
unsafe extern "system" {
    fn NtResumeProcess(process: HANDLE) -> i32;
}

pub struct Job(HANDLE);
// SAFETY: the job handle is only used through kernel calls that are safe from any thread, and `Job`
// owns it exclusively until `Drop`.
unsafe impl Send for Job {}
// SAFETY: as for `Send`; `&Job` only passes the handle to thread-safe kernel calls.
unsafe impl Sync for Job {}

impl Job {
    pub fn new() -> Result<Self, String> {
        // SAFETY: null attributes and name are the documented form; a null result is checked below.
        let handle = unsafe { CreateJobObjectW(null(), null()) };
        if handle.is_null() {
            return Err(format!(
                "create job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        let job = Self(handle);
        // SAFETY: JOBOBJECT_EXTENDED_LIMIT_INFORMATION is a plain C struct of integers and pointers, valid when all-zero.
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        // SAFETY: `job.0` is the job handle just created, and `info` is a valid struct whose size is passed.
        let ok = unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &info as *const _ as *const c_void,
                std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        };
        if ok == 0 {
            return Err(format!(
                "configure job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(job)
    }

    pub fn assign(&self, process: *mut c_void) -> Result<(), String> {
        // SAFETY: `self.0` is a valid job handle; the caller passes a process handle it owns, and failure is reported.
        if unsafe { AssignProcessToJobObject(self.0, process) } == 0 {
            return Err(format!(
                "assign job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    pub fn terminate(&self) {
        // SAFETY: `self.0` is a valid job handle owned by `self`; terminating an already-empty job is harmless.
        unsafe { TerminateJobObject(self.0, 1) };
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid job handle owned by `self` and is closed exactly once, here in Drop.
        unsafe {
            TerminateJobObject(self.0, 1);
            CloseHandle(self.0);
        }
    }
}

pub fn resume(process: *mut c_void) -> Result<(), String> {
    // SAFETY: the caller passes a process handle it owns that was created suspended; failure is returned as NTSTATUS.
    let status = unsafe { NtResumeProcess(process) };
    if status < 0 {
        return Err(format!("resume contained child: NTSTATUS {status:#x}"));
    }
    Ok(())
}

/// Best effort: delivered only when the child shares this console.
pub fn ctrl_break(pid: u32) {
    // SAFETY: takes only a constant event id and a process-group id; failure is ignored by design (best effort).
    unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) };
}

/// True when `pid` no longer names a running process.
pub fn process_absent(pid: u32) -> Result<bool, String> {
    // SAFETY: takes only constant access flags and a pid; a null result is handled below.
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if handle.is_null() {
        // SAFETY: GetLastError only reads the calling thread's last error code, set by the failed OpenProcess above.
        return if unsafe { GetLastError() } == ERROR_INVALID_PARAMETER {
            Ok(true)
        } else {
            Err("cannot prove owned process absence; recovery refused".into())
        };
    }
    // SAFETY: `handle` is the non-null process handle opened above; a zero timeout only polls.
    let state = unsafe { WaitForSingleObject(handle, 0) };
    // SAFETY: `handle` was opened above and is closed exactly once.
    unsafe { CloseHandle(handle) };
    Ok(state == WAIT_OBJECT_0)
}
