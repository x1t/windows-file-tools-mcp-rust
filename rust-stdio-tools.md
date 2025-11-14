# MCP Rust SDK - Stdio 工具注册官方指南

## 概述

本文档详细介绍了 MCP (Model Context Protocol) Rust SDK 中 stdio 工具注册的官方方式和最佳实践，基于 `rmcp` crate 的源码分析。

---

## 1. 核心架构

### 1.1 Stdio Transport 实现

**位置**: `crates/rmcp/src/transport/io.rs`

```rust
/// 创建标准输入输出的异步句柄对
/// 返回 (Stdin, Stdout) 用于 MCP 通信
pub fn stdio() -> (tokio::io::Stdin, tokio::io::Stdout) {
    (tokio::io::stdin(), tokio::io::stdout())
}
```

**特点**:
- 基于 tokio 异步运行时
- 适用于命令行工具和桌面集成
- 支持通过 `IntoTransport` trait 自动转换

---

## 2. 工具注册标准流程

### 2.1 完整代码示例

```rust
use std::sync::Arc;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{
        router::tool::ToolRouter,
        wrapper::Parameters,
    },
    model::*,
    tool, tool_handler, tool_router,
};
use tokio::sync::Mutex;

/// 服务结构体 - 持有状态和路由器
#[derive(Clone)]
pub struct Counter {
    counter: Arc<Mutex<i32>>,
    tool_router: ToolRouter<Counter>,  // 路由器字段
}

/// 使用 #[tool_router] 宏自动生成工具路由
#[tool_router]
impl Counter {
    /// 创建新实例并初始化路由器
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),  // 🔥 初始化路由器
        }
    }

    /// 使用 #[tool] 标记为 MCP 工具
    #[tool(description = "增加计数器值")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "减少计数器值")]
    async fn decrement(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter -= 1;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    #[tool(description = "获取当前计数器值")]
    async fn get_value(&self) -> Result<CallToolResult, McpError> {
        let counter = self.counter.lock().await;
        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }
}

/// 实现服务器处理器
#[tool_handler]
impl ServerHandler for Counter {
    /// 返回服务器信息
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: None,
        }
    }

    /// 返回工具路由器
    fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }
}

/// main.rs - 服务器入口点
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // 关键调用：创建服务并启动 stdio 传输
    let service = Counter::new().serve(stdio()).await?;

    // 等待服务器关闭
    service.waiting().await?;
    Ok(())
}
```

---

## 3. 关键宏详解

### 3.1 #[tool] 宏

**位置**: `crates/rmcp-macros/src/tool.rs`

**作用**: 标记函数为 MCP 工具，自动生成工具元数据

**工作流程**:
1. 扫描函数上的 `#[tool]` 属性
2. 自动生成工具元数据函数 `{函数名}_tool_attr()`
3. 提取函数签名生成 JSON Schema
4. 处理异步/同步转换

**自动生成的代码示例**:

```rust
// 源码
#[tool(description = "描述")]
async fn increment(&self) -> Result<CallToolResult, McpError> {
    let mut counter = self.counter.lock().await;
    *counter += 1;
    Ok(CallToolResult::success(vec![Content::text(
        counter.to_string(),
    )]))
}

// 宏自动生成：
pub fn increment_tool_attr() -> rmcp::model::Tool {
    rmcp::model::Tool {
        name: "increment".into(),
        description: Some("描述".into()),
        input_schema: /* 根据参数自动生成 */,
        output_schema: /* 根据返回类型自动生成 */,
        annotations: None,
        icons: None,
        meta: None,
    }
}
```

### 3.2 #[tool_router] 宏

**位置**: `crates/rmcp-macros/src/tool_router.rs`

**作用**: 自动生成工具路由器注册代码

**核心逻辑**:

