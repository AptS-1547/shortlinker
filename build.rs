use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=admin-panel/dist");

    // 获取项目根目录
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_path = Path::new(&manifest_dir).join("admin-panel/dist");

    if !dist_path.exists() {
        eprintln!("Warning: admin-panel/dist directory not found!");
        eprintln!("Please build the admin panel first:");
        eprintln!("  cd admin-panel && npm run build");

        create_fallback_files(&dist_path);
    }
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
        <p><code>cd admin-panel && yarn && yarn build</code></p>
    </div>
</body>
</html>"#;

    fs::write(dist_path.join("index.html"), fallback_html)
        .expect("Failed to write fallback index.html");

    // 创建空的 favicon
    fs::write(dist_path.join("favicon.ico"), []).expect("Failed to write fallback favicon");

    fs::create_dir_all(dist_path.join("assets")).expect("Failed to create assets directory");
}
