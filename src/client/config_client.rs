//! Configuration management client (IPC-first + ConfigService-fallback)

use std::sync::Arc;

use crate::services::{ConfigItemView, ConfigUpdateView};
use crate::system::ipc::{self, ConfigItemData, IpcResponse};

use super::context::ServiceContext;
use super::{ClientError, ipc_or_fallback};

/// Configuration operations client.
///
/// IPC-first with ConfigService-fallback for all operations.
pub struct ConfigClient {
    ctx: Arc<ServiceContext>,
}

impl ConfigClient {
    pub fn new(ctx: Arc<ServiceContext>) -> Self {
        Self { ctx }
    }

    /// List all configurations, optionally filtered by category
    pub async fn get_all(
        &self,
        category: Option<String>,
    ) -> Result<Vec<ConfigItemView>, ClientError> {
        let ctx = self.ctx.clone();
        let cat = category.clone();
        ipc_or_fallback(
            ipc::config_list(category),
            |resp| match resp {
                IpcResponse::ConfigListResult { configs } => {
                    Ok(configs.into_iter().map(config_data_to_view).collect())
                }
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_config_service().await?;
                let mut items = service.get_all();
                if let Some(cat) = cat {
                    items.retain(|item| {
                        // Match category from the key prefix (e.g., "auth.xxx" → "auth")
                        item.key.starts_with(&cat)
                    });
                }
                Ok(items)
            },
        )
        .await
    }

    /// Get a single configuration
    pub async fn get(&self, key: String) -> Result<ConfigItemView, ClientError> {
        let ctx = self.ctx.clone();
        let key2 = key.clone();
        ipc_or_fallback(
            ipc::config_get(key),
            |resp| match resp {
                IpcResponse::ConfigGetResult { config } => Ok(config_data_to_view(config)),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_config_service().await?;
                Ok(service.get(&key2)?)
            },
        )
        .await
    }

