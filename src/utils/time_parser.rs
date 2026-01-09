use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone)]
pub struct TimeParser;

impl TimeParser {
    /// 解析时间字符串，支持多种格式：
    /// - RFC3339 格式：2023-10-01T12:00:00Z
    /// - 相对时间：1d, 2w, 3M (大写M表示月), 1y, 1h30m, 2d12h
    /// - 组合格式：1d2h30m
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
                return Err(format!("无效的时间格式: '{}'", input));
            }

            let num_str = &remaining[..digit_end];
            let num: i64 = num_str
                .parse()
                .map_err(|_| format!("无效的数字: '{}'", num_str))?;

            remaining = &remaining[digit_end..];

            // Find the position where letters end
            let unit_end = remaining
                .char_indices()
                .find(|(_, c)| !c.is_alphabetic())
                .map(|(i, _)| i)
                .unwrap_or(remaining.len());

            if unit_end == 0 {
                return Err(format!("缺少时间单位，数字 '{}' 后应跟时间单位", num));
            }

            let unit_str = &remaining[..unit_end];

            // 解析单位并计算持续时间
            // 注意：大写 M 表示月份，小写 m 表示分钟
            let lower_unit_str = unit_str.to_lowercase();
            let duration =
                if unit_str == "M" || lower_unit_str == "month" || lower_unit_str == "months" {
                    Duration::days(num * 30) // 近似30天
                } else {
                    match lower_unit_str.as_str() {
                        "s" | "sec" | "second" | "seconds" => Duration::seconds(num),
                        "m" | "min" | "minute" | "minutes" => Duration::minutes(num),
                        "h" | "hour" | "hours" => Duration::hours(num),
                        "d" | "day" | "days" => Duration::days(num),
                        "w" | "week" | "weeks" => Duration::weeks(num),
                        "y" | "year" | "years" => Duration::days(num * 365), // 近似365天
                        _ => return Err(format!("不支持的时间单位: '{}'", unit_str)),
                    }
                };

            total_duration += duration;
            remaining = &remaining[unit_end..];
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
    fn test_parse_month_vs_minute() {
        let now = Utc::now();

        // 测试大写 M (月份)
        let result = TimeParser::parse_expire_time("1M").unwrap();
        let days_diff = (result - now).num_days();
        assert!(
            days_diff >= 29 && days_diff <= 31,
            "1M should be approximately 30 days, got {}",
            days_diff
        );

        // 测试小写 m (分钟)
        let result = TimeParser::parse_expire_time("1m").unwrap();
        let seconds_diff = (result - now).num_seconds();
        assert!(
            seconds_diff >= 59 && seconds_diff <= 61,
            "1m should be approximately 60 seconds, got {}",
            seconds_diff
        );

        // 测试完整单词 month
        let result = TimeParser::parse_expire_time("2months").unwrap();
        let days_diff = (result - now).num_days();
        assert!(
            days_diff >= 59 && days_diff <= 61,
            "2months should be approximately 60 days, got {}",
            days_diff
        );

        // 测试完整单词 minute
        let result = TimeParser::parse_expire_time("30minutes").unwrap();
        let seconds_diff = (result - now).num_seconds();
        assert!(
            seconds_diff >= 1799 && seconds_diff <= 1801,
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
}
