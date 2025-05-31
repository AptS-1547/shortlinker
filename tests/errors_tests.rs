use shortlinker::errors::{Result, ShortlinkerError};
use std::error::Error;

#[cfg(test)]
mod error_creation_tests {
    use super::*;

    #[test]
    fn test_database_connection_error() {
        let error = ShortlinkerError::database_connection("连接失败");

        assert!(matches!(error, ShortlinkerError::DatabaseConnection(_)));
        assert!(error.to_string().contains("数据库连接错误"));
        assert!(error.to_string().contains("连接失败"));
    }

    #[test]
    fn test_database_operation_error() {
        let error = ShortlinkerError::database_operation("操作失败");

        assert!(matches!(error, ShortlinkerError::DatabaseOperation(_)));
        assert!(error.to_string().contains("数据库操作错误"));
        assert!(error.to_string().contains("操作失败"));
    }

    #[test]
    fn test_file_operation_error() {
        let error = ShortlinkerError::file_operation("文件读取失败");

        assert!(matches!(error, ShortlinkerError::FileOperation(_)));
        assert!(error.to_string().contains("文件操作错误"));
        assert!(error.to_string().contains("文件读取失败"));
    }

    #[test]
    fn test_validation_error() {
        let error = ShortlinkerError::validation("验证失败");

        assert!(matches!(error, ShortlinkerError::Validation(_)));
        assert!(error.to_string().contains("验证错误"));
        assert!(error.to_string().contains("验证失败"));
    }

    #[test]
    fn test_not_found_error() {
        let error = ShortlinkerError::not_found("资源不存在");

        assert!(matches!(error, ShortlinkerError::NotFound(_)));
        assert!(error.to_string().contains("资源未找到"));
        assert!(error.to_string().contains("资源不存在"));
    }

    #[test]
    fn test_serialization_error() {
        let error = ShortlinkerError::serialization("序列化失败");

        assert!(matches!(error, ShortlinkerError::Serialization(_)));
        assert!(error.to_string().contains("序列化错误"));
        assert!(error.to_string().contains("序列化失败"));
    }

    #[test]
    fn test_signal_operation_error() {
        let error = ShortlinkerError::signal_operation("信号处理失败");

        assert!(matches!(error, ShortlinkerError::SignalOperation(_)));
        assert!(error.to_string().contains("信号操作错误"));
        assert!(error.to_string().contains("信号处理失败"));
    }
}

#[cfg(test)]
mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "文件未找到");
        let shortlinker_error: ShortlinkerError = io_error.into();

        assert!(matches!(
            shortlinker_error,
            ShortlinkerError::FileOperation(_)
        ));
        assert!(shortlinker_error.to_string().contains("文件操作错误"));
        assert!(shortlinker_error.to_string().contains("文件未找到"));
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // 创建一个无效的 JSON 来触发错误
        let invalid_json = "{invalid json";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let shortlinker_error: ShortlinkerError = json_error.into();

        assert!(matches!(
            shortlinker_error,
            ShortlinkerError::Serialization(_)
        ));
        assert!(shortlinker_error.to_string().contains("序列化错误"));
    }

    #[test]
    fn test_chrono_parse_error_conversion() {
        let invalid_date = "不是日期";
        let parse_error = chrono::DateTime::parse_from_rfc3339(invalid_date).unwrap_err();
        let shortlinker_error: ShortlinkerError = parse_error.into();

        assert!(matches!(shortlinker_error, ShortlinkerError::Validation(_)));
        assert!(shortlinker_error.to_string().contains("验证错误"));
        assert!(shortlinker_error.to_string().contains("时间解析错误"));
    }
}

#[cfg(test)]
mod error_trait_tests {
    use super::*;

    #[test]
    fn test_error_trait_implementation() {
        let error = ShortlinkerError::validation("测试错误");

        // 测试 Error trait 的实现
        let error_trait: &dyn Error = &error;
        assert!(!error_trait.to_string().is_empty());

        // 测试 source 方法（应该返回 None，因为我们的错误是顶级错误）
        assert!(error_trait.source().is_none());
    }