    /// Set a configuration value
    pub async fn set(&self, key: String, value: String) -> Result<ConfigUpdateView, ClientError> {
        let ctx = self.ctx.clone();
        let key2 = key.clone();
        let value2 = value.clone();
        ipc_or_fallback(
            ipc::config_set(key, value),
            |resp| match resp {
                IpcResponse::ConfigSetResult {
                    key,
                    value,
                    requires_restart,
                    is_sensitive,
                    message,
                    ..
                } => Ok(ConfigUpdateView {
                    key,
                    value,
                    requires_restart,
                    is_sensitive,
                    message,
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_config_service().await?;
                Ok(service.update(&key2, &value2).await?)
            },
        )
        .await
    }

    /// Reset a configuration to its default value
    pub async fn reset(&self, key: String) -> Result<ConfigUpdateView, ClientError> {
        let ctx = self.ctx.clone();
        let key2 = key.clone();
        ipc_or_fallback(
            ipc::config_reset(key),
            |resp| match resp {
                IpcResponse::ConfigResetResult {
                    key,
                    value,
                    requires_restart,
                    is_sensitive,
                    message,
                } => Ok(ConfigUpdateView {
                    key,
                    value,
                    requires_restart,
                    is_sensitive,
                    message,
                }),
                other => Err(unexpected_response(other)),
            },
            || async move {
                let service = ctx.get_config_service().await?;
                // Reset = set to default value
                let def = crate::config::definitions::get_def(&key2).ok_or_else(|| {
                    ClientError::Service(crate::errors::ShortlinkerError::config_not_found(
                        format!("Unknown config key: {}", key2),
                    ))
                })?;
                let default_value = (def.default_fn)();
                Ok(service.update(&key2, &default_value).await?)
            },
        )
        .await
    }
}

// ============ Conversion helpers ============

/// Convert IPC ConfigItemData to service ConfigItemView
fn config_data_to_view(data: ConfigItemData) -> ConfigItemView {
    ConfigItemView {
        key: data.key,
        value: data.value,
        value_type: parse_value_type(&data.value_type),
        requires_restart: data.requires_restart,
        is_sensitive: data.sensitive,
        updated_at: chrono::DateTime::parse_from_rfc3339(&data.updated_at)
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(|e| {
                tracing::warn!(
                    "Failed to parse 'updated_at' from IPC (value: '{}'): {}",
                    &data.updated_at,
                    e
                );
                chrono::Utc::now()
            }),
    }
}

fn parse_value_type(s: &str) -> crate::config::ValueType {
    match s.to_lowercase().as_str() {
        "bool" => crate::config::ValueType::Bool,
        "int" => crate::config::ValueType::Int,
        "float" => crate::config::ValueType::Float,
        "json" => crate::config::ValueType::Json,
        "enum" => crate::config::ValueType::Enum,
        _ => crate::config::ValueType::String,
    }
}

fn unexpected_response(resp: IpcResponse) -> ClientError {
    ClientError::Ipc(crate::system::ipc::IpcError::ProtocolError(format!(
        "Unexpected response: {:?}",
        resp
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ValueType;

    fn make_config_data(
        key: &str,
        value: &str,
        value_type: &str,
        sensitive: bool,
        requires_restart: bool,
        updated_at: &str,
    ) -> ConfigItemData {
        ConfigItemData {
            key: key.into(),
            value: value.into(),
            category: key.split('.').next().unwrap_or("unknown").into(),
            value_type: value_type.into(),
            default_value: "".into(),
            requires_restart,
            editable: true,
            sensitive,
            description: format!("Description for {}", key),
            enum_options: None,
            updated_at: updated_at.into(),
        }
    }

    // ---- parse_value_type tests ----

    #[test]
    fn test_parse_value_type_bool_lowercase() {
        assert!(matches!(parse_value_type("bool"), ValueType::Bool));
    }

    #[test]
    fn test_parse_value_type_bool_titlecase() {
        assert!(matches!(parse_value_type("Bool"), ValueType::Bool));
    }

    #[test]
    fn test_parse_value_type_int_lowercase() {
        assert!(matches!(parse_value_type("int"), ValueType::Int));
    }

    #[test]
    fn test_parse_value_type_int_titlecase() {
        assert!(matches!(parse_value_type("Int"), ValueType::Int));
    }

    #[test]
    fn test_parse_value_type_float_lowercase() {
        assert!(matches!(parse_value_type("float"), ValueType::Float));
    }

    #[test]
    fn test_parse_value_type_float_titlecase() {
        assert!(matches!(parse_value_type("Float"), ValueType::Float));
    }

    #[test]
    fn test_parse_value_type_json_lowercase() {
        assert!(matches!(parse_value_type("json"), ValueType::Json));
    }

    #[test]
    fn test_parse_value_type_json_titlecase() {
        assert!(matches!(parse_value_type("Json"), ValueType::Json));
    }

    #[test]
    fn test_parse_value_type_enum_lowercase() {
        assert!(matches!(parse_value_type("enum"), ValueType::Enum));
    }

    #[test]
    fn test_parse_value_type_enum_titlecase() {
        assert!(matches!(parse_value_type("Enum"), ValueType::Enum));
    }

    #[test]
    fn test_parse_value_type_string_explicit() {
        assert!(matches!(parse_value_type("string"), ValueType::String));
    }

    #[test]
    fn test_parse_value_type_unknown_defaults_to_string() {
        assert!(matches!(parse_value_type("unknown"), ValueType::String));
        assert!(matches!(parse_value_type(""), ValueType::String));
        assert!(matches!(parse_value_type("INTEGER"), ValueType::String));
    }

    #[test]
    fn test_parse_value_type_case_insensitive() {
        assert!(matches!(parse_value_type("BOOL"), ValueType::Bool));
        assert!(matches!(parse_value_type("INT"), ValueType::Int));
        assert!(matches!(parse_value_type("FLOAT"), ValueType::Float));
        assert!(matches!(parse_value_type("JSON"), ValueType::Json));
        assert!(matches!(parse_value_type("ENUM"), ValueType::Enum));
    }

    // ---- config_data_to_view tests ----

    #[test]
    fn test_config_data_to_view_basic() {
        let data = make_config_data(
            "auth.admin_token",
            "[REDACTED]",
            "string",
            true,
            false,
            "2025-01-01T00:00:00Z",
        );
        let view = config_data_to_view(data);
        assert_eq!(view.key, "auth.admin_token");
        assert_eq!(view.value, "[REDACTED]");
        assert!(view.is_sensitive);
        assert!(!view.requires_restart);
        assert!(matches!(view.value_type, ValueType::String));
    }

    #[test]
    fn test_config_data_to_view_bool_type() {
        let data = make_config_data(
            "features.enabled",
            "true",
            "Bool",
            false,
            true,
            "2025-06-15T12:00:00Z",
        );
        let view = config_data_to_view(data);
        assert!(matches!(view.value_type, ValueType::Bool));
        assert!(view.requires_restart);
        assert!(!view.is_sensitive);
    }

    #[test]
    fn test_config_data_to_view_valid_date() {
        let data = make_config_data(
            "k",
            "v",
            "string",
            false,
            false,
            "2025-03-15T10:30:00+08:00",
        );
        let view = config_data_to_view(data);
        // Should parse the timezone-aware date correctly
        assert_eq!(view.updated_at.year(), 2025);
        assert_eq!(view.updated_at.month(), 3);
        // 10:30 +08:00 = 02:30 UTC
        assert_eq!(view.updated_at.hour(), 2);
    }

    #[test]
    fn test_config_data_to_view_invalid_date_falls_back_to_now() {
        let data = make_config_data("k", "v", "string", false, false, "not-a-date");
        let view = config_data_to_view(data);
        let now = chrono::Utc::now();
        let diff = (now - view.updated_at).num_seconds().abs();
        assert!(diff < 5, "Expected updated_at near now, diff={}s", diff);
    }

    #[test]
    fn test_config_data_to_view_with_enum_options() {
        let data = ConfigItemData {
            key: "routes.mode".into(),
            value: "redirect".into(),
            category: "routes".into(),
            value_type: "enum".into(),
            default_value: "redirect".into(),
            requires_restart: false,
            editable: true,
            sensitive: false,
            description: "Route mode".into(),
            enum_options: Some(vec!["redirect".into(), "proxy".into()]),
            updated_at: "2025-01-01T00:00:00Z".into(),
        };
        let view = config_data_to_view(data);
        assert!(matches!(view.value_type, ValueType::Enum));
        assert_eq!(view.value, "redirect");
    }

    // ---- unexpected_response tests ----

    #[test]
    fn test_unexpected_response_returns_ipc_error() {
        let err = unexpected_response(IpcResponse::Pong {
            version: "1.0".into(),
            uptime_secs: 0,
        });
        assert!(matches!(err, ClientError::Ipc(_)));
        let msg = format!("{}", err);
        assert!(msg.contains("Unexpected response"), "got: {}", msg);
    }

    use chrono::{Datelike, Timelike};
}
