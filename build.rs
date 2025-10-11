use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // 重新构建条件
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");

    // 设置构建后钩子环境变量
    println!("cargo:rustc-env=ENABLE_BINARY_COPY=1");
    // 如果设置了环境变量，则在构建后复制二进制文
    // if env::var("SCAFFOLD_COPY_BINARY").unwrap_or_default() == "1" {
    copy_binary_to_root();
    // }
}

fn copy_binary_to_root() {
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let binary_name = get_binary_name();

    // 获取项目根目录
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let project_root = Path::new(&manifest_dir);

    // 构建源文件和目标文件路径
    let source_path = project_root
        .join("target")
        .join(&profile)
        .join(&binary_name);
    let dest_path = project_root.join(&binary_name);

    if source_path.exists() {
        match fs::copy(&source_path, &dest_path) {
            Ok(_) => {
                println!(
                    "cargo:warning=Binary copied to project root: {}",
                    dest_path.display()
                );
                // 验证复制的二进制文件
                if verify_binary(&dest_path) {
                    println!("cargo:warning=Binary verification successful");
                } else {
                    println!("cargo:warning=Binary copied but verification failed");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to copy binary: {e}");
            }
        }
    } else {
        println!(
            "cargo:warning=Source binary not found: {}",
            source_path.display()
        );
    }
}

fn get_binary_name() -> String {
    if cfg!(target_os = "windows") {
        "scafgen.exe".to_string()
    } else {
        "scafgen".to_string()
    }
}

fn verify_binary(binary_path: &Path) -> bool {
    if !binary_path.exists() {
        return false;
    }

    // 尝试运行 --version 命令验证二进制文件
    match Command::new(binary_path).arg("--version").output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
