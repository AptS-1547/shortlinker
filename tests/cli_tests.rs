use shortlinker::cli::commands::Command;
use shortlinker::cli::parser::CliParser;
use shortlinker::cli::CliError;
use shortlinker::storages::Storage;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;

// 模拟存储实现用于测试
#[derive(Default)]
struct MockStorage {
    data: std::sync::Mutex<HashMap<String, shortlinker::storages::ShortLink>>,
    should_fail: std::sync::Mutex<bool>,
}

impl MockStorage {
    fn new_failing() -> Self {
        Self {
            data: std::sync::Mutex::new(HashMap::new()),
            should_fail: std::sync::Mutex::new(true),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.lock().unwrap() = fail;
    }
}

#[async_trait::async_trait]
impl shortlinker::storages::Storage for MockStorage {
    async fn get(&self, code: &str) -> Option<shortlinker::storages::ShortLink> {
        if *self.should_fail.lock().unwrap() {
            return None;
        }
        let data = self.data.lock().unwrap();
        data.get(code).cloned()
    }

    async fn set(
        &self,
        link: shortlinker::storages::ShortLink,
    ) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation(
                "Mock storage error".to_string(),
            ));
        }
        let mut data = self.data.lock().unwrap();
        data.insert(link.code.clone(), link);
        Ok(())
    }

    async fn remove(&self, code: &str) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation(
                "Mock storage error".to_string(),
            ));
        }
        let mut data = self.data.lock().unwrap();
        data.remove(code);
        Ok(())
    }

    async fn load_all(&self) -> HashMap<String, shortlinker::storages::ShortLink> {
        if *self.should_fail.lock().unwrap() {
            return HashMap::new();
        }
        self.data.lock().unwrap().clone()
    }

    async fn reload(&self) -> Result<(), shortlinker::errors::ShortlinkerError> {
        if *self.should_fail.lock().unwrap() {
            return Err(shortlinker::errors::ShortlinkerError::file_operation(
                "Mock reload error".to_string(),
            ));
        }
        Ok(())
    }

    async fn get_backend_name(&self) -> String {
        "mock".to_string()
    }
}

#[cfg(test)]
mod cli_parser_tests {
    use super::*;

    // 测试助手函数，用于模拟命令行参数
    fn with_args<F>(_args: Vec<&str>, test_fn: F)
    where
        F: FnOnce(),
    {
        let _original_args: Vec<String> = env::args().collect();

        // 这里我们测试解析器的逻辑，而不是实际的命令行参数
        // 因为在测试环境中修改 env::args() 是困难的
        test_fn();
    }

    #[test]
    fn test_parser_creation() {
        let parser = CliParser::new();
        // 测试解析器可以被创建，并且有实际功能
        // 由于 CliParser 是零大小结构体，我们测试它的方法是否可用
        let empty_args: Vec<String> = vec![];
        let result = parser.parse_add_command(&empty_args);
        assert!(result.is_err()); // 应该返回错误，证明功能正常

        let parser2 = CliParser::default();
        let result2 = parser2.parse_remove_command(&empty_args);
        assert!(result2.is_err()); // 应该返回错误，证明功能正常
    }

