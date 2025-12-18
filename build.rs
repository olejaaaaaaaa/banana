use std::path::Path;
use std::process::Command;
use std::fs::{create_dir_all, read_dir};


fn main() {

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);

    let shaders_dir = project_root.join("./src/shared/shaders");
    let spv_output_dir = project_root.join("./src/shared/shaders/spv");

    create_dir_all(&spv_output_dir).unwrap();
    
    println!("cargo:rerun-if-changed={}", shaders_dir.display());

    if !shaders_dir.exists() {
        println!("cargo:warning=Shaders directory not found: {}", shaders_dir.display());
        return;
    }

    let dirs = read_dir(&shaders_dir).unwrap();
    for entry in dirs {
        let entry = entry.unwrap();
        let shader_path = entry.path();
        
        if shader_path.is_dir() {
            continue;
        }

        let original_name = entry.file_name().to_str().unwrap().to_string();

        if !original_name.contains('.') { continue; }

        let words: Vec<&str> = original_name.split('.').collect();

        if words.len() != 2 { continue; }

        let name = words[0];
        let format = words[1];

        if format == "frag" || format == "vert" {
            let out_name = format!("{}-{}.spv", name, format);
            let output_path = spv_output_dir.join(&out_name);

            let res = Command::new("glslc")
                .arg(&shader_path)
                .arg("-o")
                .arg(&output_path)
                .spawn()
                .unwrap()
                .wait()
                .unwrap();

            if !res.success() {
                panic!("Failed to compile shader: {}", original_name);
            } else {
                println!("cargo:warning=Compiled: {} -> {}", original_name, out_name);
            }
        }
    }

    println!("cargo:warning=Shader compilation completed!");
}