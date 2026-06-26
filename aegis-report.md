# AegisDisk Scan Report

- Root: `demo`
- Format: Markdown

## 1. Scan Summary

| Metric | Value |
|---|---:|
| Files | 6 |
| Directories | 5 |
| Total Size | 2.62 MB |
| Access Errors | 0 |
| Findings | 4 |

## 2. Diagnostic Insights

### [INFO] Scan completed without access errors

All accessible entries were scanned successfully.

### [NOTICE] Finding risk distribution

Detected 3 safe item(s), 1 review item(s), and 0 dangerous item(s) under demo.

### [INFO] No large-file pressure detected

Large files do not dominate this scan result.

### [NOTICE] Development residue can be cleaned safely first

Development residue accounts for about 20.0% of scanned space (524.29 kB). Start with dry-run.

Suggested command:

```bash
aegis-disk clean demo --dry-run --target residue
```

### [INFO] Duplicate files may release extra space

Duplicate removal candidates account for about 0.0% of scanned space (13 B).

Suggested command:

```bash
aegis-disk duplicates demo
```

### [NOTICE] Recommended review order

Review SAFE residue first, then duplicate candidates, then large files that require manual judgement.

Suggested command:

```bash
aegis-disk tui demo
```

## 3. Findings

| Risk | Size | Kind | Path | Reason |
|---|---:|---|---|---|
| SAFE | 524.29 kB | dev-residue | `demo/target` | matched development residue directory rule: target |
| REVIEW | 13 B | duplicate-0-keep | `demo/b.txt` | duplicate group 0, recommended file to keep |
| SAFE | 13 B | duplicate-0-remove | `demo/a.txt` | duplicate group 0, same blake3 hash |
| SAFE | 6 B | dev-residue | `demo/node_modules` | matched development residue directory rule: node_modules |

## 4. File Type Distribution

| Extension | Files | Total Size | Average Size | Largest File |
|---|---:|---:|---:|---|
| `bin` | 2 | 2.62 MB | 1.31 MB | demo/large.bin |
| `txt` | 2 | 26 B | 13 B | demo/a.txt |
| `rs` | 1 | 13 B | 13 B | demo/src/main.rs |
| `js` | 1 | 6 B | 6 B | demo/node_modules/pkg/index.js |

## 5. Safety Note

This report is read-only. Before executing cleanup, run `clean --dry-run` first and review all paths manually.