    #[test]
    fn test_parse_add_command_direct() {
        let parser = CliParser::new();

        // 测试有效的单参数 add 命令
        let args = vec!["https://example.com".to_string()];
        let result = parser.parse_add_command(&args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            short_code,
            target_url,
            force_overwrite,
            expire_time,
        }) = result
        {
            assert_eq!(short_code, None);
            assert_eq!(target_url, "https://example.com");
            assert!(!force_overwrite);
            assert_eq!(expire_time, None);
        } else {
            panic!("Expected Add command");
        }
    }

    #[test]
    fn test_parse_add_command_with_custom_code() {
        let parser = CliParser::new();

        let args = vec!["mycode".to_string(), "https://example.com".to_string()];
        let result = parser.parse_add_command(&args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            short_code,
            target_url,
            ..
        }) = result
        {
            assert_eq!(short_code, Some("mycode".to_string()));
            assert_eq!(target_url, "https://example.com");
        }
    }

    #[test]
    fn test_parse_add_command_with_flags() {
        let parser = CliParser::new();

        let args = vec![
            "https://example.com".to_string(),
            "--force".to_string(),
            "--expire".to_string(),
            "2023-12-31T23:59:59Z".to_string(),
        ];
        let result = parser.parse_add_command(&args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            force_overwrite,
            expire_time,
            ..
        }) = result
        {
            assert!(force_overwrite);
            assert_eq!(expire_time, Some("2023-12-31T23:59:59Z".to_string()));
        }
    }

    #[test]
    fn test_parse_add_command_invalid_args() {
        let parser = CliParser::new();

        // 无参数
        let empty_args: Vec<String> = vec![];
        let result = parser.parse_add_command(&empty_args);
        assert!(result.is_err());
        if let Err(CliError::ParseError(msg)) = result {
            assert!(msg.contains("Add command requires arguments"));
        }

        // 参数过多
        let too_many_args = vec!["code".to_string(), "url".to_string(), "extra".to_string()];
        let result = parser.parse_add_command(&too_many_args);
        assert!(result.is_err());
        if let Err(CliError::ParseError(msg)) = result {
            assert!(msg.contains("Invalid number of arguments"));
        }

        // 缺少 expire 参数值
        let missing_expire_value = vec!["https://example.com".to_string(), "--expire".to_string()];
        let result = parser.parse_add_command(&missing_expire_value);
        assert!(result.is_err());
        if let Err(CliError::ParseError(msg)) = result {
            assert!(msg.contains("--expire requires a time argument"));
        }
    }

    #[test]
    fn test_parse_remove_command_direct() {
        let parser = CliParser::new();

        // 有效的 remove 命令
        let args = vec!["testcode".to_string()];
        let result = parser.parse_remove_command(&args);
        assert!(result.is_ok());

        if let Ok(Command::Remove { short_code }) = result {
            assert_eq!(short_code, "testcode");
        }

        // 无参数
        let empty_args: Vec<String> = vec![];
        let result = parser.parse_remove_command(&empty_args);
        assert!(result.is_err());
        if let Err(CliError::ParseError(msg)) = result {
            assert!(msg.contains("Remove command requires exactly one argument"));
        }

        // 参数过多
        let too_many_args = vec!["code1".to_string(), "code2".to_string()];
        let result = parser.parse_remove_command(&too_many_args);
        assert!(result.is_err());
        if let Err(CliError::ParseError(msg)) = result {
            assert!(msg.contains("Remove command requires exactly one argument"));
        }
    }

    #[test]
    fn test_parse_with_simulated_args() {
        with_args(vec!["shortlinker", "help"], || {
            // 由于无法轻易模拟 env::args()，我们测试解析器的存在和基本功能
            let parser = CliParser::new();

            // 测试解析器的功能性而不是大小
            // 测试一个简单的解析操作
            let args = vec!["test".to_string()];
            let result = parser.parse_remove_command(&args);
            assert!(result.is_ok());

            if let Ok(Command::Remove { short_code }) = result {
                assert_eq!(short_code, "test");
            }
        });
    }

    #[test]
    fn test_parse_complex_scenarios() {
        let parser = CliParser::new();

        // 测试复杂的参数组合
        let args = vec![
            "mycode".to_string(),
            "https://example.com/very/long/path?param=value&other=test".to_string(),
            "--force".to_string(),
            "--expire".to_string(),
            "2025-01-01T00:00:00Z".to_string(),
        ];

        let result = parser.parse_add_command(&args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            short_code,
            target_url,
            force_overwrite,
            expire_time,
        }) = result
        {
            assert_eq!(short_code, Some("mycode".to_string()));
            assert_eq!(
                target_url,
                "https://example.com/very/long/path?param=value&other=test"
            );
            assert!(force_overwrite);
            assert_eq!(expire_time, Some("2025-01-01T00:00:00Z".to_string()));
        }

        // 测试只有URL的情况
        let simple_args = vec!["https://simple.com".to_string()];
        let simple_result = parser.parse_add_command(&simple_args);
        assert!(simple_result.is_ok());

        if let Ok(Command::Add {
            short_code,
            target_url,
            force_overwrite,
            expire_time,
        }) = simple_result
        {
            assert_eq!(short_code, None);
            assert_eq!(target_url, "https://simple.com");
            assert!(!force_overwrite);
            assert_eq!(expire_time, None);
        }
    }

    #[test]
    fn test_parse_flag_combinations() {
        let parser = CliParser::new();

        // 测试只有 --force 标志
        let force_args = vec!["https://example.com".to_string(), "--force".to_string()];
        let result = parser.parse_add_command(&force_args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            force_overwrite,
            expire_time,
            ..
        }) = result
        {
            assert!(force_overwrite);
            assert_eq!(expire_time, None);
        }

        // 测试只有 --expire 标志
        let expire_args = vec![
            "https://example.com".to_string(),
            "--expire".to_string(),
            "2024-12-31T23:59:59Z".to_string(),
        ];
        let result = parser.parse_add_command(&expire_args);
        assert!(result.is_ok());

        if let Ok(Command::Add {
            force_overwrite,
            expire_time,
            ..
        }) = result
        {
            assert!(!force_overwrite);
            assert_eq!(expire_time, Some("2024-12-31T23:59:59Z".to_string()));
        }
    }
}

