# Interactive Map Title and Kop Metadata Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make interactive runtime prompts ask for Drawn By, Title ("PIT A CUT & FILL MAP"), Company Name, and Design Name with smart defaults, auto-detect Date Created as today's date (formatted "D MMMM YYYY"), and auto-detect Topo Survey Date from Topo DXF file last modified timestamp.

**Architecture:** 
1. Introduce helper functions for formatted current date (e.g. "2 September 2026") and file modification date extraction without external heavy crates (using `std::time::SystemTime` + standard UTC/local calendar conversion).
2. Update CLI/interactive flow in `src/main.rs` to prompt:
   - Drawn By: default from `config.dat` (or CLI `--drawn-by`)
   - Title / Project Title: default from `config.dat` (or CLI `--rainbow-title`)
   - Company Name: default from `config.dat` (or CLI `--company`)
   - Design Name: default from design DXF file stem (or `config.dat`)
   - Date Created: automatically today's date (no prompt unless overridden via CLI/config if needed, displayed in kop)
   - Topo Date: default to Topo DXF file's `fs::metadata().modified()` date formatted as "D MMMM YYYY", prompting user with that default.
3. Keep CLI arguments having highest precedence so non-interactive CI/CD and scripts continue working seamlessly.

**Tech Stack:** Rust (2021 edition), std::time, std::fs, std::path.

---

### Task 1: Date Formatting & File Modified Helpers

**Files:**
- Create/Modify: `src/config.rs` (or `src/utils.rs` / helper module exposed in `src/lib.rs`)
- Test: `tests/test_date_helpers.rs`

- [ ] **Step 1: Write the failing test for date formatting and file modification date extraction**

Create `tests/test_date_helpers.rs`:
```rust
use rainbow_contour::config::{format_system_time, get_file_modified_date_string, get_current_date_string};
use std::fs::File;
use std::io::Write;
use std::time::{SystemTime, Duration};

#[test]
fn test_format_system_time_standard() {
    // 2026-09-02 00:00:00 UTC (1788307200 seconds after UNIX_EPOCH)
    let time = SystemTime::UNIX_EPOCH + Duration::from_secs(1788307200);
    let formatted = format_system_time(time);
    assert_eq!(formatted, "2 September 2026");
}

#[test]
fn test_file_modified_date_fallback() {
    let dir = std::env::temp_dir().join("test_rainbow_date");
    let _ = std::fs::create_dir_all(&dir);
    let file_path = dir.join("dummy_topo.dxf");
    {
        let mut f = File::create(&file_path).unwrap();
        f.write_all(b"HEADER").unwrap();
    }
    let date_str = get_file_modified_date_string(file_path.to_str().unwrap());
    assert!(!date_str.is_empty());
    // Should end with 4-digit year like "2026"
    assert!(date_str.split_whitespace().count() == 3);
}

#[test]
fn test_get_current_date_string() {
    let cur = get_current_date_string();
    assert!(!cur.is_empty());
    assert!(cur.split_whitespace().count() == 3);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test test_date_helpers`
Expected: FAIL with missing functions `format_system_time`, `get_file_modified_date_string`, `get_current_date_string`.

- [ ] **Step 3: Implement date formatting helpers in `src/config.rs`**

Add pure `SystemTime` to `D MMMM YYYY` format in `src/config.rs`:
```rust
use std::time::{SystemTime, UNIX_EPOCH};

const MONTH_NAMES: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

pub fn format_system_time(time: SystemTime) -> String {
    let duration = match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d,
        Err(_) => return "1 January 1970".to_string(),
    };
    let total_secs = duration.as_secs();
    let days = (total_secs / 86400) as i64;

    // Civil day calculation from Unix epoch days (Howard Hinnant algorithm)
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u32; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let year = if m <= 2 { y + 1 } else { y };

    let month_idx = ((m - 1) as usize).min(11);
    format!("{} {} {}", d, MONTH_NAMES[month_idx], year)
}

pub fn get_current_date_string() -> String {
    format_system_time(SystemTime::now())
}

pub fn get_file_modified_date_string<P: AsRef<Path>>(path: P) -> String {
    if let Ok(metadata) = fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            return format_system_time(modified);
        }
    }
    get_current_date_string()
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --test test_date_helpers`
Expected: PASS (all 3 tests pass).

- [ ] **Step 5: Verify existing config tests still pass**

Run: `cargo test --test test_config`
Expected: PASS.

---

### Task 2: Interactive Prompting for Kop Metadata in `src/main.rs`

**Files:**
- Modify: `src/main.rs`
- Test: `tests/test_e2e_pipeline.rs`

- [ ] **Step 1: Update `src/main.rs` interactive prompts**

In `src/main.rs`:
1. For Topo Date:
   - If CLI `--topo-date` passed: use it.
   - Else: extract default from Topo DXF file last modified date via `get_file_modified_date_string(&topo_path)`. (If `config.topo_date` is non-empty and not default, or if fallback needed).
   - If interactive (i.e. CLI arg wasn't provided), prompt: `prompt_input("Enter Topo Survey Date", true, &default_topo_date)`.
2. For Map Title / Project Title:
   - If CLI `--rainbow-title` passed: use it.
   - Else prompt: `prompt_input("Enter Map Title / Project Title", true, &config.default_title)`.
3. For Company Name:
   - If CLI `--company` passed: use it.
   - Else prompt: `prompt_input("Enter Company Name", true, &config.company_name)`.
4. For Drawn By:
   - If CLI `--drawn-by` passed: use it.
   - Else prompt: `prompt_input("Enter Drawn By", true, &config.drawn_by)`.
5. For Date Created:
   - Automatically determine today's date via `get_current_date_string()`.
6. For Design Name:
   - If CLI `--design-name` passed: use it.
   - Else prompt: `prompt_input("Enter Design Name", true, &default_design_name)`.

- [ ] **Step 2: Update e2e tests to verify the pipeline runs and preserves CLI override flags**

Run: `cargo test --test test_e2e_pipeline`
Expected: PASS.

- [ ] **Step 3: Run full test suite**

Run: `cargo test --bins --tests`
Expected: PASS with all tests passing green.

---

### Task 3: Verification & Interactive Check

**Files:**
- Run binary in testing mode or verify generated artifacts

- [ ] **Step 1: Check code formatting and compilation**

Run: `cargo check`
Run: `cargo test --bins --tests`

- [ ] **Step 2: Verify `config.dat` and documentation sync**

Ensure `config.dat` comments and `src/config.rs` remain completely consistent.
