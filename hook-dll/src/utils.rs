use jni::{JNIEnv, objects::JClass};

/// # Safety
/// This function is unsafe because it deals with raw JNI pointers and operations.
pub unsafe fn load_class_bytes<'a>(
    jni: &'a mut JNIEnv,
    class_name: &str,
    class_bytes: &[u8],
) -> jni::errors::Result<JClass<'a>> {
    let thread_class = jni.find_class("java/lang/Thread")?;
    let current_thread_obj = jni
        .call_static_method(thread_class, "currentThread", "()Ljava/lang/Thread;", &[])?
        .l()?;
    let class_loader_obj = jni
        .call_method(
            current_thread_obj,
            "getContextClassLoader",
            "()Ljava/lang/ClassLoader;",
            &[],
        )?
        .l()?;

    jni.define_class(class_name, &class_loader_obj, class_bytes)
}
