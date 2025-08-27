# 前端构建阶段
FROM node:22-alpine AS frontend-builder

RUN apk add git
RUN corepack enable && \
    corepack prepare yarn@latest --activate

COPY ./.git /app/.git

WORKDIR /app/admin-panel

# 复制前端依赖文件
COPY ./admin-panel /app/admin-panel
RUN yarn install --frozen-lockfile
RUN yarn build:prod

# 多阶段构建 - 构建阶段
FROM rust:1.89-slim AS builder

# 安装构建依赖，包含完整的 OpenSSL 开发库
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libssl3 \
    openssl \
    ca-certificates \
    musl-tools \
    musl-dev \
    && rm -rf /var/lib/apt/lists/*

# 添加 musl 目标
RUN rustup target add x86_64-unknown-linux-musl

# 设置工作目录
WORKDIR /app

# 复制源代码
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# 从前端构建阶段复制构建产物
COPY --from=frontend-builder /app/admin-panel/dist ./admin-panel/dist

# 设置 OpenSSL 环境变量和编译选项
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr
ENV RUSTFLAGS="-C link-arg=-s -C opt-level=z -C target-feature=+crt-static"

# 静态链接编译 - 使用 musl 目标
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

# 运行阶段 - 使用scratch
FROM scratch

LABEL maintainer="AptS:1547 <apts-1547@esaps.net>"
LABEL description="Shortlinker is a simple, fast, and secure URL shortener written in Rust."
LABEL version="0.2.0-alpha.2"
LABEL homepage="https://github.com/AptS-1547/shortlinker"
LABEL license="MIT"

# 从构建阶段复制二进制文件 (使用 musl 目标路径)
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/shortlinker /shortlinker

VOLUME ["/data", "/socket"]

# 暴露端口
EXPOSE 8080

# 设置环境变量
ENV DOCKER_ENV=1

# 启动命令
ENTRYPOINT ["/shortlinker"]