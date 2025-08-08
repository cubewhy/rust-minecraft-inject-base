use std::{env, process};

fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/icon.ico");
        res.set_manifest(
            r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#,
        );
        res.compile().unwrap();
    }

    // build classes
    build_entrypoint();
}

fn build_entrypoint() {
    let gradle_script_name = match env::consts::OS {
        "windows" => "gradlew.bat",
        _ => "gradlew",
    };

    let entrypoint_project_root = project_root::get_project_root()
        .unwrap()
        .join("java_stuff")
        .join("tweak-entrypoint");

    println!(
        "cargo::rerun-if-changed={}",
        entrypoint_project_root.to_string_lossy()
    );

    let gradle_script = entrypoint_project_root.join(gradle_script_name);

    // call gradle to build jar
    let mut command = process::Command::new(gradle_script);
    command.arg("build").current_dir(&entrypoint_project_root);

    let jar_path = entrypoint_project_root
        .join("build")
        .join("libs")
        .join("tweak-entrypoint.jar");

    command.spawn().unwrap().wait().unwrap();
    println!("cargo:rustc-env=JAR_PATH={}", jar_path.to_string_lossy());
}
