# OpenCode 代理使用说明

## 项目概述
这是一个基于 Rust 的 Web 服务，使用 Axum 框架，SQLite 数据库和 Redis 身份验证。

## 关键命令
- `cargo run` - 启动服务器（默认端口 4027）
- `cargo test` - 运行测试（如果有）
- `cargo check` - 检查代码而不构建
- `cargo build` - 构建项目

## 架构
- 入口点：`src/main.rs`
- API 路由在 `src/main.rs` 中定义，处理程序位于 `src/handler/` 中
- 通过 Redis 会话存储进行身份验证
- 从 `config.toml` 加载配置
- 数据库：SQLite（`zscm.db`）

## 重要约束
- 需要在本地主机（127.0.0.1）上运行 Redis 服务器用于身份验证
- 数据库文件 `zscm.db` 必须存在且可访问
- 服务器默认监听端口 4027（在 `config.toml` 中配置）
- 所有 API 端点都需要通过 `account` 和 `sessionid` 头进行身份验证

## 测试
- 未找到显式的测试运行器配置
- 测试可能在 `tests/` 目录中（如果存在）
- 单独测试执行可能需要使用 `cargo test [test_name]`

## 框架细节
- 使用 Axum Web 框架处理 HTTP
- 使用 sqlx 访问 SQLite 数据库
- 使用 Redis 进行会话管理
- 使用 Tokio 异步运行时
- 通过 TOML 文件进行配置