    #[test]
    fn test_debug_implementation() {
        let error = ShortlinkerError::database_connection("调试测试");
        let debug_string = format!("{:?}", error);

        assert!(debug_string.contains("DatabaseConnection"));
        assert!(debug_string.contains("调试测试"));
    }

    #[test]
    fn test_clone_implementation() {
        let original = ShortlinkerError::file_operation("克隆测试");
        let cloned = original.clone();

        // 验证克隆后的错误与原始错误相同
        assert_eq!(original.to_string(), cloned.to_string());
        assert!(matches!(cloned, ShortlinkerError::FileOperation(_)));
    }

    #[test]
    fn test_send_sync_traits() {
        // 验证错误类型实现了 Send 和 Sync
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}

        assert_send::<ShortlinkerError>();
        assert_sync::<ShortlinkerError>();
    }
}

#[cfg(test)]
mod result_type_tests {
    use super::*;

    #[test]
    fn test_result_ok() {
        let result: Result<String> = Ok("成功".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "成功");
    }

    #[test]
    fn test_result_err() {
        let result: Result<String> = Err(ShortlinkerError::validation("失败"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ShortlinkerError::Validation(_)));
    }

    #[test]
    fn test_result_map() {
        let result: Result<i32> = Ok(42);
        let mapped = result.map(|x| x * 2);

        assert!(mapped.is_ok());
        assert_eq!(mapped.unwrap(), 84);
    }

    #[test]
    fn test_result_and_then() {
        let result: Result<i32> = Ok(10);
        let chained = result.and_then(|x| {
            if x > 5 {
                Ok(x * 2)
            } else {
                Err(ShortlinkerError::validation("数值太小"))
            }
        });

        assert!(chained.is_ok());
        assert_eq!(chained.unwrap(), 20);
    }

    #[test]
    fn test_result_or_else() {
        let result: Result<String> = Err(ShortlinkerError::not_found("未找到"));
        let recovered: Result<String> = result.or_else(|_| Ok("默认值".to_string()));

        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), "默认值");
    }
}

#[cfg(test)]
mod error_message_tests {
    use super::*;

    #[test]
    fn test_error_message_format() {
        let test_cases = vec![
            (
                ShortlinkerError::database_connection("连接超时"),
                "数据库连接错误: 连接超时",
            ),
            (
                ShortlinkerError::database_operation("查询失败"),
                "数据库操作错误: 查询失败",
            ),
            (
                ShortlinkerError::file_operation("权限不足"),
                "文件操作错误: 权限不足",
            ),
            (
                ShortlinkerError::validation("格式错误"),
                "验证错误: 格式错误",
            ),
            (
                ShortlinkerError::not_found("用户不存在"),
                "资源未找到: 用户不存在",
            ),
            (
                ShortlinkerError::serialization("JSON错误"),
                "序列化错误: JSON错误",
            ),
            (
                ShortlinkerError::signal_operation("信号中断"),
                "信号操作错误: 信号中断",
            ),
        ];

        for (error, expected_message) in test_cases {
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn test_empty_error_message() {
        let error = ShortlinkerError::validation("");
        assert!(error.to_string().contains("验证错误"));
    }

    #[test]
    fn test_unicode_error_message() {
        let unicode_message = "错误信息包含中文和emoji 🚫";
        let error = ShortlinkerError::validation(unicode_message);

        assert!(error.to_string().contains(unicode_message));
        assert!(error.to_string().contains("验证错误"));
    }

    #[test]
    fn test_long_error_message() {
        let long_message = "这是一个非常长的错误信息，".repeat(100);
        let error = ShortlinkerError::database_operation(&long_message);

        assert!(error.to_string().contains(&long_message));
        assert!(error.to_string().len() > long_message.len());
    }
}

#[cfg(test)]
mod error_chain_tests {
    use super::*;

    #[test]
    fn test_error_propagation() {
        // 模拟错误传播链
        fn operation_that_fails() -> Result<String> {
            Err(ShortlinkerError::database_operation("底层错误"))
        }

        fn higher_level_operation() -> Result<String> {
            operation_that_fails()
                .map_err(|e| ShortlinkerError::validation(format!("高层错误: {}", e)))
        }

        let result = higher_level_operation();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ShortlinkerError::Validation(_)));
        assert!(error.to_string().contains("高层错误"));
        assert!(error.to_string().contains("数据库操作错误"));
    }

