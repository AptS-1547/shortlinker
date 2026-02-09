//! 链接状态指示器
//!
//! 显示链接的状态（锁定、活跃、即将过期、已过期）

use chrono::{DateTime, Utc};
use ratatui::style::{Color, Style};

use crate::interfaces::tui::constants::{colors, status_text};

/// 链接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkStatus {
    /// 活跃状态
    Active,
    /// 即将过期（24小时内）
    Expiring,
    /// 已过期
    Expired,
}

impl LinkStatus {
    /// 从过期时间计算状态
    pub fn from_expires_at(expires_at: Option<DateTime<Utc>>) -> Self {
        match expires_at {
            Some(exp) => {
                let now = Utc::now();
                if exp <= now {
                    Self::Expired
                } else if (exp - now).num_hours() < 24 {
                    Self::Expiring
                } else {
                    Self::Active
                }
            }
            None => Self::Active,
        }
    }

    /// 获取状态文本
    pub fn text(&self) -> &'static str {
        match self {
            Self::Active => status_text::ACTIVE,
            Self::Expiring => status_text::EXPIRING,
            Self::Expired => status_text::EXPIRED,
        }
    }

    /// 获取状态颜色
    #[allow(dead_code)]
    pub fn color(&self) -> Color {
        match self {
            Self::Active => colors::SUCCESS,
            Self::Expiring => colors::WARNING,
            Self::Expired => colors::ERROR,
        }
    }

    /// 获取状态样式
    #[allow(dead_code)]
    pub fn style(&self) -> Style {
        Style::default().fg(self.color())
    }
}

/// 状态指示器组件
pub struct StatusIndicator {
    /// 是否有密码保护
    pub has_password: bool,
    /// 链接状态
    pub status: LinkStatus,
}

impl StatusIndicator {
    /// 创建状态指示器
    pub fn new(has_password: bool, expires_at: Option<DateTime<Utc>>) -> Self {
        Self {
            has_password,
            status: LinkStatus::from_expires_at(expires_at),
        }
    }

    /// 获取完整的状态文本
    pub fn text(&self) -> String {
        let mut parts = Vec::new();

        if self.has_password {
            parts.push(status_text::LOCKED);
        }

        parts.push(self.status.text());

        parts.join(" ")
    }

    /// 获取主要颜色（基于状态）
    #[allow(dead_code)]
    pub fn color(&self) -> Color {
        self.status.color()
    }

    /// 获取样式
    #[allow(dead_code)]
    pub fn style(&self) -> Style {
        self.status.style()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_link_status_active() {
        let status = LinkStatus::from_expires_at(None);
        assert_eq!(status, LinkStatus::Active);

        let future = Some(Utc::now() + Duration::days(7));
        let status = LinkStatus::from_expires_at(future);
        assert_eq!(status, LinkStatus::Active);
    }

    #[test]
    fn test_link_status_expiring() {
        let soon = Some(Utc::now() + Duration::hours(12));
        let status = LinkStatus::from_expires_at(soon);
        assert_eq!(status, LinkStatus::Expiring);
    }

    #[test]
    fn test_link_status_expired() {
        let past = Some(Utc::now() - Duration::hours(1));
        let status = LinkStatus::from_expires_at(past);
        assert_eq!(status, LinkStatus::Expired);
    }

    #[test]
    fn test_status_indicator_text() {
        let indicator = StatusIndicator::new(true, None);
        assert!(indicator.text().contains("LOCKED"));
        assert!(indicator.text().contains("ACTIVE"));

        let indicator = StatusIndicator::new(false, None);
        assert!(!indicator.text().contains("LOCKED"));
        assert!(indicator.text().contains("ACTIVE"));
    }
}
