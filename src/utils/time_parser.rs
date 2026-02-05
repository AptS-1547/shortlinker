use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct TimeParser;

impl TimeParser {
    /// 解析时间字符串，支持多种格式：
    /// - RFC3339 格式：2023-10-01T12:00:00Z
    /// - 相对时间：1d, 2w, 3M (大写M表示月), 1y, 1h30m, 2d12h
    /// - 组合格式：1d2h30m
    ///
    /// 注意：m 表示分钟，M 表示月份
    pub fn parse_expire_time(input: &str) -> Result<DateTime<Utc>, String> {
        let input = input.trim();

        // 尝试解析 RFC3339 格式
        if let Ok(dt) = DateTime::parse_from_rfc3339(input) {
            return Ok(dt.with_timezone(&Utc));
        }

        // 尝试解析相对时间格式
        Self::parse_relative_time(input)
    }

    /// 解析过期时间，带有详细的格式帮助信息
    ///
    /// 适用于 CLI 等需要友好错误提示的场景
    pub fn parse_expire_time_with_help(input: &str) -> Result<DateTime<Utc>, String> {
        Self::parse_expire_time(input).map_err(|e| {
            format!(
                "Invalid expiration time format: {}. Supported formats:\n  \
                - RFC3339: 2023-10-01T12:00:00Z\n  \
                - Relative time: 1d, 2w, 1y, 1d2h30m",
                e
            )
        })
    }

    fn parse_relative_time(input: &str) -> Result<DateTime<Utc>, String> {
        let mut total_duration = Duration::zero();
        let mut remaining = input;

        while !remaining.is_empty() {
            // Find the position where digits end
            let digit_end = remaining
                .char_indices()
                .find(|(_, c)| !c.is_ascii_digit())
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());

            if digit_end == 0 {
                return Err(format!("Invalid time format: '{}'", input));
            }

            let num_str = &remaining[..digit_end];
            let num: i64 = num_str
                .parse()
                .map_err(|_| format!("Invalid number: '{}'", num_str))?;

            remaining = &remaining[digit_end..];

            // Find the position where letters end
            let unit_end = remaining
                .char_indices()
                .find(|(_, c)| !c.is_alphabetic())
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());

            if unit_end == 0 {
                return Err(format!("Missing time unit after number '{}'", num));
            }

            let unit_str = &remaining[..unit_end];

            // 解析单位并计算持续时间
            // 注意：大写 M 表示月份，小写 m 表示分钟
            // 使用 try_* 方法避免极端数值导致 panic
            let lower_unit_str = unit_str.to_lowercase();
            let duration =
                if unit_str == "M" || lower_unit_str == "month" || lower_unit_str == "months" {
                    // Approximate 30 days per month
                    let days = num
                        .checked_mul(30)
                        .ok_or_else(|| format!("Time value overflow: {} * 30", num))?;
                    Duration::try_days(days)
                        .ok_or_else(|| format!("Duration out of range: {} days", days))?
                } else {
                    match lower_unit_str.as_str() {
                        "s" | "sec" | "second" | "seconds" => Duration::try_seconds(num)
                            .ok_or_else(|| format!("Duration out of range: {} seconds", num))?,
                        "m" | "min" | "minute" | "minutes" => Duration::try_minutes(num)
                            .ok_or_else(|| format!("Duration out of range: {} minutes", num))?,
                        "h" | "hour" | "hours" => Duration::try_hours(num)
                            .ok_or_else(|| format!("Duration out of range: {} hours", num))?,
                        "d" | "day" | "days" => Duration::try_days(num)
                            .ok_or_else(|| format!("Duration out of range: {} days", num))?,
                        "w" | "week" | "weeks" => Duration::try_weeks(num)
                            .ok_or_else(|| format!("Duration out of range: {} weeks", num))?,
                        "y" | "year" | "years" => {
                            // Approximate 365 days per year
                            let days = num
                                .checked_mul(365)
                                .ok_or_else(|| format!("Time value overflow: {} * 365", num))?;
                            Duration::try_days(days)
                                .ok_or_else(|| format!("Duration out of range: {} days", days))?
                        }
                        _ => return Err(format!("Unsupported time unit: '{}'", unit_str)),
                    }
                };