```rust
pub fn tool_router(attr: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let attr_args = NestedMeta::parse_meta_list(attr)?;
    let ToolRouterAttribute { router, vis } = ToolRouterAttribute::from_list(&attr_args)?;
    let mut item_impl = syn::parse2::<ItemImpl>(input.clone())?;

    // 🔍 扫描所有带 `#[rmcp::tool]` 标记的函数
    let tool_attr_fns: Vec<_> = item_impl
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(fn_item) = item {
                fn_item
                    .attrs
                    .iter()
                    .any(|attr| {
                        attr.path()
                            .segments
                            .last()
                            .is_some_and(|seg| seg.ident == "tool")
                    })
                    .then_some(&fn_item.sig.ident)
            } else {
                None
            }
        })
        .collect();

    // 🔗 为每个工具注册路由
    let mut routers = vec![];
    for handler in tool_attr_fns {
        let tool_attr_fn_ident = format_ident!("{handler}_tool_attr");
        routers.push(quote! {
            .with_route((Self::#tool_attr_fn_ident(), Self::#handler))
        })
    }

    // 🚀 生成 router() 函数
    let router_fn = syn::parse2::<ImplItem>(quote! {
        #vis fn #router() -> rmcp::handler::server::router::tool::ToolRouter<Self> {
            rmcp::handler::server::router::tool::ToolRouter::<Self>::new()
                #(#routers)*
        }
    })?;
    item_impl.items.push(router_fn);
    Ok(item_impl.into_token_stream())
}
```

**生成的路由函数示例**:

```rust
impl Counter {
    pub fn tool_router() -> ToolRouter<Self> {
        ToolRouter::<Self>::new()
            .with_route((Self::increment_tool_attr(), Self::increment))
            .with_route((Self::decrement_tool_attr(), Self::decrement))
            .with_route((Self::get_value_tool_attr(), Self::get_value))
    }
}
```

### 3.3 #[tool_handler] 宏

**作用**: 自动实现 ServerHandler trait

**关键方法**:
- `get_info()`: 返回服务器信息
- `get_tool_router()`: 返回工具路由器
- 其他生命周期方法

---

## 4. 参数处理与类型安全

### 4.1 Parameters 包装器

**位置**: `crates/rmcp/src/handler/server/wrapper.rs`

**使用方式**:

```rust
use rmcp::handler::server::wrapper::Parameters;

// 参数结构体定义
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CalculationRequest {
    pub a: i32,
    pub b: i32,
    pub operation: String,
}

#[tool(description = "计算两个数的和")]
fn sum(
    &self,
    // 🔥 使用 Parameters<T> 自动解析参数
    Parameters(StructRequest { a, b }): Parameters<StructRequest>,
) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        (a + b).to_string(),
    )]))
}

#[tool(description = "执行复杂计算")]
fn calculate(
    &self,
    Parameters(request): Parameters<CalculationRequest>,
) -> Result<CallToolResult, McpError> {
    let result = match request.operation.as_str() {
        "add" => request.a + request.b,
        "subtract" => request.a - request.b,
        "multiply" => request.a * request.b,
        "divide" => {
            if request.b == 0 {
                return Err(McpError::invalid_params("Cannot divide by zero"));
            }
            request.a / request.b
        },
        _ => return Err(McpError::invalid_params("Unknown operation")),
    };

    Ok(CallToolResult::success(vec![Content::text(
        format!("Result: {}", result),
    )]))
}
```

**宏处理逻辑**:
1. 扫描函数参数中的 `Parameters<T>`
2. 提取类型 `T`
3. 使用 `cached_schema_for_type::<T>()` 生成输入 schema
4. 在函数体中自动解包 `Parameters(request)`

### 4.2 结构化输出

```rust
use rmcp::Json;
use serde::{Serialize, Deserialize};

// 返回结构化数据（自动生成输出 schema）
#[derive(Serialize, Deserialize, JsonSchema)]
struct CalculationResult {
    result: i32,
    operation: String,
    operands: (i32, i32),
}

