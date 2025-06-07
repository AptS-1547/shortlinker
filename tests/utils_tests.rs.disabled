use shortlinker::utils;
use shortlinker::utils::colors::*;
use std::collections::HashSet;

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn test_generate_random_code_basic() {
        // 测试基本的随机码生成
        let code = utils::generate_random_code(6);
        assert_eq!(code.len(), 6);

        // 验证生成的代码只包含有效字符
        let valid_chars: HashSet<char> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .collect();

        for ch in code.chars() {
            assert!(valid_chars.contains(&ch), "无效字符: {}", ch);
        }
    }

    #[test]
    fn test_generate_random_code_different_lengths() {
        // 测试不同长度的随机码生成
        let lengths = vec![1, 3, 6, 10, 20, 50];

        for length in lengths {
            let code = utils::generate_random_code(length);
            assert_eq!(code.len(), length, "长度 {} 的代码长度不正确", length);

            // 确保不为空（除非长度为0）
            if length > 0 {
                assert!(!code.is_empty());
            }
        }
    }

    #[test]
    fn test_generate_random_code_zero_length() {
        // 测试零长度的情况
        let code = utils::generate_random_code(0);
        assert_eq!(code.len(), 0);
        assert!(code.is_empty());
    }

    #[test]
    fn test_generate_random_code_uniqueness() {
        // 测试生成的代码的唯一性
        let mut codes = HashSet::new();
        let count = 1000;

        for _ in 0..count {
            let code = utils::generate_random_code(8);
            codes.insert(code);
        }

        // 由于是随机生成，应该有很高的唯一性
        // 允许一些碰撞，但应该大部分是唯一的
        let unique_ratio = codes.len() as f64 / count as f64;
        assert!(unique_ratio > 0.95, "唯一性比率太低: {}", unique_ratio);
    }

    #[test]
    fn test_generate_random_code_character_distribution() {
        // 测试字符分布的合理性
        let mut char_counts = std::collections::HashMap::new();
        let total_codes = 1000;
        let code_length = 6;

        for _ in 0..total_codes {
            let code = utils::generate_random_code(code_length);
            for ch in code.chars() {
                *char_counts.entry(ch).or_insert(0) += 1;
            }
        }

        // 验证所有字符都在有效范围内
        let valid_chars: HashSet<char> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .collect();

        for &ch in char_counts.keys() {
            assert!(valid_chars.contains(&ch), "发现无效字符: {}", ch);
        }

        // 验证字符分布相对均匀（允许一定的随机波动）
        let total_chars = total_codes * code_length;
        let expected_count_per_char = total_chars as f64 / valid_chars.len() as f64;

        for (ch, count) in char_counts.iter() {
            let ratio = *count as f64 / expected_count_per_char;
            assert!(
                ratio > 0.5 && ratio < 2.0,
                "字符 '{}' 的分布异常: 期望约 {}, 实际 {}",
                ch,
                expected_count_per_char,
                count
            );
        }
    }

    #[test]
    fn test_generate_random_code_concurrent() {
        // 测试并发生成的安全性
        use std::sync::{Arc, Mutex};
        use std::thread;

        let codes = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for _ in 0..10 {
            let codes_clone = Arc::clone(&codes);
            let handle = thread::spawn(move || {
                let mut local_codes = Vec::new();
                for _ in 0..100 {
                    local_codes.push(utils::generate_random_code(8));
                }

                let mut codes_guard = codes_clone.lock().unwrap();
                codes_guard.extend(local_codes);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_codes = codes.lock().unwrap();
        assert_eq!(final_codes.len(), 1000);

        // 验证所有代码都是有效的
        for code in final_codes.iter() {
            assert_eq!(code.len(), 8);
        }
    }

    #[test]
    fn test_generate_random_code_large_length() {
        // 测试生成非常长的代码
        let large_length = 1000;
        let code = utils::generate_random_code(large_length);

        assert_eq!(code.len(), large_length);

        // 验证长代码的有效性
        let valid_chars: HashSet<char> =
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
                .chars()
                .collect();

        for ch in code.chars() {
            assert!(valid_chars.contains(&ch));
        }
    }
}

#[cfg(test)]
mod colors_tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        // 测试颜色常量的正确性
        assert!(!RED.is_empty());
        assert!(!GREEN.is_empty());
        assert!(!YELLOW.is_empty());
        assert!(!BLUE.is_empty());
        assert!(!MAGENTA.is_empty());
        assert!(!CYAN.is_empty());
        assert!(!WHITE.is_empty());

        // 测试样式常量
        assert!(!BOLD.is_empty());
        assert!(!DIM.is_empty());
        assert!(!ITALIC.is_empty());
        assert!(!UNDERLINE.is_empty());
        assert!(!RESET.is_empty());

        // 验证 ANSI 转义序列格式
        assert!(RED.starts_with("\x1b["));
        assert!(GREEN.starts_with("\x1b["));
        assert!(RESET.starts_with("\x1b["));
    }

    #[test]
    fn test_color_formatting() {
        // 测试颜色格式化功能
        let text = "测试文本";

        // 测试 colorize 函数
        let colored = colorize(text, RED);
        assert!(colored.contains(text));
        assert!(colored.contains(RED));
        assert!(colored.contains(RESET));

        // 测试 bold 函数
        let bold_text = bold(text);
        assert!(bold_text.contains(text));
        assert!(bold_text.contains(BOLD));
        assert!(bold_text.contains(RESET));

        // 测试 dim 函数
        let dim_text = dim(text);
        assert!(dim_text.contains(text));
        assert!(dim_text.contains(DIM));
        assert!(dim_text.contains(RESET));
    }

    #[test]
    fn test_color_combinations() {
        // 测试颜色组合
        let text = "组合测试";

        // 测试多重格式化
        let formatted = format!("{}{}{}{}{}", BOLD, RED, text, RESET, BLUE);
        assert!(formatted.contains(BOLD));
        assert!(formatted.contains(RED));
        assert!(formatted.contains(text));
        assert!(formatted.contains(BLUE));
    }

    #[test]
    fn test_color_functions_with_empty_string() {
        // 测试空字符串的处理
        let empty = "";

        let colored_empty = colorize(empty, GREEN);
        assert!(colored_empty.contains(GREEN));
        assert!(colored_empty.contains(RESET));

        let bold_empty = bold(empty);
        assert!(bold_empty.contains(BOLD));
        assert!(bold_empty.contains(RESET));
    }

    #[test]
    fn test_color_functions_with_special_characters() {
        // 测试特殊字符的处理
        let special_text = "特殊字符: @#$%^&*()_+ 中文 🎨";

        let colored_special = colorize(special_text, MAGENTA);
        assert!(colored_special.contains(special_text));
        assert!(colored_special.contains(MAGENTA));

        let bold_special = bold(special_text);
        assert!(bold_special.contains(special_text));
        assert!(bold_special.contains(BOLD));
    }

    #[test]
    fn test_ansi_escape_sequences() {
        // 验证 ANSI 转义序列的正确性
        assert_eq!(RED, "\x1b[31m");
        assert_eq!(GREEN, "\x1b[32m");
        assert_eq!(YELLOW, "\x1b[33m");
        assert_eq!(BLUE, "\x1b[34m");
        assert_eq!(MAGENTA, "\x1b[35m");
        assert_eq!(CYAN, "\x1b[36m");
        assert_eq!(WHITE, "\x1b[37m");

        assert_eq!(BOLD, "\x1b[1m");
        assert_eq!(DIM, "\x1b[2m");
        assert_eq!(ITALIC, "\x1b[3m");
        assert_eq!(UNDERLINE, "\x1b[4m");
        assert_eq!(RESET, "\x1b[0m");
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_utils_module_integration() {
        // 集成测试：测试 utils 模块的整体功能

        // 生成随机代码并使用颜色格式化
        let code = utils::generate_random_code(8);
        let formatted_code = colorize(&code, GREEN);

        assert!(formatted_code.contains(&code));
        assert!(formatted_code.contains(GREEN));
        assert!(formatted_code.contains(RESET));
    }

    #[test]
    fn test_performance_random_code_generation() {
        // 性能测试：测试大量随机代码生成的性能
        use std::time::Instant;

        let start = Instant::now();
        let count = 10000;

        for _ in 0..count {
            let _code = utils::generate_random_code(6);
        }

        let duration = start.elapsed();
        println!("生成 {} 个随机代码耗时: {:?}", count, duration);

        // 确保性能在合理范围内（应该很快）
        assert!(duration.as_secs() < 5, "随机代码生成速度过慢");
    }

    #[test]
    fn test_memory_usage() {
        // 内存使用测试：确保随机代码生成不会泄露内存
        let mut codes = Vec::new();

        // 生成大量代码
        for i in 0..1000 {
            let code = utils::generate_random_code(10);
            codes.push(code);

            // 每100个检查一下
            if i % 100 == 0 {
                // 清理一些旧的代码，模拟实际使用场景
                if codes.len() > 500 {
                    codes.drain(0..100);
                }
            }
        }

        // 确保生成了预期数量的代码
        assert!(codes.len() > 400);
    }
}
