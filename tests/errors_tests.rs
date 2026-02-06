use shortlinker::errors::{Result, ShortlinkerError};
use std::error::Error;

#[cfg(test)]
mod error_creation_tests {
    use super::*;

    #[test]
    fn test_database_connection_error() {
        let error = ShortlinkerError::database_connection("connection failed");

        assert!(matches!(error, ShortlinkerError::DatabaseConnection(_)));
        assert!(error.to_string().contains("Database Connection Error"));
        assert!(error.to_string().contains("connection failed"));
    }

    #[test]
    fn test_database_operation_error() {
        let error = ShortlinkerError::database_operation("operation failed");

        assert!(matches!(error, ShortlinkerError::DatabaseOperation(_)));
        assert!(error.to_string().contains("Database Operation Error"));
        assert!(error.to_string().contains("operation failed"));
    }

    #[test]
    fn test_file_operation_error() {
        let error = ShortlinkerError::file_operation("file read failed");

        assert!(matches!(error, ShortlinkerError::FileOperation(_)));
        assert!(error.to_string().contains("File Operation Error"));
        assert!(error.to_string().contains("file read failed"));
    }

    #[test]
    fn test_validation_error() {
        let error = ShortlinkerError::validation("validation failed");

        assert!(matches!(error, ShortlinkerError::Validation(_)));
        assert!(error.to_string().contains("Validation Error"));
        assert!(error.to_string().contains("validation failed"));
    }

    #[test]
    fn test_not_found_error() {
        let error = ShortlinkerError::not_found("resource does not exist");

        assert!(matches!(error, ShortlinkerError::NotFound(_)));
        assert!(error.to_string().contains("Resource Not Found"));
        assert!(error.to_string().contains("resource does not exist"));
    }

    #[test]
    fn test_serialization_error() {
        let error = ShortlinkerError::serialization("serialization failed");

        assert!(matches!(error, ShortlinkerError::Serialization(_)));
        assert!(error.to_string().contains("Serialization Error"));
        assert!(error.to_string().contains("serialization failed"));
    }
}

#[cfg(test)]
mod error_conversion_tests {
    use super::*;

    #[test]
    fn test_io_error_conversion() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let shortlinker_error: ShortlinkerError = io_error.into();

        assert!(matches!(
            shortlinker_error,
            ShortlinkerError::FileOperation(_)
        ));
        assert!(
            shortlinker_error
                .to_string()
                .contains("File Operation Error")
        );
        assert!(shortlinker_error.to_string().contains("file not found"));
    }

    #[test]
    fn test_serde_json_error_conversion() {
        // Create invalid JSON to trigger an error
        let invalid_json = "{invalid json";
        let json_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let shortlinker_error: ShortlinkerError = json_error.into();

        assert!(matches!(
            shortlinker_error,
            ShortlinkerError::Serialization(_)
        ));
        assert!(
            shortlinker_error
                .to_string()
                .contains("Serialization Error")
        );
    }
}

#[cfg(test)]
mod error_trait_tests {
    use super::*;

    #[test]
    fn test_error_trait_implementation() {
        let error = ShortlinkerError::validation("test error");

        // Test Error trait implementation
        let error_trait: &dyn Error = &error;
        assert!(!error_trait.to_string().is_empty());

        // Test source method (should return None as our error is top-level)
        assert!(error_trait.source().is_none());
    }

    #[test]
    fn test_debug_implementation() {
        let error = ShortlinkerError::database_connection("debug test");
        let debug_string = format!("{:?}", error);

        assert!(debug_string.contains("DatabaseConnection"));
        assert!(debug_string.contains("debug test"));
    }

    #[test]
    fn test_clone_implementation() {
        let original = ShortlinkerError::file_operation("clone test");
        let cloned = original.clone();

        // Verify cloned error matches original
        assert_eq!(original.to_string(), cloned.to_string());
        assert!(matches!(cloned, ShortlinkerError::FileOperation(_)));
    }

    #[test]
    fn test_send_sync_traits() {
        // Verify error type implements Send and Sync
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
        let result: Result<String> = Ok("success".to_string());
        assert!(result.is_ok());
        assert_eq!(result.as_ref().unwrap(), "success");
    }

    #[test]
    fn test_result_err() {
        let result: Result<String> = Err(ShortlinkerError::validation("failed"));
        assert!(result.is_err());

        let error = result.as_ref().unwrap_err();
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
                Err(ShortlinkerError::validation("value too small"))
            }
        });

        assert!(chained.is_ok());
        assert_eq!(chained.unwrap(), 20);
    }

    #[test]
    fn test_result_or_else() {
        let result: Result<String> = Err(ShortlinkerError::not_found("not found"));
        let recovered: Result<String> = result.or_else(|_| Ok("default".to_string()));

        assert!(recovered.is_ok());
        assert_eq!(recovered.unwrap(), "default");
    }
}

#[cfg(test)]
mod error_message_tests {
    use super::*;

