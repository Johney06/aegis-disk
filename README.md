# AegisDisk

`AegisDisk`（磁盘神盾）是一个使用 Rust 编写的终端磁盘智能感知与安全清理工具。项目名称中的 `Aegis` 有“神盾、防护”的含义，因此本项目不仅关注磁盘空间分析，也强调清理前的风险判断、清理过程中的安全保护和清理后的可恢复性。

`Aegis` 有“神盾、防护”的含义，因此该名称强调项目不仅能扫描磁盘，还注重安全保护、仿真演练和可恢复清理。

## 1. 功能特性

- 递归扫描指定目录，统计文件数量、目录数量、总占用空间和访问错误数量
- 大文件分析，支持 `100MB`、`1GiB` 等大小表达式
- 开发残留识别，支持 `target/`、`node_modules/`、`.cache/`、`dist/`、`build/`、`__pycache__/` 等目录
- 开发残留目录真实大小估算，用于 dry-run 预计释放空间
- 重复文件检测，先按文件大小分组，再使用 `blake3` 并发计算哈希
- 文件类型分布统计，按扩展名输出文件数量、总大小、平均大小和最大文件
- 诊断建议模块，根据扫描结果生成下一步处理建议
- Markdown / JSON 报告导出，方便提交作业、归档或脚本处理
- TOML 配置文件支持，可自定义忽略目录、残留规则、保护路径和阈值
- 风险分级：`SAFE`、`REVIEW`、`DANGER`
- 支持 dry-run 仿真清理，真正执行前可预览清理计划
- 执行清理前需要二次确认，可使用 `--yes` 跳过交互确认
- 清理对象移动到 `.aegis_disk_trash/` 隔离目录，而不是直接永久删除
- 内置系统目录保护，避免误操作关键路径
- 高级 TUI 仪表盘，支持指标卡、风险统计、空间占比、过滤、排序和详情查看
- 包含单元测试与集成测试

## 2. 环境要求

- Rust 2024 Edition
- Cargo
- macOS / Linux / Windows 终端环境均可运行，TUI 建议使用支持 ANSI 的现代终端

查看 Rust 版本：

```bash
rustc --version
cargo --version
```

## 3. 快速开始

进入项目目录：

```bash
cd "disk-sentinel"
```

编译项目：

```bash
cargo build
```

运行测试：

```bash
cargo test
```

查看命令帮助：

```bash
cargo run -- --help
```

## 4. 常用命令

扫描目录：

```bash
cargo run -- scan .
```

查找大文件：

```bash
cargo run -- large . --min-size 100MB
```

识别开发残留目录：

```bash
cargo run -- residue .
```

检测重复文件：

```bash
cargo run -- duplicates .
```

统计文件类型分布：

```bash
cargo run -- types . --limit 20
```

生成诊断建议：

```bash
cargo run -- insights . --limit 5
```

导出 Markdown 报告：

```bash
cargo run -- export . --format markdown --output aegis-report.md
```

导出 JSON 报告：

```bash
cargo run -- export . --format json --output aegis-report.json
```

进入 TUI 仪表盘：

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

执行清理时程序会要求输入 `YES` 二次确认。如果在明确知道风险的脚本或演示环境中运行，可以使用：

```bash
cargo run -- clean . --execute --yes --target residue
```

## 5. 配置文件

输出默认配置：

```bash
cargo run -- config print-default > aegis-disk.toml
```

使用配置文件运行：

```bash
cargo run -- --config aegis-disk.toml scan .
```

配置文件可以修改：

- 忽略目录
- 开发残留目录规则
- 系统保护路径
- 默认大文件阈值
- 重复文件最小检测大小

## 6. TUI 快捷键

新版 TUI 采用仪表盘布局，包含顶部标题栏、扫描指标卡、空间指标卡、风险统计卡、视图控制卡、空间占比进度条、发现项列表、详情面板和底部帮助栏。

| 按键 | 作用 |
|---|---|
| `↑` / `↓` | 移动发现项 |
| `j` / `k` | 移动发现项 |
| `n` / `p` | 下一项 / 上一项 |
| `Home` / `End` | 跳到第一项 / 最后一项 |
| `←` / `→` | 切换面板 |
| `Tab` | 切换面板 |
| `Enter` / `Space` | 切换面板 |
| `f` | 切换风险过滤：`ALL`、`SAFE`、`REVIEW`、`DANGER` |
| `s` | 切换排序方式：大小、风险、类型、路径 |
| `r` | 重置过滤和排序 |
| `q` / `Esc` / `Ctrl+C` | 退出 TUI |

