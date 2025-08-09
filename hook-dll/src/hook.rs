use jni::{
    JNIEnv,
    sys::{JNIEnv as RawJNIEnv, jclass, jint, jlong},
};
use once_cell::sync::Lazy;
use retour::GenericDetour;
use std::mem;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

use crate::{thread_helpers::ThreadSuspender, tweaks::load_tweaks};

type NglClearRaw =
    unsafe extern "system" fn(env: *mut RawJNIEnv, jclazz: jclass, var0: jint, var1: jlong);

static HOOK: Lazy<Result<GenericDetour<NglClearRaw>, String>> = Lazy::new(|| unsafe {
    let target_fn_ptr = find_ngl_clear_address().map_err(|e| e.to_string())?;
    let target: NglClearRaw = mem::transmute(target_fn_ptr);
    let detour_hook: NglClearRaw = mem::transmute(hooked_ngl_clear_wrapper as *const ());

    let detour = GenericDetour::new(target, detour_hook).map_err(|e| e.to_string())?;

    let _suspender = ThreadSuspender::new()?;

    println!("Enabling the hook...");
    detour
        .enable()
        .map_err(|e| e.to_string())
        .inspect_err(|e| eprintln!("Failed to enable hook {e}"))?;

    Ok(detour)
});

fn find_ngl_clear_address() -> Result<NglClearRaw, &'static str> {
    unsafe {
        let module_name = b"lwjgl64.dll\0";
        let module_handle = GetModuleHandleA(module_name.as_ptr());
        if module_handle.is_null() {
            return Err("Cannot find module lwjgl64.dll");
        }

        let func_name = b"Java_org_lwjgl_opengl_GL11_nglClear\0";
        let func_address = GetProcAddress(module_handle, func_name.as_ptr());

        match func_address {
            Some(addr) => Ok(mem::transmute(addr)),
            None => Err("Cannot find function Java_org_lwjgl_opengl_GL11_nglClear in lwjgl64.dll"),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn install_hook() {
    match &*HOOK {
        Ok(_) => {
            println!("[+] Hook installed successfully!");
        }
        Err(e) => {
            println!("[!] Failed to install hook: {}", e);
        }
    }
}

unsafe extern "system" fn hooked_ngl_clear_wrapper(
    env: *mut RawJNIEnv,
    _jclass: jclass,
    _var0: jint,
    _var1: jlong,
) {
    println!("reached hooked_ngl_clear_wrapper (before conv)");
    let typed_env = unsafe { JNIEnv::from_raw(env).unwrap() };
    // let typed_jclazz = jclazz as jclass;
    println!("reached hooked_ngl_clear_wrapper (after conv)");

    if let Ok(hook) = &*HOOK {
        println!("Uninstalling hook...");
        let _ = unsafe { hook.disable() };

        // let original_func: NglClearRaw = unsafe { mem::transmute(hook) };

        // TODO: call origin logic
        // println!("Call origin func");
        // unsafe { original_func(env, jclass, var0, var1) };
    }

    println!("Loading main hook");

    unsafe { main_hook_logic(typed_env) };
}

unsafe fn main_hook_logic(mut env: JNIEnv) {
    println!("Loading tweaks...");

    match unsafe { load_tweaks(&mut env) } {
        Ok(_) => println!("Success loaded tweaks"),
        Err(e) => eprintln!("Failed to load tweaks, reason is {e}"),
    };
}
