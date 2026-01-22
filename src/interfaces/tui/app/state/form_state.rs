//! 表单状态管理
//!
//! 管理添加/编辑链接时的表单输入和验证

use std::collections::HashMap;

/// 当前正在编辑的字段
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditingField {
    #[default]
    ShortCode,
    TargetUrl,
    ExpireTime,
    Password,
}

impl EditingField {
    /// 所有字段的顺序
    const ALL: [Self; 4] = [
        Self::ShortCode,
        Self::TargetUrl,
        Self::ExpireTime,
        Self::Password,
    ];

    /// 切换到下一个字段
    pub fn next(&self) -> Self {
        let idx = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    /// 切换到上一个字段
    #[allow(dead_code)]
    pub fn prev(&self) -> Self {
        let idx = Self::ALL.iter().position(|x| x == self).unwrap_or(0);
        if idx == 0 {
            Self::ALL[Self::ALL.len() - 1]
        } else {
            Self::ALL[idx - 1]
        }
    }

    /// 获取字段名称（用于验证错误的 key）
    pub fn field_name(&self) -> &'static str {
        match self {
            Self::ShortCode => "short_code",
            Self::TargetUrl => "target_url",
            Self::ExpireTime => "expire_time",
            Self::Password => "password",
        }
    }

    /// 获取字段显示标题
    pub fn display_title(&self) -> &'static str {
        match self {
            Self::ShortCode => "Short Code",
            Self::TargetUrl => "Target URL",
            Self::ExpireTime => "Expire Time",
            Self::Password => "Password",
        }
    }
}

/// 表单状态
#[derive(Debug, Default)]
pub struct FormState {
    /// 短码输入
    pub short_code: String,
    /// 目标 URL 输入
    pub target_url: String,
    /// 过期时间输入
    pub expire_time: String,
    /// 密码输入
    pub password: String,
    /// 强制覆盖选项
    pub force_overwrite: bool,
    /// 验证错误 (field_name -> error_message)
    pub validation_errors: HashMap<String, String>,
    /// 当前编辑的字段
    pub currently_editing: Option<EditingField>,
}

impl FormState {
    /// 创建新的表单状态
    pub fn new() -> Self {
        Self::default()
    }

    /// 清空所有输入
    pub fn clear(&mut self) {
        self.short_code.clear();
        self.target_url.clear();
        self.expire_time.clear();
        self.password.clear();
        self.force_overwrite = false;
        self.validation_errors.clear();
        self.currently_editing = None;
    }

    /// 切换到下一个编辑字段
    pub fn toggle_field(&mut self) {
        self.currently_editing = Some(match &self.currently_editing {
            Some(field) => field.next(),
            None => EditingField::default(),
        });
    }

    /// 切换强制覆盖选项（仅在 ShortCode 字段时生效）
    pub fn toggle_overwrite(&mut self) {
        if matches!(self.currently_editing, Some(EditingField::ShortCode)) {
            self.force_overwrite = !self.force_overwrite;
        }
    }

    /// 获取当前编辑字段的输入引用
    pub fn current_input(&self) -> Option<&String> {
        self.currently_editing.as_ref().map(|field| match field {
            EditingField::ShortCode => &self.short_code,
            EditingField::TargetUrl => &self.target_url,
            EditingField::ExpireTime => &self.expire_time,
            EditingField::Password => &self.password,
        })
    }

    /// 获取当前编辑字段的输入可变引用
    pub fn current_input_mut(&mut self) -> Option<&mut String> {
        match self.currently_editing {
            Some(EditingField::ShortCode) => Some(&mut self.short_code),
            Some(EditingField::TargetUrl) => Some(&mut self.target_url),
            Some(EditingField::ExpireTime) => Some(&mut self.expire_time),
            Some(EditingField::Password) => Some(&mut self.password),
            None => None,
        }
    }

    /// 向当前编辑字段添加字符
    pub fn push_char(&mut self, c: char) {
        if let Some(input) = self.current_input_mut() {
            input.push(c);
        }
    }

    /// 从当前编辑字段删除最后一个字符
    pub fn pop_char(&mut self) {
        if let Some(input) = self.current_input_mut() {
            input.pop();
        }
    }

    /// 获取指定字段的验证错误
    pub fn get_error(&self, field: EditingField) -> Option<&String> {
        self.validation_errors.get(field.field_name())
    }

    /// 设置验证错误
    pub fn set_error(&mut self, field: EditingField, error: String) {
        self.validation_errors
            .insert(field.field_name().to_string(), error);
    }

    /// 清除验证错误
    pub fn clear_errors(&mut self) {
        self.validation_errors.clear();
    }

    /// 检查是否有验证错误
    pub fn has_errors(&self) -> bool {
        !self.validation_errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editing_field_next() {
        assert_eq!(EditingField::ShortCode.next(), EditingField::TargetUrl);
        assert_eq!(EditingField::TargetUrl.next(), EditingField::ExpireTime);
        assert_eq!(EditingField::ExpireTime.next(), EditingField::Password);
        assert_eq!(EditingField::Password.next(), EditingField::ShortCode);
    }

    #[test]
    fn test_editing_field_prev() {
        assert_eq!(EditingField::ShortCode.prev(), EditingField::Password);
        assert_eq!(EditingField::TargetUrl.prev(), EditingField::ShortCode);
        assert_eq!(EditingField::ExpireTime.prev(), EditingField::TargetUrl);
        assert_eq!(EditingField::Password.prev(), EditingField::ExpireTime);
    }

    #[test]
    fn test_form_state_toggle_field() {
        let mut form = FormState::new();
        assert!(form.currently_editing.is_none());

        form.toggle_field();
        assert_eq!(form.currently_editing, Some(EditingField::ShortCode));

        form.toggle_field();
        assert_eq!(form.currently_editing, Some(EditingField::TargetUrl));

        form.toggle_field();
        assert_eq!(form.currently_editing, Some(EditingField::ExpireTime));

        form.toggle_field();
        assert_eq!(form.currently_editing, Some(EditingField::Password));

        form.toggle_field();
        assert_eq!(form.currently_editing, Some(EditingField::ShortCode));
    }

    #[test]
    fn test_form_state_input() {
        let mut form = FormState::new();
        form.currently_editing = Some(EditingField::ShortCode);

        form.push_char('a');
        form.push_char('b');
        form.push_char('c');
        assert_eq!(form.short_code, "abc");

        form.pop_char();
        assert_eq!(form.short_code, "ab");

        form.toggle_field();
        form.push_char('x');
        assert_eq!(form.target_url, "x");
    }

    #[test]
    fn test_form_state_toggle_overwrite() {
        let mut form = FormState::new();
        form.currently_editing = Some(EditingField::ShortCode);
        assert!(!form.force_overwrite);

        form.toggle_overwrite();
        assert!(form.force_overwrite);

        form.toggle_overwrite();
        assert!(!form.force_overwrite);

        // 在其他字段时不应该切换
        form.currently_editing = Some(EditingField::TargetUrl);
        form.toggle_overwrite();
        assert!(!form.force_overwrite);
    }

    #[test]
    fn test_form_state_clear() {
        let mut form = FormState::new();
        form.short_code = "test".to_string();
        form.target_url = "https://example.com".to_string();
        form.force_overwrite = true;
        form.currently_editing = Some(EditingField::ShortCode);
        form.set_error(EditingField::ShortCode, "Test error".to_string());

        form.clear();

        assert!(form.short_code.is_empty());
        assert!(form.target_url.is_empty());
        assert!(!form.force_overwrite);
        assert!(form.currently_editing.is_none());
        assert!(!form.has_errors());
    }
}