#[tool(name = "calculate", description = "执行计算并返回结构化结果")]
async fn calculate(
    &self,
    params: Parameters<CalculationRequest>,
) -> Result<Json<CalculationResult>, McpError> {
    let result = match params.0.operation.as_str() {
        "add" => params.0.a + params.0.b,
        "subtract" => params.0.a - params.0.b,
        "multiply" => params.0.a * params.0.b,
        "divide" => {
            if params.0.b == 0 {
                return Err(McpError::invalid_params("Cannot divide by zero"));
            }
            params.0.a / params.0.b
        },
        _ => return Err(McpError::invalid_params("Unknown operation")),
    };

    Ok(Json(CalculationResult {
        result,
        operation: params.0.operation,
        operands: (params.0.a, params.0.b),
    }))
}
```

---

## 5. 服务器初始化流程

### 5.1 完整初始化序列

**位置**: `crates/rmcp/src/service/server.rs`

```rust
pub async fn serve_server_with_ct_inner<S, T>(
    service: S,
    transport: T,
    ct: CancellationToken,
) -> Result<RunningService<RoleServer, S>, ServerInitializeError>
where
    S: Service<RoleServer>,
    T: Transport<RoleServer> + 'static,
{
    let mut transport = transport.into_transport();

    // 步骤1: 等待初始化请求
    let (request, id) = expect_request(&mut transport, "initialized request").await?;

    if !matches!(request, ClientRequest::InitializeRequest(_)) {
        return Err(ServerInitializeError::ExpectedInitializeRequest(Some(...)));
    }

    // 步骤2: 创建对等体
    let (peer, peer_rx) = Peer::new(id_provider, Some(peer_info.params.clone()));

    // 步骤3: 处理初始化
    let context = RequestContext { /* ... */ };
    let init_response = service.handle_request(request, context).await?;

    // 步骤4: 发送初始化响应
    transport.send(ServerJsonRpcMessage::response(
        ServerResult::InitializeResult(init_response),
        id,
    )).await?;

    // 步骤5: 等待初始化完成通知
    let notification = expect_notification(&mut transport, "initialize notification").await?;

    // 步骤6: 启动主服务循环
    Ok(serve_inner(service, transport, peer, peer_rx, ct))
}
```

### 5.2 简化启动模式

```rust
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // 🔥 关键调用：new().serve(stdio())
    let service = Counter::new().serve(stdio()).await?;

    // 等待服务器关闭
    service.waiting().await?;
    Ok(())
}
```

---

## 6. 完整项目结构

### 6.1 目录结构

```
my-mcp-server/
├── Cargo.toml
└── src/
    ├── main.rs           // 服务器入口
    ├── lib.rs            // 业务逻辑
    └── models/           // 数据模型（可选）
        └── requests.rs   // 请求结构体
```

### 6.2 Cargo.toml 配置

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
edition = "2021"

[dependencies]
# MCP 核心库
rmcp = { version = "0.8.5", features = ["server", "macros"] }

# 异步运行时
tokio = { version = "1.0", features = ["full"] }

# 日志系统
tracing-subscriber = "0.3"
tracing = "0.1"

# 错误处理
anyhow = "1.0"

# 序列化
serde = { version = "1.0", features = ["derive"] }
schemars = { version = "0.8", features = ["derive"] }
```

### 6.3 完整 lib.rs 示例

```rust
use std::sync::Arc;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router,
};
use tokio::sync::Mutex;
use anyhow::Result;
use tracing::{info, error};

/// 服务状态
#[derive(Clone)]
pub struct App {
    tool_router: ToolRouter<App>,
    counter: Arc<Mutex<i32>>,
    messages: Arc<Mutex<Vec<String>>>,
}

/// 请求结构体
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct EchoRequest {
    pub message: String,
}

/// 工具路由器
#[tool_router]
impl App {
    /// 创建新应用实例
    pub fn new() -> Self {
        info!("初始化 MCP 服务器");
        Self {
            tool_router: Self::tool_router(),
            counter: Arc::new(Mutex::new(0)),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// 回显消息
    #[tool(description = "回显用户输入的消息")]
    async fn echo(
        &self,
        Parameters(request): Parameters<EchoRequest>,
    ) -> Result<CallToolResult, McpError> {
        info!("处理 echo 请求: {}", request.message);

        // 保存消息
        {
            let mut messages = self.messages.lock().await;
            messages.push(request.message.clone());
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Echo: {}",
            request.message
        ))]))
    }

    /// 增加计数器
    #[tool(description = "增加计数器值")]
    async fn increment(&self) -> Result<CallToolResult, McpError> {
        let mut counter = self.counter.lock().await;
        *counter += 1;
        info!("计数器值: {}", *counter);

        Ok(CallToolResult::success(vec![Content::text(
            counter.to_string(),
        )]))
    }

    /// 获取消息历史
    #[tool(description = "获取所有回显消息的历史记录")]
    async fn get_messages(&self) -> Result<CallToolResult, McpError> {
        let messages = self.messages.lock().await;
        let history = messages.join("\n");

        Ok(CallToolResult::success(vec![Content::text(
            if history.is_empty() {
                "No messages yet".to_string()
            } else {
                format!("Message history:\n{}", history)
            },
        )]))
    }
}

/// 服务器处理器
#[tool_handler]
impl ServerHandler for App {
    /// 返回服务器信息
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("这是一个示例 MCP 服务器，支持 echo、计数器等功能".to_string()),
        }
    }

    /// 返回工具路由器
    fn get_tool_router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }

    /// 处理服务器错误
    fn handle_error(&self, error: &anyhow::Error) -> Option<ErrorData> {
        error!("服务器错误: {}", error);
        Some(ErrorData::internal_error(error.to_string()))
    }
}
```

