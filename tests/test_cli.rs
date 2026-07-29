use std::process::Command;

#[test]
fn test_cli_argument_parsing() {
    let output = Command::new("cargo")
        .args(&["run", "--", "--help"])
        .output()
        .expect("Failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Rainbow Contour Cut & Fill Engine"));
    assert!(stdout.contains("--topo"));
    assert!(stdout.contains("--design"));
    assert!(stdout.contains("--boundary"));
}
