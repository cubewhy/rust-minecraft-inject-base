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

    let bump_project_root = project_root::get_project_root()
        .unwrap()
        .join("java_stuff")
        .join("class-version-bumper");

    println!(
        "cargo::rerun-if-changed={}",
        entrypoint_project_root.to_string_lossy()
    );

    let entrypoint_gradle_script = entrypoint_project_root.join(gradle_script_name);

    // call gradle to build jar
    let mut build_command = process::Command::new(entrypoint_gradle_script);
    build_command
        .arg("build")
        .current_dir(&entrypoint_project_root);

    let jar_path = entrypoint_project_root
        .join("build")
        .join("libs")
        .join("tweak-entrypoint.jar");

    build_command.spawn().unwrap().wait().unwrap();

    // bump class file version
    let bumped_jar_path = entrypoint_project_root
        .join("build")
        .join("libs")
        .join("tweak-entrypoint-bumped.jar");

    let bump_gradle_script = bump_project_root.join(gradle_script_name);

    // call gradle to bump jar
    let mut bump_command = process::Command::new(bump_gradle_script);
    bump_command
        .arg("run")
        .arg("--args")
        .arg(format!(
            "\"{}\" \"{}\"",
            jar_path.to_string_lossy(),
            bumped_jar_path.to_string_lossy()
        ))
        .current_dir(&bump_project_root);

    bump_command.spawn().unwrap().wait().unwrap();

    println!(
        "cargo:rustc-env=JAR_PATH={}",
        bumped_jar_path.to_string_lossy()
    );
}
