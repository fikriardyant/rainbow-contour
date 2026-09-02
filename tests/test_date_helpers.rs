use rainbow_contour::config::{format_system_time, get_current_date_string, get_file_modified_date_string};
use std::fs::File;
use std::io::Write;
use std::time::{Duration, SystemTime};

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
    assert_eq!(date_str.split_whitespace().count(), 3);
}

#[test]
fn test_get_current_date_string() {
    let cur = get_current_date_string();
    assert!(!cur.is_empty());
    assert_eq!(cur.split_whitespace().count(), 3);
}
