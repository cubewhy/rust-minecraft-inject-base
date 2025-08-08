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

        let h_snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if h_snapshot == INVALID_HANDLE_VALUE {
            return Err("CreateToolhelp32Snapshot failed".to_string());
        }

        let mut te32 = THREADENTRY32 {
            dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
            ..Default::default()
        };

        if unsafe { Thread32First(h_snapshot, &mut te32) } == 0 {
            unsafe { CloseHandle(h_snapshot) };
            return Err("Thread32First failed".to_string());
        }

        loop {
            if te32.th32OwnerProcessID == current_pid && te32.th32ThreadID != current_tid {
                let h_thread: HANDLE =
                    unsafe { OpenThread(THREAD_SUSPEND_RESUME, 0, te32.th32ThreadID) };

                if h_thread != std::ptr::null_mut() {
                    if unsafe { SuspendThread(h_thread) } != u32::MAX {
                        suspended_threads.push(h_thread);
                    } else {
                        for &h in &suspended_threads {
                            unsafe { ResumeThread(h) };
                        }
                        unsafe { CloseHandle(h_thread) };
                        unsafe { CloseHandle(h_snapshot) };
                        return Err(format!("Failed to suspend thread {}", te32.th32ThreadID));
                    }
                }
            }
            if unsafe { Thread32Next(h_snapshot, &mut te32) } == 0 {
                break;
            }
        }

        unsafe { CloseHandle(h_snapshot) };
        Ok(Self { suspended_threads })
    }
}

impl Drop for ThreadSuspender {
    fn drop(&mut self) {
        for &h_thread in &self.suspended_threads {
            unsafe {
                ResumeThread(h_thread);
                CloseHandle(h_thread);
            }
        }
    }
}
