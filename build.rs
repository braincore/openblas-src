use std::env;
use std::fs;
use std::path::{
    Path,
    PathBuf,
};
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
        // The target triple provided by cargo --target is not compatible with
        // OpenBLAS targets.
        env::remove_var("TARGET");
        // The working directory for both 'make' and 'make install'
        let make_working_dir: PathBuf;
        let mut make_build_cmd = Command::new("make");
        make_build_cmd
            .args(&["libs", "netlib", "shared"])
            .arg(format!("BINARY={}", binary!()))
            .arg(format!("{}_CBLAS=1", switch!(cblas)))
            .arg(format!("{}_LAPACKE=1", switch!(lapacke)))
            .arg(format!("-j{}", variable!("NUM_JOBS")));
        // Options for cross compilation
        match env::var("OPENBLAS_TARGET") {
            Ok(target) => {
                make_build_cmd.arg(format!("TARGET={}", target));

                // Since target is user-specified, we relax strictness on case.
                let canonical_target = target.to_uppercase();
                let canonical_target_path = Path::new(&canonical_target);

                // Check if there's already a dedicated folder for compiling this target.
                // If not, create it by duplicating the source folder.
                if !Path::new(&canonical_target).exists() {
                    let canonical_target_tmp = format!("{}_TMP", canonical_target);
                    // If the tmp folder exists, the copy must have failed during
                    // a previous invocation so remove it.
                    let canonical_target_tmp_path = Path::new(&canonical_target_tmp);
                    if canonical_target_tmp_path.exists() {
                        fs::remove_dir_all(canonical_target_tmp_path).unwrap();
                    }
                    run(
                        Command::new("cp")
                            .arg("-R")
                            .arg("source")
                            .arg(&canonical_target_tmp)
                    );
                    // Remove any existing compiled files since the source folder
                    // is still used for non-cross-compilations.
                    run(
                        Command::new("make")
                            .arg("clean")
                            .current_dir(&canonical_target_tmp_path)
                    );
                    // Atomic move
                    fs::rename(canonical_target_tmp_path, canonical_target_path).unwrap();
                }
                make_working_dir = canonical_target_path.to_path_buf();
            }
            _ => {
                make_working_dir = source.to_path_buf();
            },
        }
        match env::var("OPENBLAS_CC") {
            Ok(val) => { make_build_cmd.arg(format!("CC={}", val)); }
            _ => {},
        }
        match env::var("OPENBLAS_FC") {
            Ok(val) => { make_build_cmd.arg(format!("FC={}", val)); }
            _ => {},
        }
        match env::var("OPENBLAS_HOSTCC") {
            Ok(val) => { make_build_cmd.arg(format!("HOSTCC={}", val)); }
            _ => {},
        }
        run(&mut make_build_cmd.current_dir(&make_working_dir));
        run(
            Command::new("make")
                .arg("install")
                .arg(format!("DESTDIR={}", output.display()))
                .current_dir(&make_working_dir),
        );
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
