#!/bin/bash

# 本地 benchmark 脚本
# 用法: ./benchmark_local.sh [shortlink_code]

set -e

CODE="${1:-esap}"
HOST="http://127.0.0.1:8080"
DURATION="30s"
THREADS=4
CONNECTIONS=50

echo "=== Shortlinker 本地 Benchmark ==="
echo "目标: $HOST/$CODE"
echo "线程: $THREADS, 连接: $CONNECTIONS, 时长: $DURATION"
echo ""

# 检查 wrk 是否安装
if ! command -v wrk &> /dev/null; then
    echo "错误: wrk 未安装"
    echo "macOS: brew install wrk"
    exit 1
fi

# 检查服务是否运行
if ! curl -s "$HOST/health/live" > /dev/null 2>&1; then
    echo "错误: shortlinker 服务未运行"
    echo "请先启动: cargo run --release"
    exit 1
fi

echo "--- 预热 (5s) ---"
wrk -t2 -c20 -d5s "$HOST/$CODE" > /dev/null 2>&1

echo ""
echo "--- 测试存在的短链: /$CODE ---"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION --latency "$HOST/$CODE"

echo ""
echo "--- 测试不存在的短链: /nonexistent ---"
wrk -t$THREADS -c$CONNECTIONS -d$DURATION --latency "$HOST/nonexistent"

echo ""
echo "--- 随机路径压测 ---"
# 简单的随机路径测试（用 lua 脚本）
cat > /tmp/random_paths.lua << 'EOF'
local charset = "abcdefghijklmnopqrstuvwxyz0123456789"
local function random_string(length)
    local buf = {}
    for i = 1, length do
        local idx = math.random(1, #charset)
        buf[i] = charset:sub(idx, idx)
    end
    return table.concat(buf)
end

request = function()
    local path = "/" .. random_string(math.random(3, 8))
    return wrk.format("GET", path)
end
EOF

wrk -t$THREADS -c$CONNECTIONS -d$DURATION --latency -s /tmp/random_paths.lua "$HOST"

echo ""
echo "=== 测试完成 ==="