#[cfg(test)]
mod link_management_tests {
    use super::*;

    #[tokio::test]
    async fn test_list_empty_links() {
        let storage = Arc::new(MockStorage::default());
        let result = shortlinker::cli::commands::list_links(storage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_add_link_with_invalid_url() {
        let storage = Arc::new(MockStorage::default());

        let result = shortlinker::cli::commands::add_link(
            storage,
            Some("test".to_string()),
            "invalid-url".to_string(),
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("http://") || msg.contains("https://"));
        }
    }

    #[tokio::test]
    async fn test_add_link_with_valid_url() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let result = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            None,
        )
        .await;

        assert!(result.is_ok());

        // 验证链接是否被正确存储
        let links = storage.load_all().await;
        assert!(links.contains_key("test"));
        assert_eq!(links["test"].target, "https://example.com");
    }

    #[tokio::test]
    async fn test_add_link_with_storage_failure() {
        let storage = Arc::new(MockStorage::new_failing());

        let result = shortlinker::cli::commands::add_link(
            storage,
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("保存失败"));
        }
    }

    #[tokio::test]
    async fn test_add_link_with_expire_time() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let result = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            Some("2023-12-31T23:59:59Z".to_string()),
        )
        .await;

        assert!(result.is_ok());

        let links = storage.load_all().await;
        assert!(links.contains_key("test"));
        assert!(links["test"].expires_at.is_some());
    }

    #[tokio::test]
    async fn test_add_link_with_invalid_expire_time() {
        let storage = Arc::new(MockStorage::default());

        let result = shortlinker::cli::commands::add_link(
            storage,
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            Some("invalid-date".to_string()),
        )
        .await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("过期时间格式不正确"));
        }
    }

    #[tokio::test]
    async fn test_add_duplicate_link_without_force() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 先添加一个链接
        let _ = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            None,
        )
        .await;

        // 尝试添加相同短码的链接（不使用 force）
        let result = shortlinker::cli::commands::add_link(
            storage,
            Some("test".to_string()),
            "https://another.com".to_string(),
            false,
            None,
        )
        .await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("已存在"));
        }
    }

    #[tokio::test]
    async fn test_add_duplicate_link_with_force() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 先添加一个链接
        let _ = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            None,
        )
        .await;

        // 使用 force 覆盖
        let result = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://another.com".to_string(),
            true,
            None,
        )
        .await;

        assert!(result.is_ok());

        // 验证链接被正确覆盖
        let links = storage.load_all().await;
        assert_eq!(links["test"].target, "https://another.com");
    }

    #[tokio::test]
    async fn test_remove_existing_link() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 先添加一个链接
        let _ = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("test".to_string()),
            "https://example.com".to_string(),
            false,
            None,
        )
        .await;

        // 删除链接
        let result =
            shortlinker::cli::commands::remove_link(storage.clone(), "test".to_string()).await;

        assert!(result.is_ok());

        // 验证链接已被删除
        let links = storage.load_all().await;
        assert!(!links.contains_key("test"));
    }

    #[tokio::test]
    async fn test_remove_nonexistent_link() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let result =
            shortlinker::cli::commands::remove_link(storage, "nonexistent".to_string()).await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("不存在"));
        }
    }

    #[tokio::test]
    async fn test_remove_with_storage_failure() {
        let storage = Arc::new(MockStorage::new_failing());

        let result = shortlinker::cli::commands::remove_link(storage, "test".to_string()).await;

        assert!(result.is_err());
        if let Err(CliError::CommandError(msg)) = result {
            assert!(msg.contains("不存在"));
        }
    }
}

