# AegisDisk

`AegisDisk`，是一个使用 Rust 编写的终端磁盘智能感知与安全清理工具。

`Aegis` 有“神盾、防护”的含义，因此该名称强调项目不仅能扫描磁盘，还注重安全保护、仿真演练和可恢复清理。

## 功能特性

- 递归扫描指定目录，统计文件数、目录数、总占用空间
- 大文件分析，支持 `100MB`、`1GiB` 等大小表达式
- 开发残留识别，支持 `target/`、`node_modules/`、`.cache/`、`dist/`、`build/`、`__pycache__/` 等目录
- 开发残留目录真实大小估算，用于 dry-run 预计释放空间
- 重复文件检测，先按文件大小分组，再使用 `blake3` 并发计算哈希
- TOML 配置文件支持，可自定义忽略目录、残留规则和阈值
- 风险分级：`SAFE`、`REVIEW`、`DANGER`
- 默认支持 dry-run 仿真演练
- 执行清理前二次确认，可用 `--yes` 跳过交互确认
- 支持将清理对象移动到 `.aegis_disk_trash/` 隔离目录
- 内置系统目录保护
- 三看板 TUI：概览、发现项、详情
- 包含单元测试与集成测试

## 编译运行
```bash
cargo build
```

扫描目录：

```bash
cargo run -- scan .
```

查找大文件：

```bash
cargo run -- large . --min-size 100MB
```

识别开发残留：

```bash
cargo run -- residue .
```

检测重复文件：

```bash
cargo run -- duplicates .
```

进入 TUI：

```bash
cargo run -- tui .
```

清理前仿真演练：

```bash
cargo run -- clean . --dry-run --target residue
```

执行安全隔离清理：

```bash
cargo run -- clean . --execute --target residue
```

执行时会要求输入 `YES` 二次确认。如果你在自动化脚本或演示环境中确认无误，可以使用：

```bash
cargo run -- clean . --execute --yes --target residue
```

输出默认配置文件：

```bash
cargo run -- config print-default > aegis-disk.toml
```

使用配置文件运行：

```bash
cargo run -- --config aegis-disk.toml scan .
```

配置文件可以修改忽略目录、开发残留规则、保护目录、大文件默认阈值和重复文件最小检测大小。

重复文件清理计划：

```bash
cargo run -- clean . --dry-run --target duplicates
```

## TUI 按键

- `q` / `Esc`：退出
- `j` / `Down`：下移
- `k` / `Up`：上移
- `Tab`：切换看板

## 模块结构

```text
src/
  main.rs
  lib.rs
  app.rs
  cli.rs
  config.rs
  error.rs
  fs/
    metadata.rs
    safety.rs
    scanner.rs
  analysis/
    large.rs
    residue.rs
    duplicate.rs
    rule.rs
  cleaner/
    plan.rs
    trash.rs
  tui/
    state.rs
    view.rs
    events.rs
  utils/
    format.rs
```

## Rust 特性体现

- 使用 `struct` 表示文件元数据、扫描报告、清理计划
- 使用 `enum` 表示风险等级、发现类型、清理动作
- 使用 `trait Analyzer` 抽象不同分析器
- 使用切片借用 `&[FileEntry]` 在多个分析器间共享扫描结果，避免重复拷贝
- 使用 `Result` 和 `thiserror` 处理错误
- 使用 `rayon` 并发计算重复文件哈希
- 使用模块化组织工程
- 使用 `tempfile` 编写文件系统测试

## 测试与规范

格式化：

```bash
cargo fmt
```

测试：

```bash
cargo test
```

静态检查：

```bash
cargo clippy -- -D warnings
```
