use std::env;
use std::ffi::OsString;
use std::iter;
use std::path::Path;
use std::process::{self, Command, ExitStatus, Stdio};

fn main() {
    let rustc = cargo_env_var("RUSTC");
    let out_dir = cargo_env_var("OUT_DIR");
    let probefile = Path::new("build").join("probe.rs");

    let rustc_wrapper = env::var_os("RUSTC_WRAPPER").filter(|wrapper| !wrapper.is_empty());
    let mut rustc = rustc_wrapper.into_iter().chain(iter::once(rustc));
    let mut cmd = Command::new(rustc.next().unwrap());
    cmd.args(rustc);

    cmd.stderr(Stdio::null())
        .arg("--edition=2021")
        .arg("--crate-type=bin")
        .arg("--emit=link")
        .arg("-Clink-arg=-Wl,-z,nostart-stop-gc")
        .arg("--cap-lints=allow")
        .arg("--out-dir")
        .arg(out_dir)
        .arg(probefile);

    if let Some(target) = env::var_os("TARGET") {
        cmd.arg("--target").arg(target);
    }

    if let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") {
        if !rustflags.is_empty() {
            for arg in rustflags.split('\x1f') {
                cmd.arg(arg);
            }
        }
    }

    if cmd.status().as_ref().map_or(false, ExitStatus::success) {
        // https://github.com/rust-lang/cargo/issues/9554
        println!("cargo:rustc-link-arg=-Wl,-z,nostart-stop-gc");
    }
}

fn cargo_env_var(key: &str) -> OsString {
    env::var_os(key).unwrap_or_else(|| {
        eprintln!("Environment variable ${key} is not set during execution of build script");
        process::exit(1);
    })
}