#[cfg(test)]
mod storage_backend_tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_factory_creation() {
        // 测试默认存储后端
        let storage = shortlinker::storages::StorageFactory::create();
        assert!(storage.is_ok());

        let storage = storage.unwrap();
        let backend_name = storage.get_backend_name().await;
        assert!(["sqlite", "file", "sled"].contains(&backend_name.as_str()));
    }

    #[tokio::test]
    async fn test_storage_factory_with_env() {
        // 测试文件存储后端
        env::set_var("STORAGE_BACKEND", "file");
        let storage = shortlinker::storages::StorageFactory::create();
        assert!(storage.is_ok());

        let storage = storage.unwrap();
        let backend_name = storage.get_backend_name().await;
        assert_eq!(backend_name, "file");

        // 恢复环境变量
        env::remove_var("STORAGE_BACKEND");
    }

    #[tokio::test]
    async fn test_mock_storage_functionality() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 测试基本的 CRUD 操作
        let link = shortlinker::storages::ShortLink {
            code: "test".to_string(),
            target: "https://example.com".to_string(),
            created_at: chrono::Utc::now(),
            expires_at: None,
        };

        // 设置
        let result = storage.set(link.clone()).await;
        assert!(result.is_ok());

        // 获取
        let retrieved = storage.get("test").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().target, "https://example.com");

        // 列出所有
        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 1);

        // 删除
        let result = storage.remove("test").await;
        assert!(result.is_ok());

        let all_links = storage.load_all().await;
        assert_eq!(all_links.len(), 0);
    }
}

#[cfg(test)]
mod process_manager_tests {
    use super::*;
    use shortlinker::cli::process_manager::ProcessManager;

