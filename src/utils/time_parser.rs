use chrono::{DateTime, Duration, Utc};
pub use crate::structs::TimeParser;


impl TimeParser {
    /// 解析时间字符串，支持多种格式：
    /// - RFC3339 格式：2023-10-01T12:00:00Z
    /// - 相对时间：1d, 2w, 3m, 1y, 1h30m, 2d12h
    /// - 组合格式：1d2h30m
    pub fn parse_expire_time(input: &str) -> Result<DateTime<Utc>, String> {
        let input = input.trim();

        // 尝试解析 RFC3339 格式
        if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
            return Ok(dt.with_timezone(&Utc));
        }

        // 尝试解析相对时间格式
        Self::parse_relative_time(input)
    }

    fn parse_relative_time(input: &str) -> Result<DateTime<Utc>, String> {
        let mut total_duration = Duration::zero();
        let mut remaining = input;

        while !remaining.is_empty() {
            // 提取数字
            let mut num_str = String::new();
            let chars = remaining.chars();

            for c in chars {
                if c.is_ascii_digit() {
                    num_str.push(c);
                } else {
                    remaining = &remaining[num_str.len()..];
                    break;
                }
            }

            if num_str.is_empty() {
                return Err(format!("无效的时间格式: '{}'", input));
            }

            let num: i64 = num_str
                .parse()
                .map_err(|_| format!("无效的数字: '{}'", num_str))?;

            // 提取单位
            let mut unit_str = String::new();
            let chars = remaining.chars();

            for c in chars {
                if c.is_alphabetic() {
                    unit_str.push(c);
                } else {
                    break;
                }
            }

            if unit_str.is_empty() {
                return Err(format!("缺少时间单位，数字 '{}' 后应跟时间单位", num));
            }

            // 解析单位并计算持续时间
            let duration = match unit_str.to_lowercase().as_str() {
                "s" | "sec" | "second" | "seconds" => Duration::seconds(num),
                "m" | "min" | "minute" | "minutes" => Duration::minutes(num),
                "h" | "hour" | "hours" => Duration::hours(num),
                "d" | "day" | "days" => Duration::days(num),
                "w" | "week" | "weeks" => Duration::weeks(num),
                "M" | "month" | "months" => Duration::days(num * 30), // 近似30天
                "y" | "year" | "years" => Duration::days(num * 365),  // 近似365天
                _ => return Err(format!("不支持的时间单位: '{}'", unit_str)),
            };

            total_duration += duration;
            remaining = &remaining[unit_str.len()..];
        }

        if total_duration == Duration::zero() {
            return Err("时间间隔不能为零".to_string());
        }

        let now = Utc::now();
        match now.checked_add_signed(total_duration) {
            Some(future_time) => Ok(future_time),
            None => Err("计算的过期时间超出了有效范围".to_string()),
        }
    }

    /// 格式化持续时间为人类可读的字符串
    pub fn format_duration_human(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
        let duration = to.signed_duration_since(from);

        if duration.num_seconds() < 0 {
            return "已过期".to_string();
        }

        let days = duration.num_days();
        let hours = (duration.num_seconds() % 86400) / 3600;
        let minutes = (duration.num_seconds() % 3600) / 60;

        if days > 0 {
            if hours > 0 {
                format!("{}天{}小时", days, hours)
            } else {
                format!("{}天", days)
            }
        } else if hours > 0 {
            if minutes > 0 {
                format!("{}小时{}分钟", hours, minutes)
            } else {
                format!("{}小时", hours)
            }
        } else if minutes > 0 {
            format!("{}分钟", minutes)
        } else {
            format!("{}秒", duration.num_seconds())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relative_time() {
        let now = Utc::now();

        // 测试单个单位
        let result = TimeParser::parse_expire_time("1d").unwrap();
        assert!((result - now).num_days() == 1);

        let result = TimeParser::parse_expire_time("2w").unwrap();
        assert!((result - now).num_days() == 14);

        let result = TimeParser::parse_expire_time("1y").unwrap();
        assert!((result - now).num_days() == 365);

        // 测试组合格式
        let result = TimeParser::parse_expire_time("1d2h30m").unwrap();
        let expected_seconds = 24 * 3600 + 2 * 3600 + 30 * 60;
        let actual_seconds = (result - now).num_seconds();
        assert!((actual_seconds - expected_seconds).abs() < 5); // 允许5秒误差
    }

    #[test]
    fn test_parse_rfc3339() {
        let result = TimeParser::parse_expire_time("2023-10-01T12:00:00Z");
        assert!(result.is_ok());
    }

    #[test]
    fn test_invalid_format() {
        assert!(TimeParser::parse_expire_time("invalid").is_err());
        assert!(TimeParser::parse_expire_time("1x").is_err());
        assert!(TimeParser::parse_expire_time("abc").is_err());
    }
}