---

## 7. 高级特性

### 7.1 自定义验证

```rust
use schemars::{JsonSchema, ValidateJsSchema};
use serde::{Serialize, Deserialize};

// 添加自定义验证
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[schemars(validate_with = "validate_range")]
pub struct RangeRequest {
    pub value: i32,
}

fn validate_range(schema: &schemars::gen::SchemaGenerator, root_schema: &mut schemars::schema::SchemaObject) {
    // 自定义验证逻辑
    if let Some(obj) = &mut root_schema.object {
        if let Some(props) = &mut obj.properties {
            if let Some(value_schema) = props.get_mut("value") {
                // 添加数值范围验证
                // ...
            }
        }
    }
}

#[tool(description = "验证数值范围")]
async fn validate_range(
    &self,
    Parameters(request): Parameters<RangeRequest>,
) -> Result<CallToolResult, McpError> {
    // 验证已通过 schematic 完成
    Ok(CallToolResult::success(vec![Content::text(
        format!("Valid value: {}", request.value),
    )]))
}
```

### 7.2 并发处理

```rust
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Clone)]
pub struct ConcurrentApp {
    tool_router: ToolRouter<ConcurrentApp>,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

#[tool_router]
impl ConcurrentApp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tool(description = "并发读取缓存")]
    async fn read_cache(&self, key: String) -> Result<CallToolResult, McpError> {
        let cache = self.cache.read().await;
        let value = cache.get(&key).cloned().unwrap_or_else(|| "Not found".to_string());
        Ok(CallToolResult::success(vec![Content::text(value)]))
    }

    #[tool(description = "并发写入缓存")]
    async fn write_cache(&self, key: String, value: String) -> Result<CallToolResult, McpError> {
        let mut cache = self.cache.write().await;
        cache.insert(key, value);
        Ok(CallToolResult::success(vec![Content::text("Success")]))
    }
}
```

---

## 8. 最佳实践

### 8.1 错误处理

```rust
use rmcp::ErrorData as McpError;

async fn risky_operation(
    &self,
    params: Parameters<Request>,
) -> Result<CallToolResult, McpError> {
    // 使用 ? 操作符传播错误
    let data = fetch_data(params.0.id).await
        .map_err(|e| McpError::internal_error(e.to_string()))?;

    // 自定义错误
    if data.is_empty() {
        return Err(McpError::invalid_params("Data cannot be empty"));
    }

    Ok(CallToolResult::success(vec![Content::text("Success")]))
}
```

### 8.2 日志记录

```rust
use tracing::{info, warn, error, debug};

#[tool(description = "处理请求")]
async fn handle_request(
    &self,
    Parameters(request): Parameters<Request>,
) -> Result<CallToolResult, McpError> {
    info!("收到请求: {:?}", request);

    // 业务逻辑
    let result = process(request).await
        .map_err(|e| {
            error!("处理请求失败: {}", e);
            McpError::internal_error(e.to_string())
        })?;

    debug!("请求处理成功: {:?}", result);
    Ok(CallToolResult::success(vec![Content::text(result)]))
}
```

---

## 9. 常见问题

### Q: 如何添加新工具？

A: 只需在 impl 块中添加新的 `#[tool]` 标记的函数，路由器宏会自动注册。

### Q: 如何自定义参数验证？

A: 使用 `schemars::JsonSchema` 和 `ValidateJsSchema` trait，或在函数内添加验证逻辑。

### Q: 如何处理异步操作？

A: 直接使用 `async` 函数，rmcp 原生支持 async/await。

### Q: 如何返回复杂数据？

A: 使用 `Json<T>` 类型，自动生成输出 schema。

---

## 10. 总结

MCP Rust SDK 的 stdio 工具注册官方方式具有以下优势：

1. **声明式注册**: 通过宏自动生成路由代码
2. **类型安全**: 编译时验证参数类型
3. **Schema 自动生成**: 基于 Rust 类型自动生成 JSON Schema
4. **最小样板代码**: 消除重复的注册逻辑
5. **异步支持**: 原生 async/await 支持
6. **模块化设计**: 清晰的分层架构

这套实现模式已经被多个生产项目验证，是 MCP Rust 生态的标准做法。

---

## 参考资源

- **官方仓库**: [https://github.com/modelcontextprotocol/rust-sdk](https://github.com/modelcontextprotocol/rust-sdk)
- **文档**: [https://docs.rs/rmcp](https://docs.rs/rmcp)
- **示例**: 查看项目中的 examples 目录
