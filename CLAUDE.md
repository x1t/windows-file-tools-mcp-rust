# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 项目概述

企业级 Windows 文件操作 MCP（Model Context Protocol）服务器，Rust + **rmcp 3.1.1** 构建，通过 stdio 与 MCP 客户端通信。提供文件读写、编辑、glob 匹配、ripgrep 搜索和 TodoWrite 六个工具。crate 名 `file-bash-tools-mcp`（import 为 `file_bash_tools_mcp`）。

## 架构：单文件实现，src/ 只有两个文件

`src/lib.rs`（951 行）是**唯一被编译的服务器实现**：所有请求结构体、路径校验、原子文件操作、搜索逻辑、六个 `#[tool]`、`ServerHandler` 和全部单元测试都在这里。`src/main.rs`（30 行）只做 tracing 初始化和 `FileBashToolsService::new().serve(stdio()).await`。

代码刻意集中在单文件而非模块化（符合 KISS）；不要在 `src/` 下新建模块目录。

## 常用命令

```bash
cargo build / cargo build --release   # 构建
cargo run / cargo run --release       # 运行（stdio 服务器）
cargo test                            # 运行全部测试（lib.rs 的 mod tests）
cargo test test_glob_pattern_matching # 运行单个测试
cargo test -- --nocapture             # 显示 println 输出
RUST_LOG=debug cargo run              # 详细日志（输出到 stderr）
cargo fmt && cargo clippy             # 格式化与静态检查
# MCP Inspector 交互式测试
npx @modelcontextprotocol/inspector cargo run --release
```

## rmcp 3.1.1 API 约定（自 0.8.x 升级适配后）

- 服务入口：`use rmcp::ServiceExt;` → `serve(stdio()).await` → `.waiting().await`
- 错误类型：`use rmcp::ErrorData as McpError`；构造用 `McpError::invalid_params(msg, None)` / `McpError::internal_error(msg, None)`
- 工具签名：`async fn tool(&self, Parameters(req): Parameters<XxxRequest>) -> Result<CallToolResult, McpError>`
- 成功返回：`CallToolResult::success(vec![ContentBlock::text(...)])`（3.x 由 `Content` 更名）
- 注册：`#[tool_router] impl FileBashToolsService` 内每个工具加 `#[tool(name = "...", description = "...")]`
- ServerInfo：`#[tool_handler] impl ServerHandler for Service` 的 `fn get_info()`；`ServerInfo::default()` 后逐字段赋值（`non_exhaustive` 不能结构体字面量），capabilities 用 `ServerCapabilities::builder().enable_tools().build()`，protocol 用 `ProtocolVersion::V_2024_11_05`

## 数据流

```
MCP Client → stdio → main.rs → lib.rs（#[tool_router] 路由）→ 具体工具 → 文件系统
```

## 工具清单（均在 lib.rs）

- `write_file` / `read_file` / `edit_file`：请求结构 `WriteRequest` / `ReadRequest` / `EditRequest`
- `glob`：方法名 `glob_tool`，结构 `GlobRequest`
- `grep`：结构 `GrepRequest`（17 个字段：pattern/path/glob/type/output_mode/case_insensitive/show_line_numbers/before/after/context/head_limit/multiline 等）
- `todo_write`：结构 `TodoWriteRequest` + `TodoItem` / `TodoStatus`

## 关键实现细节

- **原子写**：`tempfile::NamedTempFile` 建在目标同目录 → `persist()` 原子重命名；失败降级 `tokio::fs::write`（`atomic_write_file` / `fallback_write_file`）
- **并发**：`Arc<Semaphore>::new(10)`（`file_semaphore`）限制同时读取文件数
- **搜索深度**（`search_files`，lib.rs:377）：`files_with_matches`→20 层，`count`→30 层，默认→有 glob 过滤 10 层否则 50 层
- **性能**：跳过 >10MB 文件；`walkdir` 遍历且 `follow_links(false)`
- **Windows 路径**：所有工具标注 "Only Windows"，路径用双反斜杠 `C:\\path\\file`；`validate_file_path` 强制绝对路径、拒绝 `..` 目录穿越

## 测试

- 单元测试在 `src/lib.rs` 末尾 `mod tests`（4 个 `#[tokio::test]`，无 mock）：`test_glob_pattern_matching`、`test_glob_request_validation`、`test_grep_regex_matcher`、`test_todo_write_request_validation`
- 集成测试用 `mcp-client/` 目录下的 TypeScript 客户端（源码已入库，git 忽略 node_modules/dist）：`@modelcontextprotocol/sdk` 通过 stdio 连 `cargo run --release`，脚本逐个验证工具
  - `mcp-client/src/test_file_tools.ts`、`test_all_tools.ts` 等
  - 依赖注意：`zod` 需显式安装；pnpm 重建需 `CI=true`；node_modules 已提交忽略

## 相关文档

- `README.md`：功能与结构说明
- 根目录 `test_files/`：测试用临时文件

## 设计原则

遵循 KISS > YAGNI > SOLID 的优先级，保持简单直接。代码注释使用中文，日志消息使用 emoji 前缀。
