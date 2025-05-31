# shortlinker 测试套件

这个目录包含了 shortlinker 项目的完整测试套件。

## 测试文件说明

### integration_tests.rs

- 集成测试，测试完整的 HTTP 请求处理流程
- 测试短链接重定向功能
- 测试管理 API 的认证和权限控制

### utils_tests.rs

- 工具函数的单元测试
- 主要测试随机代码生成功能
- 验证代码长度、字符集、唯一性

### storage_tests.rs

- 存储层的单元测试
- 测试文件存储的增删改查操作
- 验证数据持久化和一致性

### cli_tests.rs

- 命令行界面的测试
- 测试各种 CLI 命令和参数
- 验证错误处理和用户交互

### performance_tests.rs

- 性能基准测试
- 测试代码生成速度和唯一性
- 并发性能和内存使用测试

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定测试文件
cargo test --test integration_tests
cargo test --test utils_tests
cargo test --test storage_tests
cargo test --test cli_tests
cargo test --test performance_tests

# 运行带输出的测试
cargo test -- --nocapture

# 运行性能测试并显示输出
cargo test --test performance_tests -- --nocapture
```

## 测试环境要求

- Rust 1.82+
- 临时文件权限（用于测试文件存储）
- 网络权限（用于HTTP测试）

## 注意事项

- 所有测试都使用临时文件和目录，不会影响实际数据
- CLI 测试需要项目能够正常编译
- 性能测试可能需要较长时间运行
