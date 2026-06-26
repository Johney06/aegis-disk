//! 集成测试。
//!
//! 这些测试会使用 `tempfile` 创建临时目录和临时文件，不会扫描或修改用户真实文件。
//! 测试结束后，临时目录会自动清理，适合验证文件系统相关逻辑。

use std::fs;

use aegis_disk::{
    analysis::{
        Analyzer, DuplicateAnalyzer, FileTypeAnalyzer, LargeFileAnalyzer, ResidueAnalyzer,
        rule::cleanable_findings,
    },
    app,
    cleaner::{CleanPlan, TrashCleaner},
    cli::{Cli, Command},
    config::AppConfig,
    fs::{SafetyGuard, Scanner},
    utils::format::parse_size,
};
use tempfile::tempdir;

#[test]
fn scanner_collects_files_and_directories() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    let report = Scanner::new(AppConfig::default()).scan(dir.path());

    assert!(report.stats.files >= 1);
    assert!(report.stats.dirs >= 1);
    assert!(report.stats.total_size > 0);
}

#[test]
fn analyzers_find_expected_items() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("target")).unwrap();
    fs::write(dir.path().join("target/cache.bin"), vec![1_u8; 128]).unwrap();
    fs::write(dir.path().join("large.bin"), vec![2_u8; 256]).unwrap();
    fs::write(dir.path().join("a.txt"), "same").unwrap();
    fs::write(dir.path().join("b.txt"), "same").unwrap();

    let config = AppConfig::default();
    let report = Scanner::new(config.clone()).scan(dir.path());

    // target 目录应该被识别为开发残留，并且大小应等于内部 cache 文件大小。
    let residues = ResidueAnalyzer::new(&config).analyze(&report.entries);
    assert_eq!(residues[0].size, 128);

    // large.bin 超过 200 字节阈值，应被大文件分析器发现。
    assert!(
        !LargeFileAnalyzer::new(200)
            .analyze(&report.entries)
            .is_empty()
    );

    // a.txt 和 b.txt 内容相同，应形成一个重复文件组，共两个发现项。
    assert_eq!(DuplicateAnalyzer::new(1).analyze(&report.entries).len(), 2);

    // 文件类型分析器应能统计 bin 和 txt 等扩展名的数量与空间。
    let type_stats = FileTypeAnalyzer::new().analyze(&report.entries);
    assert!(type_stats.iter().any(|stat| stat.extension == "bin"));
    assert!(type_stats.iter().any(|stat| stat.extension == "txt"));
}

#[test]
fn dry_run_estimates_reclaimable_space() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("target")).unwrap();
    fs::write(dir.path().join("target/cache.bin"), vec![1_u8; 512]).unwrap();

    let config = AppConfig::default();
    let report = Scanner::new(config.clone()).scan(dir.path());
    let findings = ResidueAnalyzer::new(&config).analyze(&report.entries);
    let cleanable = cleanable_findings(&findings);
    let plan = CleanPlan::from_findings(
        dir.path().to_path_buf(),
        &cleanable,
        dir.path().join(".aegis_disk_trash"),
    );
    let summary = TrashCleaner::new(SafetyGuard::new(&config)).dry_run(&plan);

    assert_eq!(plan.estimated_bytes(), 512);
    assert_eq!(summary.reclaimed_bytes, 512);
}

#[test]
fn config_can_roundtrip_as_toml() {
    let config = AppConfig::default();
    let toml = config.to_pretty_toml().unwrap();
    let decoded: AppConfig = toml::from_str(&toml).unwrap();

    assert!(decoded.residue_dirs.contains(&"target".to_owned()));
}

#[test]
fn scan_reports_missing_root_as_error() {
    let dir = tempdir().unwrap();
    let missing = dir.path().join("missing");
    let cli = Cli {
        config: None,
        command: Command::Scan {
            path: missing,
            limit: 10,
        },
    };

    let error = app::run(cli).unwrap_err().to_string();
    assert!(error.contains("scan root does not exist"));
}

#[test]
fn size_parser_supports_decimal_units() {
    assert_eq!(parse_size("100MB").unwrap(), 100_000_000);
    assert_eq!(parse_size("1KiB").unwrap(), 1024);
}
