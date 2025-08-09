use std::{mem, thread, time::Duration};
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
    System::{
        Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First, Thread32Next,
        },
        Threading::{
            GetCurrentProcessId, GetCurrentThreadId, OpenThread, ResumeThread, SuspendThread,
            THREAD_SUSPEND_RESUME,
        },
    },
};

pub struct ThreadSuspender {
    suspended_threads: Vec<HANDLE>,
}

impl ThreadSuspender {
    pub fn new() -> Result<Self, String> {
        let mut suspended_threads: Vec<HANDLE> = Vec::new();
        let current_pid = unsafe { GetCurrentProcessId() };
        let current_tid = unsafe { GetCurrentThreadId() };

        // snapshot all threads
        let h_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if h_snapshot == INVALID_HANDLE_VALUE {
            return Err("CreateToolhelp32Snapshot failed".to_string());
        }

        // init THREADENTRY32 properly
        let mut te32: THREADENTRY32 = unsafe { mem::zeroed() };
        te32.dwSize = mem::size_of::<THREADENTRY32>() as u32;

        // first
        if unsafe { Thread32First(h_snapshot, &mut te32) } == 0 {
            unsafe { CloseHandle(h_snapshot) };
            return Err("Thread32First failed".to_string());
        }

        loop {
            // only threads of current process, and not current thread
            if te32.th32OwnerProcessID == current_pid && te32.th32ThreadID != current_tid {
                // OpenThread returns 0 on failure
                let h_thread: HANDLE =
                    unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, te32.th32ThreadID) };

                if h_thread != std::ptr::null_mut() {
                    // Try to suspend - SuspendThread returns u32::MAX (or -1) on error
                    let suspend_res = unsafe { SuspendThread(h_thread) };
                    if suspend_res != u32::MAX {
                        // success: record handle so we can Resume + Close later
                        suspended_threads.push(h_thread);
                    } else {
                        // suspend failed for this handle: cleanup and return error
                        // resume & close already suspended ones
                        for &h in &suspended_threads {
                            let _ = unsafe { ResumeThread(h) };
                            unsafe { CloseHandle(h) };
                        }
                        // close this thread handle (if non-zero)
                        unsafe { CloseHandle(h_thread) };
                        unsafe { CloseHandle(h_snapshot) };
                        return Err(format!("Failed to suspend thread {}", te32.th32ThreadID));
                    }
                } // else OpenThread failed => skip that thread (could be system or race), continue
            }

            if unsafe { Thread32Next(h_snapshot, &mut te32) } == 0 {
                break;
            }
        }

        // small delay to let threads actually settle (optional, short)
        thread::sleep(Duration::from_millis(5));

        unsafe { CloseHandle(h_snapshot) };
        println!("Suspended threads");
        Ok(Self { suspended_threads })
    }
}

impl Drop for ThreadSuspender {
    fn drop(&mut self) {
        for &h_thread in &self.suspended_threads {
            unsafe {
                // Best-effort resume; ignore return value
                let _ = ResumeThread(h_thread);
                CloseHandle(h_thread);
            }
        }
        // clear vector to avoid double-closing if Drop somehow called twice
        self.suspended_threads.clear();
        println!("Resumed threads.");
    }
}
