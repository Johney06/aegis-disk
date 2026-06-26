//! 程序二进制入口。
//!
//! 这里保持尽量简洁：只负责解析命令行参数，然后把真正的业务流程交给 `app::run`。
//! 这样做可以让主流程更容易测试，也方便后续把核心逻辑作为库复用。

use aegis_disk::{app, cli::Cli};
use anyhow::Result;

fn main() -> Result<()> {
    // `Cli::parse_args` 内部使用 clap 读取终端参数，并转换成强类型命令枚举。
    let cli = Cli::parse_args();
    app::run(cli)
}
