fn main() {
    std::fs::remove_dir_all("components/.uibeam")
        .ok(); // Ignore error if the directory does not exist
    std::fs::create_dir("components/.uibeam")
        .expect("Failed to create .uibeam directory");

    // wasm-pack build components --target web --out-dir '.uibeam' --out-name 'lasers'
    let output = std::process::Command::new("wasm-pack")
        .args(["build", "components", "--target", "web", "--out-dir", ".uibeam", "--out-name", "lasers"])
        .output()
        .expect("Failed to run wasm-pack build");

    std::fs::write("log.stdout", output.stdout).expect("Failed to write stdout to log");
    std::fs::write("log.stderr", output.stderr).expect("Failed to write stderr to log");
}
