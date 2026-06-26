//! 配置模块。
//!
//! 默认配置保证程序开箱即用；TOML 配置文件则允许用户根据自己的开发环境
//! 自定义忽略目录、残留目录规则、保护目录和扫描阈值。

use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// 扫描时跳过的目录名，例如 .git 和本项目自己的回收站目录。
    pub ignore_dirs: Vec<String>,
    /// 被认为是“开发残留”的目录名规则。
    pub residue_dirs: Vec<String>,
    /// 不允许被清理的系统目录或高风险目录。
    pub protected_roots: Vec<PathBuf>,
    /// 可选最大扫描深度；为 None 时不限制深度。
    pub max_depth: Option<usize>,
    /// TUI 默认使用的大文件阈值。
    pub default_large_file_threshold: String,
    /// 参与重复文件检测的最小文件大小，过小文件可跳过以减少噪声。
    pub duplicate_min_size: u64,
}

impl AppConfig {
    /// 从用户指定的 TOML 文件加载配置；未指定时返回默认配置。
    pub fn load(path: Option<PathBuf>) -> anyhow::Result<Self> {
        match path {
            Some(path) => {
                let content = fs::read_to_string(&path)?;
                let config = toml::from_str(&content)?;
                Ok(config)
            }
            None => Ok(Self::default()),
        }
    }

    /// 把默认配置格式化为 TOML，便于用户生成配置模板。
    pub fn to_pretty_toml(&self) -> anyhow::Result<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ignore_dirs: vec![
                ".git".into(),
                ".svn".into(),
                ".hg".into(),
                ".aegis_disk_trash".into(),
            ],
            residue_dirs: vec![
                "target".into(),
                "node_modules".into(),
                ".cache".into(),
                "dist".into(),
                "build".into(),
                "__pycache__".into(),
                ".pytest_cache".into(),
                ".next".into(),
                "coverage".into(),
            ],
            protected_roots: default_protected_roots(),
            max_depth: None,
            default_large_file_threshold: "100MB".into(),
            duplicate_min_size: 1,
        }
    }
}

/// 根据操作系统返回默认保护目录。
///
/// macOS/Linux 下会保护系统目录；Windows 下保护 Windows 和 Program Files。
fn default_protected_roots() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from("C:\\Windows"),
            PathBuf::from("C:\\Program Files"),
        ]
    }
    #[cfg(not(target_os = "windows"))]
    {
        vec![
            PathBuf::from("/"),
            PathBuf::from("/System"),
            PathBuf::from("/bin"),
            PathBuf::from("/sbin"),
            PathBuf::from("/usr"),
            PathBuf::from("/etc"),
            PathBuf::from("/Library"),
        ]
    }
}
