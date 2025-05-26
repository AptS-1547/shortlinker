use std::process::Command;
use std::env;

fn main() {
    // 在 CI 环境中跳过 build.rs 的交叉编译
    if env::var("CI").is_ok() {
        println!("cargo:warning=Running in CI, skipping build.rs cross compilation");
        return;
    }

    // 只在 release 模式下进行交叉编译
    if env::var("PROFILE").unwrap_or_default() != "release" {
        return;
    }

    // 检查是否安装了 cross
    if !is_cross_installed() {
        println!("cargo:warning=cross not installed, skipping cross compilation");
        return;
    }

    let targets = vec![
        "x86_64-pc-windows-gnu",
        "x86_64-apple-darwin",
        "x86_64-unknown-linux-gnu",
        "aarch64-apple-darwin",
        "aarch64-unknown-linux-gnu",
    ];

    // 获取当前目标架构，避免重复编译
    let current_target = env::var("TARGET").unwrap_or_default();
    
    for target in targets {
        if target == current_target {
            continue; // 跳过当前目标，避免重复编译
        }

        println!("cargo:warning=Building for target: {}", target);
        
        let output = Command::new("cross")
            .args(&["build", "--release", "--target", target])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    println!("cargo:warning=Successfully built for {}", target);
                } else {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("cargo:warning=Failed to build for {}: {}", target, stderr);
                }
            }
            Err(e) => {
                println!("cargo:warning=Error building for {}: {}", target, e);
            }
        }
    }
}

fn is_cross_installed() -> bool {
    Command::new("cross")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}