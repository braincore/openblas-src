use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
        let source = PathBuf::from("source");
        let output = PathBuf::from(variable!("OUT_DIR").replace(r"\", "/"));
        env::remove_var("TARGET");
        let make_working_dir: PathBuf;
        let mut make_build_cmd = Command::new("make");
        make_build_cmd
            .args(&["libs", "netlib", "shared"])
            .arg(format!("BINARY={}", binary!()))
            .arg(format!("{}_CBLAS=1", switch!(cblas)))
            .arg(format!("{}_LAPACKE=1", switch!(lapacke)))
            .arg(format!("-j{}", variable!("NUM_JOBS")));
        match env::var("OPENBLAS_TARGET") {
            Ok(target) => {
                make_build_cmd.arg(format!("TARGET={}", target));
                let canonical_target = PathBuf::from(&target.to_lowercase());
                if !canonical_target.exists() {
                    let canonical_target_tmp =
                        PathBuf::from(format!("{}_TMP", canonical_target.to_str().unwrap()));
                    if canonical_target_tmp.exists() {
                        fs::remove_dir_all(&canonical_target_tmp).unwrap();
                    }
                    run(Command::new("cp")
                        .arg("-R")
                        .arg("source")
                        .arg(&canonical_target_tmp));
                    run(Command::new("make")
                        .arg("clean")
                        .current_dir(&canonical_target_tmp));
                    fs::rename(&canonical_target_tmp, &canonical_target).unwrap();
                }
                make_working_dir = canonical_target;
            }
            _ => {
                make_working_dir = source.to_path_buf();
            }
        }
        match env::var("OPENBLAS_CC") {
            Ok(value) => {
                make_build_cmd.arg(format!("CC={}", value));
            }
            _ => {}
        }
        match env::var("OPENBLAS_FC") {
            Ok(value) => {
                make_build_cmd.arg(format!("FC={}", value));
            }
            _ => {}
        }
        match env::var("OPENBLAS_HOSTCC") {
            Ok(value) => {
                make_build_cmd.arg(format!("HOSTCC={}", value));
            }
            _ => {}
        }
        run(&mut make_build_cmd.current_dir(&make_working_dir));
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
