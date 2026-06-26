//! 路径安全保护模块。
//!
//! 清理工具最重要的要求之一是不能误删系统目录。本模块把保护目录封装成
//! `SafetyGuard`，清理前和执行清理时都可以重复调用，形成双重保护。

use std::path::{Path, PathBuf};

use crate::config::AppConfig;

#[derive(Debug, Clone)]
pub struct SafetyGuard {
    protected_roots: Vec<PathBuf>,
}

impl SafetyGuard {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            protected_roots: config.protected_roots.clone(),
        }
    }

    /// 判断某个路径是否位于保护目录中。
    ///
    /// 对 `/` 这种根目录只做精确匹配，否则所有 Unix 路径都会因为以 `/` 开头
    /// 而被误判为受保护；对 `/System`、`/usr` 等具体目录则使用前缀保护。
    pub fn is_protected(&self, path: &Path) -> bool {
        let normalized = normalize_path(path);
        self.protected_roots.iter().any(|root| {
            let normalized_root = normalize_path(root);
            if normalized_root.parent().is_none() {
                normalized == normalized_root
            } else {
                normalized == normalized_root || normalized.starts_with(&normalized_root)
            }
        })
    }

    /// 判断用户传入的扫描根目录本身是否就是危险目录。
    pub fn is_dangerous_root(&self, path: &Path) -> bool {
        let normalized = normalize_path(path);
        self.protected_roots
            .iter()
            .any(|root| normalized == normalize_path(root))
    }
}

/// 尽量把路径转换为规范路径；如果路径不存在或无权限，则保留原路径。
pub fn normalize_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

/// 提取路径中的可读组件，当前主要作为扩展工具函数保留。
pub fn path_component_names(path: &Path) -> impl Iterator<Item = String> + '_ {
    path.components()
        .filter_map(|component| component.as_os_str().to_str())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn exact_protected_root_is_dangerous() {
        let guard = SafetyGuard::new(&AppConfig {
            protected_roots: vec![PathBuf::from("/tmp/sentinel-protected")],
            ..AppConfig::default()
        });
        assert!(guard.is_dangerous_root(Path::new("/tmp/sentinel-protected")));
    }
}
