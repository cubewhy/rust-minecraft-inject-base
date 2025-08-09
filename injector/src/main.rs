#![feature(trim_prefix_suffix)]
use std::{
    io::{Cursor, Read},
    process,
};

use bytes::Bytes;
use clap::Parser;
use dll_syringe::{
    Syringe,
    process::{BorrowedProcess, OwnedProcess, ProcessModule},
};

use crate::args::ProgramArgs;

pub mod args;

fn main() -> anyhow::Result<()> {
    // resolve args
    let args = ProgramArgs::parse();
    let Ok(target_process) = OwnedProcess::from_pid(args.pid) else {
        eprintln!("Cannot found process with pid {}", args.pid);
        process::exit(1);
    };

    println!("Injecting to process {}", args.pid);
    let syringe = Syringe::for_process(target_process);

    // inject to process
    let injected_payload = syringe.inject(&args.dll).unwrap();
    println!("Completed injected");

    // now we injected to the game

    // init dll
    println!("Init dll");
    init_dll(&args, &syringe, injected_payload)?;

    // now the dll hooked java
    // we can load classes into the JVM
    println!("Loading classes to memory");
    cache_classes(&syringe, injected_payload)?;

    println!("Loading tweaks...");
    // call dll to invoke the entrypoint in org.cubewhy.Tweaker class
    install_tweak(&syringe, injected_payload)?;

    // eject the payload from the target (optional)
    // syringe.eject(injected_payload)?;

    Ok(())
}

fn init_dll(
    args: &ProgramArgs,
    syringe: &Syringe,
    injected_payload: ProcessModule<BorrowedProcess<'_>>,
) -> anyhow::Result<()> {
    let remote_init_func =
        unsafe { syringe.get_payload_procedure::<fn() -> ()>(injected_payload, "init") }?.unwrap();
    remote_init_func.call()?;

    let remote_set_entry_point_func = unsafe {
        syringe.get_payload_procedure::<fn(class_name: String, function_name: String, entry_args: String) -> ()>(
            injected_payload,
            "set_entry_point",
        )
    }?
    .unwrap();

    // parse entrypoint
    // org.cubewhy.[Entrypoint] -> org/cubewhy, Entrypoint
    let splitted: Vec<_> = args.entrypoint.split(".").map(|s| s.to_string()).collect();
    let entry_func_name = splitted.last().unwrap();
    let entry_class_name = args
        .entrypoint
        .trim_suffix(format!(".{entry_func_name}").as_str())
        .replace(".", "/")
        .to_string();
    remote_set_entry_point_func.call(&entry_class_name, &entry_func_name, &args.args)?;

    Ok(())
}

fn cache_classes(
    syringe: &Syringe,
    injected_payload: ProcessModule<BorrowedProcess<'_>>,
) -> anyhow::Result<()> {
    let remote_cache_class_func = unsafe {
        syringe
            .get_payload_procedure::<fn(String, Vec<u8>) -> bool>(injected_payload, "cache_class")
    }?
    .unwrap();

    // load the jar
    let jar_bytes: Bytes = Bytes::from_static(include_bytes!(env!("JAR_PATH")));

    // TODO: parse the jar
    let mut jar_archive = zip::ZipArchive::new(Cursor::new(jar_bytes))?;

    let file_names: Vec<_> = jar_archive.file_names().map(|s| s.to_string()).collect();
    for file_name in file_names {
        let mut entry = jar_archive.by_name(&file_name)?;
        if entry.is_dir() || !file_name.ends_with(".class") {
            continue; // skip dirs
        }
        let class_name = file_name.trim_suffix(".class").to_string();
        // read class bytes
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        // add the class
        // call the function to load class to the jvm
        println!(
            "Add class {class_name} into remote memory ({}bytes)",
            buf.len()
        );
        remote_cache_class_func.call(&class_name, &buf)?;
    }

    // TODO: resources file support

    Ok(())
}

fn install_tweak(
    syringe: &Syringe,
    injected_payload: ProcessModule<BorrowedProcess<'_>>,
) -> anyhow::Result<()> {
    let remote_init_func =
        unsafe { syringe.get_payload_procedure::<fn() -> ()>(injected_payload, "do_tweak") }?
            .unwrap();
    remote_init_func.call()?;

    Ok(())
}
