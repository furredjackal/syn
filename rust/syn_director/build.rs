use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Check if storylet codegen is explicitly enabled
    // This is disabled by default to make builds fast (especially for flutter_rust_bridge codegen)
    if env::var("SYN_DIRECTOR_ENABLE_BUILD_CODEGEN").is_err() {
        println!("cargo:warning=Skipping syn_director storylet compilation (SYN_DIRECTOR_ENABLE_BUILD_CODEGEN not set)");
        return;
    }
    
    println!("cargo:rerun-if-changed=../../storylets");
    
    // Get the manifest directory (rust/syn_director)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = PathBuf::from(&manifest_dir);
    
    // Paths
    let project_root = manifest_path.parent().unwrap().parent().unwrap();
    let storylets_dir = project_root.join("storylets");
    let data_dir = manifest_path.join("data");
    let output_bin = data_dir.join("storylets.bin");
    
    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
    
    // Only compile if storylets directory exists
    if !storylets_dir.exists() {
        println!("cargo:warning=Storylets directory not found, skipping compilation");
        return;
    }
    
    println!("cargo:warning=Compiling storylets from {:?} to {:?}", storylets_dir, output_bin);
    
    // Run storyletc compiler
    let status = Command::new("cargo")
        .args([
            "run",
            "--bin",
            "storyletc",
            "--",
            "--input",
            storylets_dir.to_str().unwrap(),
            "--output",
            output_bin.to_str().unwrap(),
        ])
        .current_dir(project_root.join("rust"))
        .status();
    
    match status {
        Ok(status) if status.success() => {
            println!("cargo:warning=Successfully compiled storylets");
        }
        Ok(status) => {
            println!("cargo:warning=Storylet compilation failed with status: {}", status);
        }
        Err(e) => {
            println!("cargo:warning=Failed to run storyletc: {}", e);
        }
    }
}
