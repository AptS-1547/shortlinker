# 多阶段构建 - 构建阶段
FROM rust:1.87-slim AS builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    ca-certificates \
    musl-tools \
    && rm -rf /var/lib/apt/lists/*

# 添加 musl 目标
RUN rustup target add x86_64-unknown-linux-musl

# 设置工作目录
WORKDIR /app

# 复制源代码
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 静态链接编译 - 使用 musl 目标
ENV RUSTFLAGS="-C link-arg=-s -C opt-level=z"
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# 运行阶段 - 使用scratch
FROM scratch

LABEL maintainer="AptS:1547 <apts-1547@esaps.net>"
LABEL description="Shortlinker is a simple, fast, and secure URL shortener written in Rust."
LABEL version="1.0.0"
LABEL homepage="https://github.com/AptS-1547/shortlinker"
LABEL license="MIT"

# 从构建阶段复制二进制文件 (使用 musl 目标路径)
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/shortlinker /shortlinker

VOLUME ["/data"]

# 暴露端口
EXPOSE 8080

# 设置环境变量
ENV DOCKER_ENV=1
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080
ENV DB_FILE_NAME=/data/shortlinker.data
ENV RUST_LOG=info

# 启动命令
ENTRYPOINT ["/shortlinker"]