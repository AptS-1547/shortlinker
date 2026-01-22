//! IPC 协议序列化性能基准测试

use bytes::BytesMut;
use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use shortlinker::system::ipc::protocol::{decode, encode};
use shortlinker::system::ipc::types::{IpcCommand, IpcResponse, ShortLinkData};
use shortlinker::system::reload::ReloadTarget;

fn create_test_commands() -> Vec<IpcCommand> {
    vec![
        IpcCommand::Ping,
        IpcCommand::GetStatus,
        IpcCommand::Shutdown,
        IpcCommand::Reload {
            target: ReloadTarget::All,
        },
        IpcCommand::AddLink {
            code: Some("test123".to_string()),
            target: "https://example.com/very/long/path/to/destination".to_string(),
            force: true,
            expires_at: Some("2025-12-31T23:59:59Z".to_string()),
            password: Some("secret".to_string()),
        },
        IpcCommand::ListLinks {
            page: 1,
            page_size: 50,
            search: Some("example".to_string()),
        },
    ]
}

fn create_test_responses() -> Vec<IpcResponse> {
    vec![
        IpcResponse::Pong {
            version: "0.4.0".to_string(),
            uptime_secs: 86400,
        },
        IpcResponse::Status {
            version: "0.4.0".to_string(),
            uptime_secs: 86400,
            is_reloading: false,
            last_data_reload: Some("2025-01-01T00:00:00Z".to_string()),
            last_config_reload: None,
            links_count: 10000,
        },
        IpcResponse::Error {
            code: "E001".to_string(),
            message: "Something went wrong with a detailed error message".to_string(),
        },
        IpcResponse::LinkList {
            links: (0..10)
                .map(|i| ShortLinkData {
                    code: format!("code_{}", i),
                    target: format!("https://example.com/{}", i),
                    created_at: "2025-01-01T00:00:00Z".to_string(),
                    expires_at: Some("2025-12-31T23:59:59Z".to_string()),
                    password: None,
                    click: i * 100,
                })
                .collect(),
            total: 1000,
            page: 1,
            page_size: 10,
        },
    ]
}

/// 命令编码性能
fn bench_encode_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc/encode");

    let commands = create_test_commands();
    for (i, cmd) in commands.iter().enumerate() {
        let name = match cmd {
            IpcCommand::Ping => "ping",
            IpcCommand::GetStatus => "get_status",
            IpcCommand::Shutdown => "shutdown",
            IpcCommand::Reload { .. } => "reload",
            IpcCommand::AddLink { .. } => "add_link",
            IpcCommand::ListLinks { .. } => "list_links",
            _ => continue,
        };

        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("command_{}", name), |b| {
            b.iter(|| {
                let _ = encode(&commands[i]);
            });
        });
    }

    group.finish();
}

/// 响应编码性能
fn bench_encode_responses(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc/encode");

    let responses = create_test_responses();
    let names = ["pong", "status", "error", "link_list"];

    for (i, name) in names.iter().enumerate() {
        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("response_{}", name), |b| {
            b.iter(|| {
                let _ = encode(&responses[i]);
            });
        });
    }

    group.finish();
}

/// 命令解码性能
fn bench_decode_commands(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc/decode");

    let commands = create_test_commands();
    let encoded: Vec<Vec<u8>> = commands.iter().map(|cmd| encode(cmd).unwrap()).collect();
    let names = [
        "ping",
        "get_status",
        "shutdown",
        "reload",
        "add_link",
        "list_links",
    ];

    for (i, name) in names.iter().enumerate() {
        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("command_{}", name), |b| {
            b.iter(|| {
                let mut buf = BytesMut::from(&encoded[i][..]);
                let _: Option<IpcCommand> = decode(&mut buf).unwrap();
            });
        });
    }

    group.finish();
}

/// 响应解码性能
fn bench_decode_responses(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc/decode");

    let responses = create_test_responses();
    let encoded: Vec<Vec<u8>> = responses.iter().map(|resp| encode(resp).unwrap()).collect();
    let names = ["pong", "status", "error", "link_list"];

    for (i, name) in names.iter().enumerate() {
        group.throughput(Throughput::Elements(1));
        group.bench_function(format!("response_{}", name), |b| {
            b.iter(|| {
                let mut buf = BytesMut::from(&encoded[i][..]);
                let _: Option<IpcResponse> = decode(&mut buf).unwrap();
            });
        });
    }

    group.finish();
}

/// 批量编解码性能
fn bench_batch_roundtrip(c: &mut Criterion) {
    let mut group = c.benchmark_group("ipc/batch");

    // 模拟批量 link list 响应
    for num_links in [10, 100, 500] {
        let response = IpcResponse::LinkList {
            links: (0..num_links)
                .map(|i| ShortLinkData {
                    code: format!("code_{}", i),
                    target: format!("https://example.com/path/{}", i),
                    created_at: "2025-01-01T00:00:00Z".to_string(),
                    expires_at: Some("2025-12-31T23:59:59Z".to_string()),
                    password: None,
                    click: i * 10,
                })
                .collect(),
            total: num_links as usize,
            page: 1,
            page_size: num_links as u64,
        };

        group.throughput(Throughput::Elements(num_links as u64));
        group.bench_function(format!("roundtrip_{}_links", num_links), |b| {
            b.iter(|| {
                let encoded = encode(&response).unwrap();
                let mut buf = BytesMut::from(&encoded[..]);
                let _: Option<IpcResponse> = decode(&mut buf).unwrap();
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_encode_commands,
    bench_encode_responses,
    bench_decode_commands,
    bench_decode_responses,
    bench_batch_roundtrip,
);
criterion_main!(benches);
