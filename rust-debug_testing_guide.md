# MCP 工具调试与测试完整指南

## 📋 目录
1. [MCP Inspector 调试工具](#1-mcp-inspector-调试工具)
2. [单元测试最佳实践](#2-单元测试最佳实践)
3. [集成测试方案](#3-集成测试方案)
4. [日志系统与错误处理](#4-日志系统与错误处理)
5. [常见问题排查](#5-常见问题排查)
6. [调试技巧与工具](#6-调试技巧与工具)

---

## 1. MCP Inspector 调试工具

### 1.1 什么是 MCP Inspector？
MCP Inspector 是官方提供的 Web 界面调试工具，用于可视化和测试 MCP 服务器的功能。

**GitHub 地址**: https://github.com/modelcontextprotocol/inspector

### 1.2 安装与使用

```bash
# 全局安装 Inspector
npm install -g @modelcontextprotocol/inspector

# 或使用 npx（推荐，无需全局安装）
npx @modelcontextprotocol/inspector
```

### 1.3 测试不同传输方式的服务器

#### STDIO 传输（推荐用于开发）
```bash
# 在 examples/servers 目录运行
cargo run --example counter_stdio

# 在另一个终端启动 Inspector
npx @modelcontextprotocol/inspector

# 或在 Rust 代码中直接提示
/// npx @modelcontextprotocol/inspector cargo run -p mcp-server-examples --example std_io
```

#### SSE 传输
```bash
# 运行 SSE 示例服务器
cargo run --example servers_counter_sse

# 浏览器访问提示的 URL
# 通常是 http://127.0.0.1:8000/sse
```

#### Streamable HTTP 传输
```bash
# 运行 Streamable HTTP 示例
cargo run --example counter_streamhttp

# 浏览器访问
# http://127.0.0.1:8001/mcp
```

### 1.4 Inspector 功能特性

**✅ 可视化功能**:
- 查看工具列表和参数模式
- 测试工具调用并查看响应
- 监控进度通知
- 查看资源列表和内容
- 测试提示模板

**✅ 调试信息**:
- 协议消息跟踪
- 错误详情显示
- 性能监控

**✅ 传输测试**:
- STDIO（标准输入/输出）
- SSE（Server-Sent Events）
- Streamable HTTP
- WebSocket（部分实现）

### 1.5 常用调试命令

```bash
# 启动 Inspector 并连接到 STDIO 服务器
npx @modelcontextprotocol/inspector cargo run --example counter_stdio

# 启动 Inspector 并连接到自定义服务器
npx @modelcontextprotocol/inspector --stdio ./target/debug/my-mcp-server

# 连接 HTTP 服务器
npx @modelcontextprotocol/inspector --http http://127.0.0.1:8000/mcp

# 连接 SSE 服务器
npx @modelcontextprotocol/inspector --sse http://127.0.0.1:8000/sse
```

---

## 2. 单元测试最佳实践

### 2.1 测试结构

MCP Rust SDK 提供了一套完整的测试工具，位于 `examples/servers/src/common/` 中。

**基础测试结构** (`counter.rs` 示范):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_execution() {
        // 测试工具执行逻辑
        let counter = Counter::new();
        let result = counter.increment().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prompt_attributes() {
        // 测试宏生成的属性
        let attr = Counter::example_prompt_prompt_attr();
        assert_eq!(attr.name, "example_prompt");
        assert!(attr.description.is_some());
    }

    #[tokio::test]
    async fn test_router_routes() {
        // 测试路由器路由
        let router = Counter::prompt_router();
        assert!(router.has_route("example_prompt"));
        assert!(router.has_route("counter_analysis"));
    }
}
```

### 2.2 工具测试模板

```rust
// 1. 定义测试结构
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TestRequest {
    pub param1: String,
    pub param2: Option<i32>,
}

// 2. 工具实现
#[tool]
async fn test_tool(
    &self,
    Parameters(params): Parameters<TestRequest>,
) -> Result<CallToolResult, McpError> {
    // 业务逻辑
    Ok(CallToolResult::success(vec![Content::text("success")]))
}

// 3. 测试用例
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_with_valid_input() {
        let server = MyServer::new();

        let result = server.test_tool(Parameters(TestRequest {
            param1: "test".to_string(),
            param2: Some(42),
        })).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_tool_with_invalid_input() {
        let server = MyServer::new();

        // 测试错误情况
        let result = server.test_tool(Parameters(TestRequest {
            param1: "".to_string(),
            param2: None,
        })).await;

        assert!(result.is_err());
    }
}
```

### 2.3 提示测试模板

```rust
#[tokio::test]
async fn test_prompt_execution() {
    let counter = Counter::new();

    // 创建提示上下文
    let context = rmcp::handler::server::prompt::PromptContext::new(
        &counter,
        "example_prompt".to_string(),
        Some({
            let mut map = serde_json::Map::new();
            map.insert(
                "message".to_string(),
                serde_json::Value::String("Test message".to_string()),
            );
            map
        }),
        RequestContext {
            meta: Default::default(),
            ct: tokio_util::sync::CancellationToken::new(),
            id: rmcp::model::NumberOrString::String("test-1".to_string()),
            peer: Default::default(),
            extensions: Default::default(),
        },
    );

    // 执行提示
    let router = Counter::prompt_router();
    let result = router.get_prompt(context).await;

    assert!(result.is_ok());
    let prompt_result = result.unwrap();
    assert_eq!(prompt_result.messages.len(), 1);
}
```

### 2.4 资源测试模板

```rust
#[tokio::test]
async fn test_read_resource() {
    let counter = Counter::new();

    let result = counter.read_resource(
        ReadResourceRequestParam {
            uri: "memo://insights".to_string(),
        },
        RequestContext::default(),
    ).await;

    assert!(result.is_ok());
    let resource_result = result.unwrap();
    assert_eq!(resource_result.contents.len(), 1);
}
```

---

## 3. 集成测试方案

### 3.1 JavaScript 集成测试

SDK 提供了完整的 JS 集成测试示例 (`test_with_js.rs`):

```rust
#[tokio::test]
async fn test_with_js_client() -> anyhow::Result<()> {
    // 初始化日志
    let _ = tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".to_string().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    // 安装依赖
    tokio::process::Command::new("npm")
        .arg("install")
        .current_dir("tests/test_with_js")
        .spawn()?
        .wait()
        .await?;

    // 启动 MCP 服务器
    let ct = SseServer::serve(SSE_BIND_ADDRESS.parse()?)
        .await?
        .with_service(Calculator::default);

    // 运行 JavaScript 客户端测试
    let exit_status = tokio::process::Command::new("node")
        .arg("tests/test_with_js/client.js")
        .spawn()?
        .wait()
        .await?;

    assert!(exit_status.success());
    ct.cancel();
    Ok(())
}
```

### 3.2 JavaScript 客户端示例 (`tests/test_with_js/client.js`)

```javascript
const { Client } = require('@modelcontextprotocol/sdk/client/index.js');
const { StdioClientTransport } = require('@modelcontextprotocol/sdk/client/stdio.js');

// 创建客户端
const client = new Client({
    name: "test-client",
    version: "1.0.0",
}, {
    capabilities: {
        tools: {},
    },
});

// 连接到 STDIO 传输
const transport = new StdioClientTransport({
    command: 'cargo',
    args: ['run', '--example', 'counter_stdio'],
});

await client.connect(transport);

// 测试工具调用
const result = await client.callTool({
    name: 'increment',
    arguments: {},
});

console.log('Tool result:', result);

// 测试列出工具
const tools = await client.listTools();
console.log('Tools:', tools);

// 关闭连接
await client.close();
```

### 3.3 Python 集成测试

同样支持 Python 集成测试（参考 `test_with_python.py` 的结构）:

```python
from mcp.client import Client
from mcp.client.stdio import StdioClientTransport

async def test_mcp_server():
    transport = StdioClientTransport(
        command='cargo',
        args=['run', '--example', 'counter_stdio']
    )

    async with Client('test-client', '1.0.0', transport=transport) as client:
        # 测试工具列表
        tools = await client.list_tools()
        print(f"Available tools: {tools}")

        # 测试工具调用
        result = await client.call_tool(
            name='increment',
            arguments={}
        )
        print(f"Tool result: {result}")

        # 测试提示列表
        prompts = await client.list_prompts()
        print(f"Available prompts: {prompts}")

        # 测试获取提示
        prompt = await client.get_prompt(
            name='example_prompt',
            arguments={'message': 'Hello from Python'}
        )
        print(f"Prompt: {prompt}")
```

### 3.4 HTTP Streamable 测试

```rust
#[tokio::test]
async fn test_streamable_http() -> anyhow::Result<()> {
    // 启动 Streamable HTTP 服务器
    let service: StreamableHttpService<Calculator, LocalSessionManager> =
        StreamableHttpService::new(
            || Ok(Calculator::new()),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
            },
        );

    let router = axum::Router::new().nest_service("/mcp", service);
    let tcp_listener = tokio::net::TcpListener::bind(STREAMABLE_HTTP_BIND_ADDRESS).await?;

    // 运行服务器
    let ct = CancellationToken::new();
    let handle = tokio::spawn({
        let ct = ct.clone();
        async move {
            let _ = axum::serve(tcp_listener, router)
                .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                .await;
        }
    });

    // 运行客户端测试
    let exit_status = tokio::process::Command::new("node")
        .arg("tests/test_with_js/streamable_client.js")
        .spawn()?
        .wait()
        .await?;

    assert!(exit_status.success());
    ct.cancel();
    handle.await?;
    Ok(())
}
```

---

## 4. 日志系统与错误处理

### 4.1 日志配置

所有示例都使用 `tracing` 和 `tracing-subscriber`:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志系统
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::DEBUG.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    tracing::info!("Starting MCP server");
    // ... 服务器代码
}
```

### 4.2 不同环境的日志级别

```rust
// 开发环境 - 详细日志
tracing_subscriber::fmt()
    .with_env_filter("debug")
    .init();

// 生产环境 - 警告和错误
tracing_subscriber::fmt()
    .with_env_filter("warn")
    .init();

// 通过环境变量控制
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

### 4.3 结构化日志记录

```rust
// 基本日志
tracing::info!("Server started");
tracing::debug!("Processing request");
tracing::warn!("Invalid parameter");
tracing::error!("Error: {}", err);

// 结构化日志（推荐）
tracing::info!(
    method = %request.method,
    path = %request.path,
    "Handling request"
);

tracing::error!(
    error = %err,
    request_id = %request_id,
    "Operation failed"
);
```

### 4.4 错误处理模式

#### 工具错误处理

```rust
#[tool]
async fn my_tool(&self, params: Parameters<MyParams>) -> Result<CallToolResult, McpError> {
    // 验证参数
    if params.0.value.is_empty() {
        return Err(McpError::invalid_params(
            "Value cannot be empty",
            Some(json!({
                "field": "value",
                "reason": "empty"
            })),
        ));
    }

    // 处理业务逻辑
    let result = process_data(&params.0).map_err(|e| {
        McpError::internal_error(
            format!("Failed to process data: {}", e),
            Some(json!({
                "original_error": e.to_string()
            })),
        )
    })?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

#### 提示错误处理

```rust
#[prompt]
async fn my_prompt(
    &self,
    Parameters(args): Parameters<MyPromptArgs>,
) -> Result<GetPromptResult, McpError> {
    if args.language.is_empty() {
        return Err(McpError::invalid_params(
            "Language cannot be empty",
            None,
        ));
    }

    tracing::info!(
        prompt_name = "my_prompt",
        language = %args.language,
        "Generating prompt"
    );

    // 生成提示逻辑
    Ok(GetPromptResult {
        description: Some(format!("Prompt for {}", args.language)),
        messages: vec![/* ... */],
    })
}
```

#### 资源错误处理

```rust
async fn read_resource(
    &self,
    ReadResourceRequestParam { uri }: ReadResourceRequestParam,
    _: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    match uri.as_str() {
        "resource://valid" => {
            // 返回资源内容
            Ok(ReadResourceResult {
                contents: vec![ResourceContents::text(content, uri)],
            })
        }
        _ => Err(McpError::resource_not_found(
            "Resource not found",
            Some(json!({
                "uri": uri
            })),
        )),
    }
}
```

### 4.5 进度通知

对于长时间运行的操作，使用进度通知:

```rust
use tokio_stream::StreamExt;

#[tool]
async fn long_running_task(&self, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
    let total = 100;

    for i in 0..total {
        // 执行部分工作

        // 发送进度通知
        ctx.peer.notify_progress(ProgressNotificationParam {
            progress_token: ProgressToken(NumberOrString::Number(1)),
            progress: i as f64,
            total: Some(total as f64),
            Some(format!("Processing item {}", i)),
        }).await.map_err(|e| {
            McpError::internal_error(
                format!("Failed to notify progress: {}", e),
                None,
            )
        })?;

        // 等待一小段时间
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    Ok(CallToolResult::success(vec![Content::text("Task completed")]))
}
```

---

## 5. 常见问题排查

### 5.1 连接问题

**症状**: 无法连接到 MCP 服务器

**排查步骤**:
1. 检查服务器是否正在运行
2. 验证传输方式（STDIO/SSE/HTTP）
3. 检查端口是否被占用
4. 验证防火墙设置

**调试命令**:
```bash
# 检查端口占用
netstat -an | grep 8000
lsof -i :8000

# 测试 HTTP 连接
curl http://127.0.0.1:8000/mcp

# 测试 STDIO 连接
echo '{}' | ./target/debug/your-mcp-server
```

### 5.2 工具调用失败

**症状**: 工具调用返回错误

**排查步骤**:
1. 使用 Inspector 查看详细错误信息
2. 检查参数是否正确
3. 查看服务器日志
4. 验证工具实现

**调试代码**:
```rust
// 在工具中添加详细日志
#[tool]
async fn my_tool(&self, params: Parameters<MyParams>) -> Result<CallToolResult, McpError> {
    tracing::debug!(
        params = ?params.0,
        "Tool called"
    );

    // 业务逻辑

    tracing::info!("Tool completed successfully");
    Ok(CallToolResult::success(vec![Content::text("OK")]))
}
```

### 5.3 提示执行错误

**症状**: 提示无法获取或执行失败

**排查步骤**:
1. 检查提示参数模式
2. 验证必需参数
3. 查看提示路由器配置

**测试命令**:
```bash
# 使用 Inspector 测试提示
npx @modelcontextprotocol/inspector cargo run --example prompt_stdio
```

### 5.4 资源访问问题

**症状**: 无法列出或读取资源

**排查步骤**:
1. 验证 URI 格式
2. 检查资源列表实现
3. 确认资源内容可读性

### 5.5 性能问题

**症状**: 响应慢或超时

**排查方法**:
1. 使用日志记录执行时间
2. 检查异步操作是否正确
3. 优化长时间运行的操作

**性能监控代码**:
```rust
use std::time::Instant;

#[tool]
async fn my_tool(&self) -> Result<CallToolResult, McpError> {
    let start = Instant::now();

    // 业务逻辑
    let result = expensive_operation().await?;

    let duration = start.elapsed();
    tracing::info!(
        operation = "my_tool",
        duration_ms = duration.as_millis(),
        "Tool execution completed"
    );

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

---

## 6. 调试技巧与工具

### 6.1 代码调试

#### 启用详细日志
```bash
# 通过环境变量设置日志级别
export RUST_LOG=debug
cargo run

# 或在代码中设置
tracing_subscriber::fmt()
    .with_env_filter("debug")
    .init();
```

#### 使用 Inspector 调试
```bash
# 启动 Inspector 并连接到服务器
npx @modelcontextprotocol/inspector cargo run --example counter_stdio
```

#### 手动测试 STDIO
```bash
# 创建测试 JSON
cat > test_request.json << 'EOF'
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/list",
  "params": {}
}
EOF

# 发送到服务器
cat test_request.json | ./target/debug/your-mcp-server
```

### 6.2 协议调试

#### 启用消息跟踪
```rust
let service = server.serve(stdio()).await.inspect_err(|e| {
    tracing::error!("serving error: {:?}", e);
})?;
```

#### 查看协议版本
```bash
# 发送 initialize 请求
curl -X POST http://127.0.0.1:8000/mcp \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }'
```

### 6.3 网络调试

#### 使用 Wireshark
```bash
# 监控本地 HTTP 流量
sudo wireshark -i lo -f "port 8000"
```

#### 使用 netcat 测试 SSE
```bash
# 测试 SSE 端点
curl -N http://127.0.0.1:8000/sse
```

### 6.4 内存和性能分析

#### 使用 perf
```bash
# 性能分析
cargo install cargo-perf
cargo perf record --bin your-mcp-server
cargo perf report
```

#### 使用 valgrind
```bash
# 内存检查
valgrind --tool=memcheck ./target/debug/your-mcp-server
```

### 6.5 常见调试模式

#### 1. 逐步调试
```rust
#[tool]
async fn my_tool(&self, params: Parameters<MyParams>) -> Result<CallToolResult, McpError> {
    tracing::debug!("Step 1: Received params");

    // 验证
    if !validate(&params.0) {
        tracing::warn!("Step 1 failed: validation error");
        return Err(McpError::invalid_params("Invalid params", None));
    }

    tracing::debug!("Step 2: Validation passed");

    // 处理
    let result = process(&params.0).await.map_err(|e| {
        tracing::error!("Step 2 failed: {}", e);
        McpError::internal_error(e.to_string(), None)
    })?;

    tracing::debug!("Step 3: Processing completed");

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

#### 2. 条件调试
```rust
#[tool]
async fn debug_tool(&self) -> Result<CallToolResult, McpError> {
    if std::env::var("DEBUG_MODE").is_ok() {
        tracing::info!("Debug mode enabled");

        // 输出内部状态
        tracing::debug!("Internal state: {:?}", self.internal_state);
    }

    // 正常逻辑
}
```

### 6.6 测试驱动开发

#### 创建测试优先
```rust
// 1. 编写测试
#[tokio::test]
async fn test_new_feature() {
    let server = MyServer::new();
    let result = server.new_tool().await;
    assert!(result.is_ok());
}

// 2. 运行测试
cargo test test_new_feature

// 3. 实现功能
#[tool]
async fn new_tool(&self) -> Result<CallToolResult, McpError> {
    // 实现
}
```

### 6.7 调试清单

**部署前检查**:
- [ ] 所有单元测试通过
- [ ] 集成测试通过
- [ ] Inspector 测试通过
- [ ] 日志级别正确设置
- [ ] 错误处理完整
- [ ] 性能符合要求
- [ ] 文档更新

**常见错误检查**:
- [ ] 异步函数正确使用 `.await`
- [ ] 错误类型匹配
- [ ] 路径验证
- [ ] 参数验证
- [ ] 超时处理

---

## 7. 参考资料

### 官方资源
- **MCP Inspector**: https://github.com/modelcontextprotocol/inspector
- **MCP 规范**: https://spec.modelcontextprotocol.io/
- **Rust SDK**: https://github.com/modelcontextprotocol/rust-sdk

### 示例代码
- `examples/servers/src/counter_stdio.rs` - 基础 STDIO 示例
- `examples/servers/src/prompt_stdio.rs` - 提示示例
- `examples/servers/src/structured_output.rs` - 结构化输出示例
- `examples/servers/src/progress_demo.rs` - 进度通知示例

### 测试资源
- `examples/servers/src/common/` - 通用测试组件
- `crates/rmcp/tests/` - SDK 测试示例

### 社区资源
- **GitHub Issues**: 报告问题和建议
- **Discord**: 实时讨论和帮助
- **博客文章**: 最佳实践和案例研究

---

## 总结

本指南涵盖了 MCP 工具调试和测试的各个方面：

1. **MCP Inspector** 是最重要的调试工具，必须熟练使用
2. **单元测试** 确保代码质量，每个工具都应该有测试
3. **集成测试** 验证端到端功能
4. **日志系统** 提供调试信息的关键
5. **错误处理** 提升用户体验
6. **调试技巧** 提高开发效率

通过遵循这些最佳实践，你可以高效地开发和调试 MCP 工具！🚀
