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
// The job handle is only used through thread-safe kernel calls.
unsafe impl Send for Job {}
unsafe impl Sync for Job {}

impl Job {
    pub fn new() -> Result<Self, String> {
        let handle = unsafe { CreateJobObjectW(null(), null()) };
        if handle.is_null() {
            return Err(format!(
                "create job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        let job = Self(handle);
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
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
        if unsafe { AssignProcessToJobObject(self.0, process) } == 0 {
            return Err(format!(
                "assign job object: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    pub fn terminate(&self) {
        unsafe { TerminateJobObject(self.0, 1) };
    }
}

impl Drop for Job {
    fn drop(&mut self) {
        unsafe {
            TerminateJobObject(self.0, 1);
            CloseHandle(self.0);
        }
    }
}

pub fn resume(process: *mut c_void) -> Result<(), String> {
    let status = unsafe { NtResumeProcess(process) };
    if status < 0 {
        return Err(format!("resume contained child: NTSTATUS {status:#x}"));
    }
    Ok(())
}

/// Best effort: delivered only when the child shares this console.
pub fn ctrl_break(pid: u32) {
    unsafe { GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, pid) };
}

/// True when `pid` no longer names a running process.
pub fn process_absent(pid: u32) -> Result<bool, String> {
    let handle = unsafe {
        OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SYNCHRONIZE,
            0,
            pid,
        )
    };
    if handle.is_null() {
        return if unsafe { GetLastError() } == ERROR_INVALID_PARAMETER {
            Ok(true)
        } else {
            Err("cannot prove owned process absence; recovery refused".into())
        };
    }
    let state = unsafe { WaitForSingleObject(handle, 0) };
    unsafe { CloseHandle(handle) };
    Ok(state == WAIT_OBJECT_0)
}