            total_duration += duration;
            remaining = &remaining[unit_end..];
        }

        if total_duration == Duration::zero() {
            return Err("Duration cannot be zero".to_string());
        }

        let now = Utc::now();
        match now.checked_add_signed(total_duration) {
            Some(future_time) => Ok(future_time),
            None => Err("Calculated expiration time is out of valid range".to_string()),
        }
    }

    /// 格式化持续时间为人类可读的字符串
    pub fn format_duration_human(from: DateTime<Utc>, to: DateTime<Utc>) -> String {
        let duration = to.signed_duration_since(from);

        if duration.num_seconds() < 0 {
            return "Expired".to_string();
        }

        let days = duration.num_days();
        let hours = (duration.num_seconds() % 86400) / 3600;
        let minutes = (duration.num_seconds() % 3600) / 60;

        if days > 0 {
            if hours > 0 {
                format!("{}d {}h", days, hours)
            } else {
                format!("{}d", days)
            }
        } else if hours > 0 {
            if minutes > 0 {
                format!("{}h {}m", hours, minutes)
            } else {
                format!("{}h", hours)
            }
        } else if minutes > 0 {
            format!("{}m", minutes)
        } else {
            format!("{}s", duration.num_seconds())
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
    fn test_parse_month_vs_minute() {
        let now = Utc::now();

        // 测试大写 M (月份)
        let result = TimeParser::parse_expire_time("1M").unwrap();
        let days_diff = (result - now).num_days();
        assert!(
            (29..=31).contains(&days_diff),
            "1M should be approximately 30 days, got {}",
            days_diff
        );

        // 测试小写 m (分钟)
        let result = TimeParser::parse_expire_time("1m").unwrap();
        let seconds_diff = (result - now).num_seconds();
        assert!(
            (59..=61).contains(&seconds_diff),
            "1m should be approximately 60 seconds, got {}",
            seconds_diff
        );

        // 测试完整单词 month
        let result = TimeParser::parse_expire_time("2months").unwrap();
        let days_diff = (result - now).num_days();
        assert!(
            (59..=61).contains(&days_diff),
            "2months should be approximately 60 days, got {}",
            days_diff
        );

        // 测试完整单词 minute
        let result = TimeParser::parse_expire_time("30minutes").unwrap();
        let seconds_diff = (result - now).num_seconds();
        assert!(
            (1799..=1801).contains(&seconds_diff),
            "30minutes should be approximately 1800 seconds, got {}",
            seconds_diff
        );

        // 测试组合：3个月和30分钟
        let result = TimeParser::parse_expire_time("3M30m").unwrap();
        let expected_seconds = 3 * 30 * 24 * 3600 + 30 * 60; // 3 months + 30 minutes
        let actual_seconds = (result - now).num_seconds();
        assert!(
            (actual_seconds - expected_seconds).abs() < 5,
            "3M30m should be approximately {} seconds, got {}",
            expected_seconds,
            actual_seconds
        );
    }

    #[test]
    fn test_invalid_format() {
        assert!(TimeParser::parse_expire_time("invalid").is_err());
        assert!(TimeParser::parse_expire_time("1x").is_err());
        assert!(TimeParser::parse_expire_time("abc").is_err());
    }

    #[test]
    fn test_extreme_values_no_panic() {
        // 测试极端数值不会 panic，而是返回错误
        // 这些值会导致 chrono::Duration 溢出
        let extreme_cases = [
            "999999999999999999d",
            "999999999999999999y",
            "999999999999999999w",
            "999999999999999999h",
            "999999999999999999m",
            "999999999999999999s",
            "999999999999999999M",
        ];

        for case in extreme_cases {
            let result = TimeParser::parse_expire_time(case);
            assert!(
                result.is_err(),
                "Extreme value '{}' should return an error instead of panicking",
                case
            );
        }
    }

    #[test]
    fn test_overflow_multiplication() {
        // 测试乘法溢出的情况 (num * 30 和 num * 365)
        let result = TimeParser::parse_expire_time("999999999999999M"); // months: num * 30
        assert!(
            result.is_err(),
            "Month multiplication overflow should return an error"
        );

        let result = TimeParser::parse_expire_time("999999999999999y"); // years: num * 365
        assert!(
            result.is_err(),
            "Year multiplication overflow should return an error"
        );
    }

    #[test]
    fn test_reasonable_large_values() {
        // 合理的大值应该正常工作
        let result = TimeParser::parse_expire_time("100y"); // 100 年
        assert!(result.is_ok(), "100y should be a valid value");

        let result = TimeParser::parse_expire_time("3650d"); // 约 10 年
        assert!(result.is_ok(), "3650d should be a valid value");

        let result = TimeParser::parse_expire_time("520w"); // 约 10 年
        assert!(result.is_ok(), "520w should be a valid value");
    }
}
