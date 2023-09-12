use std::{env, path::PathBuf};

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let crate_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(&format!("lib{crate_name}.h"))
        .display()
        .to_string();

    // check if cbindgen is in path or panic
    let _cbindgen = match which::which("cbindgen") {
        Ok(path) => path,
        Err(_) => panic!("cbindgen not found in path"),
    };

    let mut config: cbindgen::Config = Default::default();
    config.language = cbindgen::Language::C;
    cbindgen::generate_with_config(&crate_dir, config)
        .expect("Unable to generate bindings")
        .write_to_file(&output_file);
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(dir) = env::var("OUT_DIR") {
        PathBuf::from(dir).join("../../..")
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