    #[test]
    fn test_multiple_error_conversions() {
        // 测试多重错误转换
        let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "权限被拒绝");
        let shortlinker_error: ShortlinkerError = io_error.into();

        // 再次包装错误
        let wrapped_error =
            ShortlinkerError::validation(format!("包装错误: {}", shortlinker_error));

        assert!(matches!(wrapped_error, ShortlinkerError::Validation(_)));
        assert!(wrapped_error.to_string().contains("包装错误"));
        assert!(wrapped_error.to_string().contains("文件操作错误"));
        assert!(wrapped_error.to_string().contains("权限被拒绝"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_error_in_real_scenarios() {
        // 模拟真实场景中的错误处理

        // 场景1：文件操作失败
        fn simulate_file_operation() -> Result<String> {
            // 模拟文件不存在的情况
            let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "配置文件不存在");
            Err(io_error.into())
        }

        let result = simulate_file_operation();
        assert!(result.is_err());

        match result.unwrap_err() {
            ShortlinkerError::FileOperation(msg) => {
                assert!(msg.contains("配置文件不存在"));
            }
            _ => panic!("期望文件操作错误"),
        }

        // 场景2：验证失败
        fn simulate_validation() -> Result<()> {
            let url = "invalid-url";
            if !url.starts_with("http") {
                return Err(ShortlinkerError::validation("URL格式无效"));
            }
            Ok(())
        }

        let result = simulate_validation();
        assert!(result.is_err());

        // 场景3：资源未找到
        fn simulate_resource_lookup(id: &str) -> Result<String> {
            if id == "nonexistent" {
                return Err(ShortlinkerError::not_found(format!(
                    "ID {} 对应的资源不存在",
                    id
                )));
            }
            Ok("found".to_string())
        }

        let result = simulate_resource_lookup("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nonexistent"));
    }

    #[test]
    fn test_error_recovery_patterns() {
        // 测试错误恢复模式

        fn fallback_operation() -> Result<String> {
            // 首先尝试主要操作
            primary_operation()
                .or_else(|_| {
                    // 主要操作失败，尝试备用操作
                    secondary_operation()
                })
                .or_else(|_| {
                    // 备用操作也失败，使用默认值
                    Ok("默认值".to_string())
                })
        }

        fn primary_operation() -> Result<String> {
            Err(ShortlinkerError::database_connection("主数据库不可用"))
        }

        fn secondary_operation() -> Result<String> {
            Err(ShortlinkerError::database_connection("备用数据库不可用"))
        }

        let result = fallback_operation();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "默认值");
    }

    #[test]
    fn test_concurrent_error_handling() {
        // 测试并发环境下的错误处理
        use std::sync::{Arc, Mutex};
        use std::thread;

        let errors = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let errors_clone = Arc::clone(&errors);
            let handle = thread::spawn(move || {
                let error = ShortlinkerError::validation(format!("线程{}的错误", i));
                let mut errors_guard = errors_clone.lock().unwrap();
                errors_guard.push(error);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_errors = errors.lock().unwrap();
        assert_eq!(final_errors.len(), 10);

        for (i, error) in final_errors.iter().enumerate() {
            assert!(error.to_string().contains(&format!("线程{}的错误", i)));
        }
    }
}
