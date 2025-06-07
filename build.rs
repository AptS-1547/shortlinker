use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=admin-panel/dist");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("static_files.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    // 获取项目根目录
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_path = Path::new(&manifest_dir).join("admin-panel/dist");

    if !dist_path.exists() {
        eprintln!("Warning: admin-panel/dist directory not found!");
        eprintln!("Please build the admin panel first:");
        eprintln!("  cd admin-panel && npm run build");

        create_fallback_files(&dist_path);
        generate_fallback_static_files(&mut f);
    } else {
        generate_static_files_mapping(&mut f, &dist_path, &manifest_dir);
    }
}

fn generate_static_files_mapping<W: Write>(f: &mut W, dist_path: &Path, _manifest_dir: &str) {
    writeln!(f, "// Auto-generated static files mapping").unwrap();
    writeln!(f, "{{").unwrap(); // 开始代码块

    // 处理 assets 目录
    let assets_path = dist_path.join("assets");
    if assets_path.exists() {
        if let Ok(entries) = fs::read_dir(&assets_path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name_str = file_name.to_string_lossy();
                let file_path = entry.path();

                // 只处理实际的资源文件，跳过 .map 文件
                if file_name_str.ends_with(".css")
                    || (file_name_str.ends_with(".js") && !file_name_str.ends_with(".map"))
                {
                    let key = format!("assets/{}", file_name_str);
                    // 使用绝对路径，生成单行语句
                    let absolute_path = file_path.to_string_lossy().replace("\\", "/"); // 处理 Windows 路径

                    writeln!(
                        f,
                        r#"    files.insert("{}".to_string(), include_bytes!("{}"));"#,
                        key, absolute_path
                    )
                    .unwrap();

                    println!("cargo:rerun-if-changed={}", absolute_path);
                    println!("  ✓ Added: {}", key);
                }

                // 如果是 .map 文件，也包含但标记为可选
                if file_name_str.ends_with(".map") {
                    let key = format!("assets/{}", file_name_str);
                    let absolute_path = file_path.to_string_lossy().replace("\\", "/");

                    writeln!(
                        f,
                        r#"    files.insert("{}".to_string(), include_bytes!("{}"));"#,
                        key, absolute_path
                    )
                    .unwrap();

                    println!("cargo:rerun-if-changed={}", absolute_path);
                    println!("  ✓ Added: {} (source map)", key);
                }
            }
        }
    }

    // 处理其他可能的静态文件
    for entry in ["manifest.json", "robots.txt"].iter() {
        let file_path = dist_path.join(entry);
        if file_path.exists() {
            let absolute_path = file_path.to_string_lossy().replace("\\", "/");

            writeln!(
                f,
                r#"    files.insert("{}".to_string(), include_bytes!("{}"));"#,
                entry, absolute_path
            )
            .unwrap();

            println!("cargo:rerun-if-changed={}", absolute_path);
            println!("  ✓ Added: {}", entry);
        }
    }

    writeln!(f, "}}").unwrap(); // 结束代码块
}

fn generate_fallback_static_files<W: Write>(f: &mut W) {
    writeln!(f, "// Fallback static files (admin panel not built)").unwrap();
    writeln!(f, "{{").unwrap(); // 开始代码块
    writeln!(
        f,
        r#"    files.insert("assets/fallback.css".to_string(), b"/* Admin panel not built */");"#
    )
    .unwrap();
    writeln!(f, r#"    files.insert("assets/fallback.js".to_string(), b"console.log('Admin panel not built');");"#).unwrap();
    writeln!(f, "}}").unwrap(); // 结束代码块
}

fn create_fallback_files(dist_path: &Path) {
    fs::create_dir_all(dist_path).expect("Failed to create dist directory");

    let fallback_html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Short Linker - Admin Panel Not Built</title>
    <style>
        body { 
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 600px; 
            margin: 100px auto; 
            padding: 20px;
            text-align: center;
        }
        .warning { 
            background: #fff3cd; 
            border: 1px solid #ffeaa7; 
            padding: 20px; 
            border-radius: 8px; 
            margin: 20px 0;
        }
        code { 
            background: #f1f3f4; 
            padding: 2px 6px; 
            border-radius: 4px; 
            font-family: monospace;
        }
    </style>
</head>
<body>
    <h1>🔗 Short Linker</h1>
    <div class="warning">
        <h2>⚠️ Admin Panel Not Built</h2>
        <p>The admin panel needs to be built before running the server.</p>
        <p>Please run the following commands:</p>
        <p><code>cd admin-panel && npm install && npm run build</code></p>
    </div>
</body>
</html>"#;

    fs::write(dist_path.join("index.html"), fallback_html)
        .expect("Failed to write fallback index.html");

    // 创建空的 favicon
    fs::write(dist_path.join("favicon.ico"), &[]).expect("Failed to write fallback favicon");

    fs::create_dir_all(dist_path.join("assets")).expect("Failed to create assets directory");
}