## 7. 项目结构

```text
aegis-disk/
  Cargo.toml
  Cargo.lock
  README.md
  src/
    main.rs
    lib.rs
    app.rs
    cli.rs
    config.rs
    error.rs
    fs/
      metadata.rs
      scanner.rs
      safety.rs
    analysis/
      mod.rs
      large.rs
      residue.rs
      duplicate.rs
      file_type.rs
      insight.rs
      rule.rs
    cleaner/
      mod.rs
      plan.rs
      trash.rs
    export/
      mod.rs
      markdown.rs
      json.rs
    tui/
      mod.rs
      state.rs
      view.rs
      events.rs
    utils/
      mod.rs
      format.rs
  tests/
    core.rs
```

## 8. 核心设计

### 8.1 分层架构

项目采用分层结构：

```text
CLI 参数解析
  ↓
配置加载
  ↓
文件系统扫描 Scanner
  ↓
分析器 Analyzer
  ↓
命令行输出 / TUI 展示 / 报告导出 / 清理计划
  ↓
dry-run 或安全隔离清理
```

扫描模块只负责收集文件信息；分析模块只负责产生发现项；清理模块只负责执行安全动作；TUI 和导出模块只负责展示结果。不同模块之间通过结构体和 trait 传递数据，降低耦合。

### 8.2 分析器 trait

项目使用统一的 `Analyzer` trait 抽象不同分析器：

```rust
pub trait Analyzer {
    fn name(&self) -> &'static str;
    fn analyze(&self, entries: &[FileEntry]) -> Vec<Finding>;
}
```

`LargeFileAnalyzer`、`ResidueAnalyzer` 和 `DuplicateAnalyzer` 都实现了该接口。分析器共享扫描结果的只读切片 `&[FileEntry]`，避免复制大量文件元数据。

### 8.3 安全清理机制

清理功能默认推荐使用 dry-run：

```bash
cargo run -- clean . --dry-run --target residue
```

真正执行时，程序会：

1. 检查是否指定了 `--execute`
2. 检查目标路径是否属于保护路径
3. 要求用户输入 `YES` 二次确认
4. 将清理对象移动到 `.aegis_disk_trash/`
5. 输出移动数量、跳过数量、失败数量和预计释放空间

这种设计避免了直接删除文件造成不可恢复损失。

### 8.4 重复文件检测

重复文件检测采用两阶段策略：

1. 先按文件大小分组，不同大小的文件不可能完全重复
2. 只对大小相同的候选文件计算 BLAKE3 哈希
3. 使用 `rayon::par_iter()` 并发计算哈希
4. 每组推荐一个保留文件，其余标记为可清理候选

这样既减少了不必要的磁盘 I/O，又体现了 Rust 并发处理能力。

## 9. Rust 特性体现

- 使用 `struct` 表示文件元数据、扫描报告、发现项、清理计划和导出上下文
- 使用 `enum` 表示命令类型、风险等级、发现项类型、过滤模式和排序模式
- 使用 `trait Analyzer` 抽象分析器接口
- 使用所有权和借用共享扫描结果，避免重复拷贝
- 使用 `Result`、`anyhow` 和 `thiserror` 处理错误
- 使用 `serde`、`toml` 和 `serde_json` 实现配置和报告序列化
- 使用 `rayon` 并发计算重复文件哈希
- 使用 `ratatui` 和 `crossterm` 实现终端 UI
- 使用 `tempfile` 编写不污染真实文件系统的集成测试

## 10. 错误处理

项目实现了较完整的错误处理机制：

- 无效大小表达式会返回 `InvalidSize`
- 扫描路径不存在会返回 `ScanRootNotFound`
- 扫描目标不是目录会返回 `ScanRootNotDirectory`
- 清理命令未指定模式会返回 `MissingCleanMode`
- 同时使用 `--dry-run` 和 `--execute` 会返回 `ConflictingCleanMode`
- 尝试清理保护路径会返回 `ProtectedPath`
- 单个文件扫描失败不会导致程序崩溃，而是记录到扫描报告中
- 清理移动失败会记录到清理摘要中

## 11. 测试与规范

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

当前项目包含单元测试和集成测试，覆盖扫描、分析、配置序列化、错误处理和 dry-run 清理估算等功能。

## 12. 代码规模

排除空行和注释后，项目当前有效 Rust 代码约为：

```text
src 主程序有效代码：2604 行
tests 测试有效代码：90 行
总有效 Rust 代码：2694 行
```
