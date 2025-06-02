use std::fmt;

#[derive(Debug, Clone)]
pub enum ShortlinkerError {
    DatabaseConnection(String),
    DatabaseOperation(String),
    FileOperation(String),
    Validation(String),
    NotFound(String),
    Serialization(String),
    SignalOperation(String),
    DateParse(String),
}

impl fmt::Display for ShortlinkerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShortlinkerError::DatabaseConnection(msg) => write!(f, "数据库连接错误: {}", msg),
            ShortlinkerError::DatabaseOperation(msg) => write!(f, "数据库操作错误: {}", msg),
            ShortlinkerError::FileOperation(msg) => write!(f, "文件操作错误: {}", msg),
            ShortlinkerError::Validation(msg) => write!(f, "验证错误: {}", msg),
            ShortlinkerError::NotFound(msg) => write!(f, "资源未找到: {}", msg),
            ShortlinkerError::Serialization(msg) => write!(f, "序列化错误: {}", msg),
            ShortlinkerError::SignalOperation(msg) => write!(f, "信号操作错误: {}", msg),
            ShortlinkerError::DateParse(msg) => write!(f, "日期解析错误: {}", msg),
        }
    }
}

impl std::error::Error for ShortlinkerError {}

// 便捷的构造函数
impl ShortlinkerError {
    pub fn database_connection<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DatabaseConnection(msg.into())
    }

    pub fn database_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::DatabaseOperation(msg.into())
    }

    pub fn file_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::FileOperation(msg.into())
    }

    pub fn validation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::Validation(msg.into())
    }

    pub fn not_found<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::NotFound(msg.into())
    }

    pub fn serialization<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::Serialization(msg.into())
    }

    pub fn signal_operation<T: Into<String>>(msg: T) -> Self {
        ShortlinkerError::SignalOperation(msg.into())
    }
}

// 为常见的错误类型实现 From trait
impl From<rusqlite::Error> for ShortlinkerError {
    fn from(err: rusqlite::Error) -> Self {
        ShortlinkerError::DatabaseOperation(err.to_string())
    }
}

impl From<std::io::Error> for ShortlinkerError {
    fn from(err: std::io::Error) -> Self {
        ShortlinkerError::FileOperation(err.to_string())
    }
}

impl From<serde_json::Error> for ShortlinkerError {
    fn from(err: serde_json::Error) -> Self {
        ShortlinkerError::Serialization(err.to_string())
    }
}

impl From<chrono::ParseError> for ShortlinkerError {
    fn from(err: chrono::ParseError) -> Self {
        ShortlinkerError::Validation(format!("时间解析错误: {}", err))
    }
}

pub type Result<T> = std::result::Result<T, ShortlinkerError>;
