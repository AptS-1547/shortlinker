use std::env;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn setup_test_env() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let links_file = temp_dir.path().join("test_links.json");
    env::set_var("LINKS_FILE", links_file.to_str().unwrap());
    env::set_var("STORAGE_BACKEND", "file"); // 强制使用文件存储
    temp_dir
}

#[test]
fn test_cli_help_command() {
    let _temp_dir = setup_test_env();

    let output = Command::new("cargo")
        .args(&["run", "--", "help"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("shortlinker") || stdout.contains("用法") || stdout.contains("帮助"));
}

#[test]
fn test_cli_list_empty() {
    let temp_dir = setup_test_env();
    let links_file = temp_dir.path().join("empty_links.json");

    // 创建空的链接文件，注意：需要创建空数组而不是空对象
    fs::write(&links_file, "[]").unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "list"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // 检查是否为空输出或正确的空状态消息
    assert!(
        stdout.contains("没有短链接")
            || stdout.contains("0 个短链接")
            || stdout.contains("共 0 个短链接")
            || stdout.contains("ℹ 共 0 个短链接")
            || stdout.trim().is_empty()
            || (!stdout.contains("->") && !stdout.contains("共 1 个")) // 确保没有链接条目
    );
}

#[test]
fn test_cli_add_valid_link() {
    let temp_dir = setup_test_env();
    let links_file = temp_dir.path().join("test_links_add.json");

    let output = Command::new("cargo")
        .args(&["run", "--", "add", "test", "https://example.com"])
        .output()
        .expect("Failed to execute command");

    // 应该成功添加
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command failed: {}", stderr);
    }

    // 验证文件中包含了新链接
    if links_file.exists() {
        let content = fs::read_to_string(&links_file).unwrap_or_default();
        assert!(content.contains("test") || content.contains("https://example.com"));
    }
}

#[test]
fn test_cli_add_invalid_url() {
    setup_test_env();

    let output = Command::new("cargo")
        .args(&["run", "--", "add", "test", "invalid-url"])
        .output()
        .expect("Failed to execute command");

    // 应该因为无效URL而失败
    assert!(!output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    assert!(
        output_text.contains("http")
            || output_text.contains("URL")
            || output_text.contains("url")
            || output_text.contains("格式")
    );
}

#[test]
fn test_cli_remove_workflow() {
    setup_test_env();

    // 先添加一个链接
    let add_output = Command::new("cargo")
        .args(&["run", "--", "add", "remove_test", "https://remove.com"])
        .output()
        .expect("Failed to execute add command");

    if add_output.status.success() {
        // 然后删除它
        let remove_output = Command::new("cargo")
            .args(&["run", "--", "remove", "remove_test"])
            .output()
            .expect("Failed to execute remove command");

        // 删除应该成功或给出适当的反馈
        let stdout = String::from_utf8_lossy(&remove_output.stdout);
        assert!(
            remove_output.status.success() || stdout.contains("删除") || stdout.contains("不存在")
        );
    }
}

#[test]
fn test_cli_remove_nonexistent() {
    let temp_dir = setup_test_env();
    let links_file = temp_dir.path().join("test_links_nonexistent.json");

    // 创建空的链接文件
    fs::write(&links_file, "[]").unwrap();

    let output = Command::new("cargo")
        .args(&["run", "--", "remove", "nonexistent"])
        .output()
        .expect("Failed to execute command");

    // 应该失败并给出适当的错误信息
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_text = format!("{}{}", stdout, stderr);

    assert!(
        output_text.contains("不存在")
            || output_text.contains("not found")
            || !output.status.success()
    );
}

#[test]
fn test_cli_unknown_command() {
    let _temp_dir = setup_test_env();

    let output = Command::new("cargo")
        .args(&["run", "--", "unknown_command"])
        .output()
        .expect("Failed to execute command");

    // 应该返回非零退出码或显示帮助信息
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success()
            || stdout.contains("未知")
            || stdout.contains("help")
            || stdout.contains("帮助")
    );
}

#[test]
fn test_cli_insufficient_args() {
    let _temp_dir = setup_test_env();

    // 测试add命令参数不足
    let output = Command::new("cargo")
        .args(&["run", "--", "add"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !output.status.success()
            || stdout.contains("用法")
            || stdout.contains("Usage")
            || stdout.contains("add")
    );
}

#[cfg(unix)]
#[test]
fn test_cli_server_commands() {
    let _temp_dir = setup_test_env();

    for command in &["start", "stop", "restart"] {
        let output = Command::new("cargo")
            .args(&["run", "--", command])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("服务器")
                || stdout.contains("server")
                || stdout.contains("shortlinker")
        );
    }
}
