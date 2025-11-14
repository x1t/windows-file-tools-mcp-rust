# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is an enterprise-grade File and Bash Tools MCP (Model Context Protocol) Server implemented in Rust. It provides file operations and shell command execution capabilities through the MCP protocol, specifically designed for Windows environments.

## Key Features

- File operations: Write, Read, Edit
- Shell command execution: Bash (PowerShell)
- File pattern matching: Glob
- Text search: Grep (powered by ripgrep)
- Task management: TodoWrite

## Common Development Commands

### Building the Project
```bash
cargo build
cargo build --release
```

### Running Tests
```bash
cargo test
```

### Running the Server
```bash
cargo run
```

### Debugging with MCP Inspector
```bash
npx @modelcontextprotocol/inspector cargo run
```

## Code Architecture

### Main Components

1. **Main Entry Point**: `src/main.rs` - Server initialization and startup
2. **Core Service**: `src/lib.rs` - Main service implementation with all tools
3. **File Operations**: Directly implemented in `src/lib.rs`
4. **Shell Tools**: `src/tools/shell_tools.rs`
5. **Search Tools**: `src/tools/search_tools.rs`
6. **File Tools**: `src/tools/file_tools.rs`

### Key Patterns

1. **Tool Registration**: Uses `#[tool_router]` and `#[tool]` macros for automatic tool registration
2. **Parameter Handling**: Uses `Parameters<T>` wrapper for automatic parameter deserialization
3. **Error Handling**: Uses `McpError` for consistent error responses
4. **Async Operations**: All tools are async functions using tokio runtime

### Data Structures

1. **WriteRequest**: File write operation parameters
2. **ReadRequest**: File read operation parameters
3. **EditRequest**: File edit operation parameters
4. **GlobRequest**: File pattern matching parameters
5. **GrepRequest**: Text search parameters
6. **BashRequest**: Shell command execution parameters
7. **TodoItem**: Task management item structure

## Development Guidelines

### Adding New Tools

1. Define a request structure implementing `serde::Deserialize` and `JsonSchema`
2. Add a new method in the `#[tool_router]` impl block with `#[tool]` attribute
3. Follow the existing patterns for parameter handling and error management

### Testing

1. Unit tests are included in each module
2. Use `cargo test` to run all tests
3. Follow existing test patterns for new functionality

### Error Handling

1. Use `McpError::invalid_params()` for parameter validation errors
2. Use `McpError::internal_error()` for internal failures
3. Always provide meaningful error messages

### Logging

1. Use `tracing::info!`, `tracing::debug!`, `tracing::warn!`, `tracing::error!` macros
2. Include relevant context in log messages
3. Use appropriate log levels (debug for detailed information, info for key events)

## Windows-Specific Considerations

1. All file paths must be absolute
2. Use double backslashes for Windows paths in documentation/examples
3. Path validation includes security checks for relative path traversal