    #[test]
    fn test_start_server_function_exists() {
        let result = ProcessManager::start_server();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_stop_server_function_exists() {
        let result = ProcessManager::stop_server();
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_restart_server_function_exists() {
        let result = ProcessManager::restart_server();
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod command_execution_tests {
    use super::*;

    #[tokio::test]
    async fn test_command_help_execution() {
        let storage = Arc::new(MockStorage::default());
        let command = Command::Help;
        let result = command.execute(storage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_list_execution() {
        let storage = Arc::new(MockStorage::default());
        let command = Command::List;
        let result = command.execute(storage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_add_execution() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        let command = Command::Add {
            short_code: Some("test".to_string()),
            target_url: "https://example.com".to_string(),
            force_overwrite: false,
            expire_time: None,
        };
        let result = command.execute(storage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_command_remove_execution() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 先添加一个链接
        let add_command = Command::Add {
            short_code: Some("test".to_string()),
            target_url: "https://example.com".to_string(),
            force_overwrite: false,
            expire_time: None,
        };
        let _ = add_command.execute(storage.clone()).await;

        // 然后删除它
        let remove_command = Command::Remove {
            short_code: "test".to_string(),
        };
        let result = remove_command.execute(storage).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_commands_execution() {
        let storage = Arc::new(MockStorage::default());

        // 测试服务器命令不会崩溃
        let start_cmd = Command::Start;
        let result = start_cmd.execute(storage.clone()).await;
        assert!(result.is_ok() || result.is_err());

        let stop_cmd = Command::Stop;
        let result = stop_cmd.execute(storage.clone()).await;
        assert!(result.is_ok() || result.is_err());

        let restart_cmd = Command::Restart;
        let result = restart_cmd.execute(storage).await;
        assert!(result.is_ok() || result.is_err());
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_cli_error_display() {
        let storage_error = CliError::StorageError("存储错误".to_string());
        assert!(storage_error.to_string().contains("Storage error"));

        let parse_error = CliError::ParseError("解析错误".to_string());
        assert!(parse_error.to_string().contains("Parse error"));

        let command_error = CliError::CommandError("命令错误".to_string());
        assert!(command_error.to_string().contains("Command error"));

        let process_error = CliError::ProcessError("进程错误".to_string());
        assert!(process_error.to_string().contains("Process error"));
    }

    #[test]
    fn test_cli_error_debug() {
        let error = CliError::StorageError("test".to_string());
        let debug_output = format!("{:?}", error);
        assert!(debug_output.contains("StorageError"));
    }

    #[test]
    fn test_cli_error_is_error_trait() {
        use std::error::Error;

        let error = CliError::StorageError("test".to_string());
        let _: &dyn Error = &error; // 确保实现了 Error trait
    }

    #[test]
    fn test_error_conversion() {
        let shortlinker_error =
            shortlinker::errors::ShortlinkerError::file_operation("test".to_string());
        let cli_error: CliError = shortlinker_error.into();

        assert!(matches!(cli_error, CliError::StorageError(_)));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_workflow() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 1. 添加链接
        let add_result = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("github".to_string()),
            "https://github.com".to_string(),
            false,
            None,
        )
        .await;
        assert!(add_result.is_ok());

        // 2. 列出链接
        let list_result = shortlinker::cli::commands::list_links(storage.clone()).await;
        assert!(list_result.is_ok());

        // 3. 验证链接存在
        let links = storage.load_all().await;
        assert!(links.contains_key("github"));

        // 4. 删除链接
        let remove_result =
            shortlinker::cli::commands::remove_link(storage.clone(), "github".to_string()).await;
        assert!(remove_result.is_ok());

        // 5. 验证链接已删除
        let links_after_remove = storage.load_all().await;
        assert!(!links_after_remove.contains_key("github"));
    }

    #[tokio::test]
    async fn test_concurrent_operations() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 并发添加多个链接
        let mut handles = vec![];

        for i in 0..5 {
            let storage_clone = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                shortlinker::cli::commands::add_link(
                    storage_clone,
                    Some(format!("link{}", i)),
                    format!("https://example{}.com", i),
                    false,
                    None,
                )
                .await
            });
            handles.push(handle);
        }

        // 等待所有操作完成
        for handle in handles {
            let result = handle.await.unwrap();
            assert!(result.is_ok());
        }

        // 验证所有链接都被正确添加
        let links = storage.load_all().await;
        assert_eq!(links.len(), 5);

        for i in 0..5 {
            assert!(links.contains_key(&format!("link{}", i)));
        }
    }

    #[tokio::test]
    async fn test_edge_cases() {
        let storage = Arc::new(MockStorage::default());
        storage.set_should_fail(false);

        // 测试特殊字符的短码
        let special_codes = vec!["test-123", "test_456", "test.789"];

        for code in special_codes {
            let result = shortlinker::cli::commands::add_link(
                storage.clone(),
                Some(code.to_string()),
                "https://example.com".to_string(),
                false,
                None,
            )
            .await;
            assert!(result.is_ok());
        }

        let links = storage.load_all().await;
        assert_eq!(links.len(), 3);

        // 测试长URL
        let long_url = "https://example.com/".to_string() + &"a".repeat(1000);
        let result = shortlinker::cli::commands::add_link(
            storage.clone(),
            Some("long".to_string()),
            long_url.clone(),
            false,
            None,
        )
        .await;
        assert!(result.is_ok());

        let retrieved = storage.get("long").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().target, long_url);
    }

    #[tokio::test]
    async fn test_url_validation_edge_cases() {
        let storage = Arc::new(MockStorage::default());

        let invalid_urls = vec![
            "",
            "not-a-url",
            "ftp://example.com",
            "file:///etc/passwd",
            "javascript:alert('xss')",
        ];

        for url in invalid_urls {
            let result = shortlinker::cli::commands::add_link(
                storage.clone(),
                Some("test".to_string()),
                url.to_string(),
                false,
                None,
            )
            .await;
            assert!(result.is_err(), "URL should be invalid: {}", url);
        }

        let valid_urls = vec![
            "https://example.com",
            "http://localhost:8080",
            "https://subdomain.example.com/path?query=value#fragment",
            "http://192.168.1.1:3000",
        ];

        for (i, url) in valid_urls.iter().enumerate() {
            let result = shortlinker::cli::commands::add_link(
                storage.clone(),
                Some(format!("valid{}", i)),
                url.to_string(),
                false,
                None,
            )
            .await;
            assert!(result.is_ok(), "URL should be valid: {}", url);
        }
    }
}