    #[test]
    fn test_error_message_format() {
        let test_cases = vec![
            (
                ShortlinkerError::database_connection("connection timeout"),
                "Database Connection Error: connection timeout",
            ),
            (
                ShortlinkerError::database_operation("query failed"),
                "Database Operation Error: query failed",
            ),
            (
                ShortlinkerError::file_operation("permission denied"),
                "File Operation Error: permission denied",
            ),
            (
                ShortlinkerError::validation("format error"),
                "Validation Error: format error",
            ),
            (
                ShortlinkerError::not_found("user not found"),
                "Resource Not Found: user not found",
            ),
            (
                ShortlinkerError::serialization("JSON error"),
                "Serialization Error: JSON error",
            ),
            (
                ShortlinkerError::notify_server("notify server failed"),
                "Notify Server Error: notify server failed",
            ),
        ];

        for (error, expected_message) in test_cases {
            assert_eq!(error.to_string(), expected_message);
        }
    }

    #[test]
    fn test_empty_error_message() {
        let error = ShortlinkerError::validation("");
        assert!(error.to_string().contains("Validation Error"));
    }

    #[test]
    fn test_unicode_error_message() {
        let unicode_message = "error with unicode and emoji 🚫";
        let error = ShortlinkerError::validation(unicode_message);

        assert!(error.to_string().contains(unicode_message));
        assert!(error.to_string().contains("Validation Error"));
    }

    #[test]
    fn test_long_error_message() {
        let long_message = "this is a very long error message, ".repeat(100);
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
        // Simulate error propagation chain
        fn operation_that_fails() -> Result<String> {
            Err(ShortlinkerError::database_operation("underlying error"))
        }

        fn higher_level_operation() -> Result<String> {
            operation_that_fails()
                .map_err(|e| ShortlinkerError::validation(format!("high level error: {}", e)))
        }

        let result = higher_level_operation();
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(matches!(error, ShortlinkerError::Validation(_)));
        assert!(error.to_string().contains("high level error"));
        assert!(error.to_string().contains("Database Operation Error"));
    }

    #[test]
    fn test_multiple_error_conversions() {
        // Test multiple error conversions
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let shortlinker_error: ShortlinkerError = io_error.into();

        // Wrap the error again
        let wrapped_error =
            ShortlinkerError::validation(format!("wrapped error: {}", shortlinker_error));

        assert!(matches!(wrapped_error, ShortlinkerError::Validation(_)));
        assert!(wrapped_error.to_string().contains("wrapped error"));
        assert!(wrapped_error.to_string().contains("File Operation Error"));
        assert!(wrapped_error.to_string().contains("permission denied"));
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_error_in_real_scenarios() {
        // Simulate real-world error handling scenarios

        // Scenario 1: File operation failure
        fn simulate_file_operation() -> Result<String> {
            // Simulate file not found
            let io_error =
                std::io::Error::new(std::io::ErrorKind::NotFound, "config file not found");
            Err(io_error.into())
        }

        let result = simulate_file_operation();
        assert!(result.is_err());

        match result.unwrap_err() {
            ShortlinkerError::FileOperation(msg) => {
                assert!(msg.contains("config file not found"));
            }
            _ => panic!("expected file operation error"),
        }

        // Scenario 2: Validation failure
        fn simulate_validation() -> Result<()> {
            let url = "invalid-url";
            if !url.starts_with("http") {
                return Err(ShortlinkerError::validation("invalid URL format"));
            }
            Ok(())
        }

        let result = simulate_validation();
        assert!(result.is_err());

        // Scenario 3: Resource not found
        fn simulate_resource_lookup(id: &str) -> Result<String> {
            if id == "nonexistent" {
                return Err(ShortlinkerError::not_found(format!(
                    "resource with ID {} not found",
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
        // Test error recovery patterns

        fn fallback_operation() -> Result<String> {
            // First try primary operation
            primary_operation()
                .or_else(|_| {
                    // Primary failed, try secondary
                    secondary_operation()
                })
                .or_else(|_| {
                    // Secondary also failed, use default
                    Ok("default".to_string())
                })
        }

        fn primary_operation() -> Result<String> {
            Err(ShortlinkerError::database_connection(
                "primary database unavailable",
            ))
        }

        fn secondary_operation() -> Result<String> {
            Err(ShortlinkerError::database_connection(
                "secondary database unavailable",
            ))
        }

        let result = fallback_operation();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "default");
    }

    #[test]
    fn test_concurrent_error_handling() {
        // Test concurrent error handling
        use std::sync::{Arc, Mutex};
        use std::thread;

        let errors = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for i in 0..10 {
            let errors_clone = Arc::clone(&errors);
            let handle = thread::spawn(move || {
                let error = ShortlinkerError::validation(format!("thread {} error", i));
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

        // Verify all errors are validation errors
        for error in final_errors.iter() {
            assert!(matches!(error, ShortlinkerError::Validation(_)));
            assert!(error.to_string().contains("Validation Error"));
            assert!(error.to_string().contains("thread"));
            assert!(error.to_string().contains("error"));
        }

        // Verify all thread IDs exist (0-9)
        let mut found_thread_ids = std::collections::HashSet::new();
        for error in final_errors.iter() {
            let error_msg = error.to_string();
            for i in 0..10 {
                if error_msg.contains(&format!("thread {}", i)) {
                    found_thread_ids.insert(i);
                    break;
                }
            }
        }
        assert_eq!(
            found_thread_ids.len(),
            10,
            "should contain all thread IDs 0-9"
        );
    }
}
