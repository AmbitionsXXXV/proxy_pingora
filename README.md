# proxy_pingora

proxy_pingora 是一个基于 Rust 的高性能用户管理 API 服务示例，集成了 axum、dashmap、argon2、tokio、serde 等现代 Rust 生态库，支持用户的增删改查、密码加密存储、健康检查等功能，适合用作学习和二次开发的基础模板。

## 特性

1. 用户注册、查询、更新、删除接口
2. 密码安全存储（Argon2 加密）
3. 并发安全的内存存储（DashMap）
4. 健康检查接口
5. 现代 Rust 异步与 web 框架实践

## 快速开始

### 运行示例服务

```bash
cargo run --example server
```

服务启动后，监听 `127.0.0.1:3000`，可通过如下接口进行操作：

- `GET    /users`         获取所有用户
- `GET    /users/:id`     获取指定用户
- `POST   /users`         创建用户
- `PUT    /users/:id`     更新用户
- `DELETE /users/:id`     删除用户
- `GET    /health`        健康检查

### 创建用户示例

```bash
curl -X POST http://127.0.0.1:3000/users \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"123456","name":"测试用户"}'
```

### 依赖

- Rust 2024 edition
- axum
- dashmap
- argon2
- tokio
- serde
- serde_json
- chrono
- rand
- anyhow
- tracing
- tracing-subscriber

## 许可证

本项目基于 MIT 协议开源。

详见 [LICENSE](LICENSE.md) 。

Copyright 2025 Etcetera <etetera3636@gmail.com>
