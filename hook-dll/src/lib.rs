use std::{ffi::c_void, ptr, sync::Mutex, thread};

use bytes::Bytes;
use dialog::DialogBox;
use dll_syringe_macros::payload_procedure;
use once_cell::sync::OnceCell;
use windows_sys::{Win32::Foundation::HINSTANCE, core::BOOL};

use crate::hook::install_hook;

pub mod hook;
pub mod thread_helpers;
pub mod tweaks;
pub mod utils;

static mut H_LIB_MODULE: HINSTANCE = std::ptr::null_mut();

static mut CACHED_CLASSES: OnceCell<Mutex<Vec<(String, Bytes)>>> =
    OnceCell::with_value(Mutex::new(Vec::new()));

static mut ENTRY_POINT_CLASS: OnceCell<Mutex<String>> = OnceCell::new();
static mut ENTRY_POINT_FUNCTION_NAME: OnceCell<Mutex<String>> = OnceCell::new();
static mut ENTRY_POINT_ARGS: OnceCell<Mutex<String>> = OnceCell::new();

fn dll_main() -> anyhow::Result<()> {
    Ok(())
}

#[payload_procedure]
fn init() {}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    hinstDLL: HINSTANCE,
    fdwReason: u32,
    _lpvReserved: *mut c_void,
) -> BOOL {
    const DLL_PROCESS_ATTACH: u32 = 1;

    if fdwReason == DLL_PROCESS_ATTACH {
        // store the handle ptr
        unsafe { H_LIB_MODULE = hinstDLL };

        println!("[Tweaker] Loading dll");
        println!("[Tweaker] Brought you by earthsworth");
        println!("[Tweaker] https://lunarclient.top");

        // create a new thread to load dll
        thread::spawn(|| match dll_main() {
            Ok(_) => println!("[Tweaker] Complete loaded dll"),
            Err(e) => {
                eprintln!("[Tweaker] Failed to load dll, reason is {e}");
                // show the error dialog
                dialog::Message::new(format!("[Tweaks] Error: {e}\nBrought you by https://lunarclient.top\nOrigin developer: earthsworth"))
                .title("Faled to load dll")
                .show()
                .expect("Could not display dialog box");
            }
        });
    }

    1 // TRUE
}

#[payload_procedure]
fn set_entry_point(class_name: String, function_name: String, entry_args: String) {
    unsafe { (*ptr::addr_of!(ENTRY_POINT_CLASS)).set(Mutex::new(class_name)) }.unwrap();
    unsafe { (*ptr::addr_of!(ENTRY_POINT_FUNCTION_NAME)).set(Mutex::new(function_name)) }.unwrap();
    unsafe { (*ptr::addr_of!(ENTRY_POINT_ARGS)).set(Mutex::new(entry_args)) }.unwrap();
}

#[payload_procedure]
fn cache_class(class_name: String, class_bytes: Vec<u8>) -> bool {
    println!(
        "Caching class {class_name} (size is {}bytes)",
        class_bytes.len()
    );

    unsafe {
        (*std::ptr::addr_of_mut!(CACHED_CLASSES))
            .get_mut()
            .unwrap()
            .lock()
            .unwrap()
            .push((class_name, Bytes::from(class_bytes)));
    }
    true
}

#[payload_procedure]
fn do_tweak() {
    install_hook();
}
