use std::process::Command;

fn main() {
    cargo_build("../stage1")
}

fn cargo_build(path: &str) {
    println!("cargo:rerun-if-changed={path}");
    let target_dir = format!("{}/embeds", std::env::var("OUT_DIR").unwrap());

    let output = Command::new("cargo")
        .arg("build")
        .arg("--profile") // for stripping
        .arg("embed")
        .arg("--target-dir")
        .arg(target_dir)
        .current_dir(path)
        .spawn()
        .unwrap()
        .wait_with_output()
        .unwrap();
    if !output.status.success() {
        panic!(
            "Building {} failed.\nStdout: {}\nStderr: {}",
            path,
            String::from_utf8_lossy(&output.stdout[..]),
            String::from_utf8_lossy(&output.stderr[..]),
        )
    }
}
