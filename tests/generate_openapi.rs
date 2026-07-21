#![cfg(all(debug_assertions, feature = "openapi"))]
//! OpenAPI 生成和 wire contract 回归测试。

use std::collections::HashSet;
use std::fs;

use shortlinker::api::openapi::ApiDoc;
use utoipa::OpenApi;

#[test]
fn generate_openapi() {
    let json = serde_json::to_string_pretty(&ApiDoc::openapi()).expect("serialize OpenAPI");
    fs::create_dir_all("./admin-panel/generated").expect("create OpenAPI output directory");
    fs::write("./admin-panel/generated/openapi.json", format!("{json}\n"))
        .expect("write OpenAPI document");
}

#[test]
fn error_code_schema_uses_numeric_wire_values() {
    let document = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI value");
    let schema = &document["components"]["schemas"]["ErrorCode"];
    let values = schema["enum"]
        .as_array()
        .expect("ErrorCode schema should contain enum values");
    let actual = values
        .iter()
        .filter_map(serde_json::Value::as_i64)
        .collect::<Vec<_>>();

    assert_eq!(schema["type"], "integer");
    assert_eq!(actual.first(), Some(&0));
    assert!(actual.contains(&1000));
    assert!(actual.contains(&6002));
    assert_eq!(
        actual.len(),
        values.len(),
        "every ErrorCode must be numeric"
    );

    let names = schema["x-enum-varnames"]
        .as_array()
        .expect("ErrorCode schema should expose variant names for TypeScript enums");
    assert_eq!(names.len(), values.len());
    assert_eq!(names.first(), Some(&serde_json::json!("Success")));
    assert!(names.contains(&serde_json::json!("BadRequest")));
}

#[test]
fn frontend_schema_components_are_complete_and_unique() {
    let document = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI value");
    let schemas = document["components"]["schemas"]
        .as_object()
        .expect("OpenAPI schemas should be an object");
    let required = [
        "AnalyticsQuery",
        "ConfigSchema",
        "ErrorCode",
        "ImportMode",
        "LinkResponse",
        "PostNewLink",
        "ValueType",
    ];
    let mut seen = HashSet::new();

    for name in required {
        assert!(schemas.contains_key(name), "missing frontend schema {name}");
        assert!(seen.insert(name), "duplicate schema assertion {name}");
    }
}

#[test]
fn management_api_paths_and_operations_are_registered() {
    let document = serde_json::to_value(ApiDoc::openapi()).expect("serialize OpenAPI value");
    let paths = document["paths"]
        .as_object()
        .expect("OpenAPI paths should be an object");
    let required_paths = [
        "/admin/v1/auth/login",
        "/admin/v1/links",
        "/admin/v1/links/{code}",
        "/admin/v1/links/{code}/analytics",
        "/admin/v1/analytics/trends",
        "/admin/v1/config/schema",
    ];

    for path in required_paths {
        assert!(
            paths.contains_key(path),
            "missing management API path {path}"
        );
    }

    let methods = ["get", "post", "put", "delete"];
    let operation_ids = paths
        .values()
        .filter_map(serde_json::Value::as_object)
        .flat_map(|path| methods.iter().filter_map(move |method| path.get(*method)))
        .filter_map(|operation| operation["operationId"].as_str())
        .collect::<HashSet<_>>();

    assert_eq!(operation_ids.len(), 31);
    assert!(operation_ids.contains("list_links"));
    assert!(operation_ids.contains("get_config_schema"));
    assert!(operation_ids.contains("export_analytics_report"));
}
