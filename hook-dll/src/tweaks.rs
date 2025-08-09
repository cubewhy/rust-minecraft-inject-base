use anyhow::anyhow;
use jni::{JNIEnv, objects::JValue};
use std::ptr;

use crate::{
    CACHED_CLASSES, ENTRY_POINT_ARGS, ENTRY_POINT_CLASS, ENTRY_POINT_FUNCTION_NAME,
    utils::load_class_bytes,
};

pub unsafe fn load_tweaks(env: &mut JNIEnv) -> anyhow::Result<()> {
    // get the jvm
    let jvm = env.get_java_vm()?;
    let mut guard = jvm.attach_current_thread().unwrap();

    let cached_classes = unsafe {
        (*ptr::addr_of!(CACHED_CLASSES))
            .get()
            .unwrap()
            .lock()
            .unwrap()
    };

    println!("Injecting classes into VM");
    // load class into classloader
    for (class_name, bytes) in cached_classes.iter() {
        // TODO: only define class if it not found in the classloader
        // load the class
        println!("Inject class {class_name} ({}bytes)", bytes.len());
        unsafe { load_class_bytes(&mut guard, class_name, bytes) }?;
    }

    let entry_class_name = unsafe {
        (*ptr::addr_of!(ENTRY_POINT_CLASS))
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .to_string()
    };

    let entry_method_name = unsafe {
        (*ptr::addr_of!(ENTRY_POINT_FUNCTION_NAME))
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .to_string()
    };

    // find tweaker entrypoint
    let Ok(tweaker_entry) = guard.find_class(&entry_class_name) else {
        eprintln!("Failed to load entry class {entry_class_name}");
        return Err(anyhow!("Failed to load entry class"));
    };
    // call the entry

    let entry_arg = unsafe {
        (*ptr::addr_of!(ENTRY_POINT_ARGS))
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .to_string()
    };

    println!("Call the entry {entry_class_name}.{entry_method_name}");

    let entry_arg = guard.new_string(entry_arg.as_str())?;

    guard.call_static_method(
        tweaker_entry,
        entry_method_name,
        "(Ljava/lang/String;)V",
        &[JValue::Object(&entry_arg.into())],
    )?;

    Ok(())
}
