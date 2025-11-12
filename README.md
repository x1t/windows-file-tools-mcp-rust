# File-Bash-Tools MCP Server

🚀 **企业级文件和Shell工具MCP服务器** - 基于Rust MCP SDK实现的高性能工具集

## 📋 功能特性

### 📁 **文件操作工具**
- **Write** - 安全的文件写入，支持自动目录创建
- **Read** - 智能文件读取，支持文本和图像格式
- **Edit** - 精确的文本替换和编辑功能

### 🔍 **搜索功能**
- **Grep** - 基于ripgrep的强大文本搜索
- **Glob** - 基于fd的高效文件模式匹配

### 💻 **Shell工具**
- **Bash** - PowerShell命令执行（仅支持pwsh.exe）
- **BashOutput** - 后台进程输出获取
- **KillBash** - 安全的后台进程终止

## 🏗️ 架构设计

- **高性能**: 基于tokio异步运行时
- **类型安全**: 完整的Rust类型系统支持
- **企业级**: 完善的错误处理和安全机制
- **模块化**: 清晰的代码组织和接口设计

## 🚀 快速开始

### 环境要求

- **Rust**: 1.90+
- **PowerShell**: pwsh.exe
- **外部工具**: ripgrep (rg), fd

### 安装依赖

```bash
# Windows (使用PowerShell)
winget install BurntSushi.ripgrep
winget install sharkdp.fd

# 或使用scoop
scoop install ripgrep fd
```

### 编译运行

```bash
# 克隆项目
git clone <repository-url>
cd file-bash-tools-mcp

# 编译
cargo build --release

# 运行
cargo run

# 或直接运行二进制文件
./target/release/file-bash-tools-mcp
```

## 📖 工具使用指南

### Write工具
```json
{
  "file_path": "C:\\path\\to\\file.txt",
  "content": "Hello, World!"
}
```

### Read工具
```json
{
  "file_path": "C:\\path\\to\\file.txt",
  "offset": 1,
  "limit": 10
}
```

### Edit工具
```json
{
  "file_path": "C:\\path\\to\\file.txt",
  "old_string": "World",
  "new_string": "Rust",
  "replace_all": false
}
```

### Grep工具
```json
{
  "pattern": "TODO",
  "path": "C:\\project",
  "output_mode": "content",
  "-i": true,
  "-n": true,
  "-C": 3
}
```

### Glob工具
```json
{
  "pattern": "*.rs",
  "path": "C:\\project\\src"
}
```

### Bash工具
```json
{
  "command": "Get-Process",
  "timeout": 10000,
  "description": "获取进程列表",
  "run_in_background": false
}
```

## 🧪 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_file_operations

# 运行集成测试
cargo test --test integration_tests
```

## 📊 性能特性

- **并发处理**: 支持多文件并行搜索
- **内存优化**: 流式处理大文件
- **缓存机制**: 智能结果缓存
- **超时控制**: 可配置的命令执行超时

## 🔒 安全特性

- **路径验证**: 防止路径遍历攻击
- **命令过滤**: 基础的危险命令检测
- **权限检查**: 文件读写权限验证
- **资源限制**: 内存和CPU使用限制

## 🛠️ 开发

### 代码结构
```
src/
├── models/           # 数据模型定义
│   ├── file_ops.rs   # 文件操作模型
│   ├── search.rs     # 搜索模型
│   └── shell.rs      # Shell操作模型
├── tools/            # 工具实现
│   ├── file_tools.rs # 文件工具
│   ├── search_tools.rs # 搜索工具
│   └── shell_tools.rs # Shell工具
├── utils/            # 工具函数
│   ├── ripgrep_utils.rs # ripgrep封装
│   └── fd_utils.rs   # fd封装
└── handlers/         # 处理器
    └── file_handler.rs # 文件处理器
```

### 代码质量工具

```bash
# 格式化代码
cargo fmt

# 代码检查
cargo clippy

# 生成文档
cargo doc --open

# 测试覆盖率
cargo llvm-cov --lcov
```

## 🤝 贡献

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情。

## 🙏 致谢

- [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) - MCP协议Rust实现
- [ripgrep](https://github.com/BurntSushi/ripgrep) - 高性能文本搜索工具
- [fd](https://github.com/sharkdp/fd) - 用户友好的文件查找工具

## 📞 联系方式

- **项目主页**: [GitHub Repository](https://github.com/xctcc/file-bash-tools-mcp)
- **问题反馈**: [Issues](https://github.com/xctcc/file-bash-tools-mcp/issues)
- **作者**: XCT CC

---

⭐ 如果这个项目对你有帮助，请给个星标支持！