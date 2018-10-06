use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use std::fs::File;
use std::io::Write;

macro_rules! binary(() => (if cfg!(target_pointer_width = "32") { "32" } else { "64" }));
macro_rules! feature(($name:expr) => (env::var(concat!("CARGO_FEATURE_", $name)).is_ok()));
macro_rules! switch(($condition:expr) => (if $condition { "YES" } else { "NO" }));
macro_rules! variable(($name:expr) => (env::var($name).unwrap()));

fn main() {
    let kind = if feature!("STATIC") {
        "static"
    } else {
        "dylib"
    };
    if !feature!("SYSTEM") {
        let cblas = feature!("CBLAS");
        let lapacke = feature!("LAPACKE");
        let output = PathBuf::from(variable!("OUT_DIR").replace(r"\", "/"));
        let mut f = File::create("/tmp/openblas-src-envs.txt").unwrap();
        for (key, val) in env::vars() {
            f.write(&format!("{} {}\n", key, val).into_bytes()).unwrap();
        }
        let mut make = Command::new("make");
        make.args(&["libs", "netlib", "shared"])
            .arg(format!("BINARY={}", binary!()))
            .arg(format!("{}_CBLAS=1", switch!(cblas)))
            .arg(format!("{}_LAPACKE=1", switch!(lapacke)))
            .arg(format!("-j{}", variable!("NUM_JOBS")));
        let target = match env::var("OPENBLAS_TARGET") {
            Ok(openblas_target) => {
                make.arg(format!("TARGET={}", openblas_target));
                openblas_target
            }
            _ => variable!("TARGET"),
        }.to_lowercase();
        env::remove_var("TARGET");
        let make_working_dir = PathBuf::from(&target.to_lowercase());
        if !make_working_dir.exists() {
            let make_working_dir_tmp =
                PathBuf::from(format!("{}_TMP", make_working_dir.to_str().unwrap()));
            if make_working_dir_tmp.exists() {
                fs::remove_dir_all(&make_working_dir_tmp).unwrap();
            }
            run(Command::new("cp")
                .arg("-R")
                .arg("source")
                .arg(&make_working_dir_tmp));
            fs::rename(&make_working_dir_tmp, &make_working_dir).unwrap();
        }
        match env::var("OPENBLAS_CC") {
            Ok(value) => {
                make.arg(format!("CC={}", value));
            }
            _ => {}
        }
        match env::var("OPENBLAS_FC") {
            Ok(value) => {
                make.arg(format!("FC={}", value));
            }
            _ => {}
        }
        match env::var("OPENBLAS_HOSTCC") {
            Ok(value) => {
                make.arg(format!("HOSTCC={}", value));
            }
            _ => {}
        }
        run(&mut make.current_dir(&make_working_dir));
        run(Command::new("make")
            .arg("install")
            .arg(format!("DESTDIR={}", output.display()))
            .current_dir(&make_working_dir));
        println!(
            "cargo:rustc-link-search={}",
            output.join("opt/OpenBLAS/lib").display(),
        );
    }
    println!("cargo:rustc-link-lib=dylib=gfortran");
    println!("cargo:rustc-link-lib={}=openblas", kind);
}

fn run(command: &mut Command) {
    println!("Running: `{:?}`", command);
    match command.status() {
        Ok(status) => if !status.success() {
            panic!("Failed: `{:?}` ({})", command, status);
        },
        Err(error) => {
            panic!("Failed: `{:?}` ({})", command, error);
        }
    }
}
